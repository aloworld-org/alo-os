"""Read-only pending-card evidence; literal daemon protocol, never approval."""

import copy
import json
import socket
import threading
import unittest

from pending_microphone import read_pending


def response():
    return {'format': 1, 'tells': {'waiting': {'changes': [{
        'number': 0, 'sentence': {'text': 'enable the microphone for this alo Studio recording',
                                 'came_from': 'the-source'},
        'lapses_in': 300, 'touches': [{'application': 'org.aloworld.Studio'}]}]}}}


class PendingMicrophone(unittest.TestCase):
    def exchange(self, data, *, enabled=True, timeout=.2):
        client, server = socket.socketpair()
        self.addCleanup(client.close)
        self.addCleanup(server.close)
        requests = []

        def serve():
            request = b''
            while not request.endswith(b'\n'):
                part = server.recv(1024)
                if not part:
                    return
                request += part
            requests.append(request)
            try:
                server.sendall(data)
                server.shutdown(socket.SHUT_WR)
            except OSError:
                pass

        worker = threading.Thread(target=serve)
        worker.start()
        try:
            return read_pending(client, enabled=enabled, timeout=timeout)
        finally:
            client.close()
            worker.join(1)
            self.assertFalse(worker.is_alive())
            self.assertEqual(requests, [b'{"format":1,"asks":{"waiting":{}}}\n'])

    def query(self, value, **options):
        return self.exchange((json.dumps(value) + '\n').encode(), **options)

    def test_exact_local_pending_microphone_with_zero_number(self):
        self.assertEqual(self.query(response()), {
            'number': 0, 'sentence': 'enable the microphone for this alo Studio recording'})

    def test_no_pending_change_is_not_an_approval_or_completion(self):
        value = response()
        value['tells']['waiting']['changes'] = []
        self.assertIsNone(self.query(value))

    def test_disable_requires_its_own_exact_sentence(self):
        value = response()
        value['tells']['waiting']['changes'][0]['sentence']['text'] = (
            'disable the microphone for this alo Studio recording')
        self.assertEqual(self.query(value, enabled=False)['number'], 0)
        with self.assertRaises(ValueError):
            self.query(value)

    def test_multiple_changes_are_ambiguous_even_if_one_matches(self):
        value = response()
        value['tells']['waiting']['changes'] *= 2
        with self.assertRaises(ValueError):
            self.query(value)

    def test_wrong_expired_remote_or_text_carrying_change_refuses(self):
        alterations = [
            ('number', True), ('number', -1), ('number', 2**64), ('number', 1.0),
            ('lapses_in', None), ('lapses_in', 0), ('lapses_in', True),
            ('from', 'another machine'), ('texts', [{'argument': 'text', 'text': 'extra'}]),
            ('touches', [{'application': 'other.app'}]),
            ('sentence', {'text': 'enable the microphone for this alo Studio recording',
                          'came_from': 'no-sentence'}),
            ('sentence', {'text': 'turn the microphone on', 'came_from': 'the-source'}),
        ]
        for key, value in alterations:
            with self.subTest(key=key, value=value):
                body = response()
                body['tells']['waiting']['changes'][0][key] = value
                with self.assertRaises(ValueError):
                    self.query(body)

    def test_missing_required_fields_and_wrong_envelopes_refuse(self):
        body = response()
        for key in ('number', 'sentence', 'lapses_in', 'touches'):
            changed = copy.deepcopy(body)
            del changed['tells']['waiting']['changes'][0][key]
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.query(changed)
        for value in ({'format': True, 'tells': body['tells']},
                      {'format': 1, 'tells': {'refused': {'text': 'no'}}},
                      {'format': 1, 'tells': {'waiting': {'changes': None}}}):
            with self.subTest(value=value), self.assertRaises(ValueError):
                self.query(value)

    def test_truncated_duplicate_nonfinite_oversized_and_trailing_data_refuse(self):
        valid = (json.dumps(response()) + '\n').encode()
        for data in (valid[:-1], valid + valid, b'\xff\n', b'{"format":1,"format":1}\n',
                     b'{"format":NaN}\n', b' ' * 65537 + valid):
            with self.subTest(size=len(data)), self.assertRaises(ValueError):
                self.exchange(data)

    def test_silent_connection_times_out_without_becoming_empty_waiting(self):
        client, server = socket.socketpair()
        self.addCleanup(client.close)
        self.addCleanup(server.close)
        with self.assertRaises((ValueError, TimeoutError)):
            read_pending(client, enabled=True, timeout=.05)


if __name__ == '__main__':
    unittest.main()
