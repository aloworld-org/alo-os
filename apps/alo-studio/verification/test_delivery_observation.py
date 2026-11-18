"""Test-first delivery observations on a private session bus, never installed proof."""

import json
import os
import pathlib
import select
import subprocess
import sys
import time
import unittest

import dbus

from delivery_observation import DeliveryObservation


SERVICE = r'''
import json, sys
import dbus, dbus.service
from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib
DBusGMainLoop(set_as_default=True)
bus = dbus.SessionBus(private=True)
name = dbus.service.BusName('org.aloworld.Studio', bus, do_not_queue=True)
class Recorder(dbus.service.Object):
    def __init__(self):
        super().__init__(name, '/org/aloworld/Studio/Recorder')
        self.calls = []
    @dbus.service.method('org.aloworld.Studio.Recorder1', in_signature='', out_signature='')
    def EnableMicrophone(self):
        self.calls.append('enable')
    @dbus.service.method('org.aloworld.Studio.Recorder1', in_signature='', out_signature='')
    def DisableMicrophone(self):
        self.calls.append('disable')
    @dbus.service.method('org.aloworld.Studio.Recorder1', in_signature='', out_signature='')
    def Finish(self):
        self.calls.append('finish')
    @dbus.service.method('org.aloworld.Studio.Recorder1', in_signature='', out_signature='s')
    def GetStatus(self):
        return json.dumps({'actual_fixture_calls': self.calls})
recorder = Recorder()
print('ready', flush=True)
GLib.MainLoop().run()
'''


@unittest.skipUnless(os.environ.get('STUDIO_PRIVATE_UI_SESSION') == '1',
                     'requires the explicitly isolated verification session bus')
class DeliveryEvidence(unittest.TestCase):
    def setUp(self):
        self.service = None
        self.bus = dbus.SessionBus(private=True)
        self.addCleanup(self.bus.close)
        self.addCleanup(self.stop_service)
        self.start_service()

    def start_service(self, source=SERVICE):
        self.service = subprocess.Popen([sys.executable, '-I', '-B', '-c', source],
                                        stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        ready, _, _ = select.select([self.service.stdout], [], [], 5)
        self.assertTrue(ready, 'the independent D-Bus fixture did not start')
        self.assertEqual(self.service.stdout.readline(), 'ready\n')

    def stop_service(self):
        if self.service is not None:
            self.service.terminate()
            self.service.communicate(timeout=5)
            self.service = None

    def interface(self, owner='org.aloworld.Studio'):
        return dbus.Interface(self.bus.get_object(owner, '/org/aloworld/Studio/Recorder',
                                                  introspect=False),
                              'org.aloworld.Studio.Recorder1')

    def observe(self):
        observation = DeliveryObservation()
        self.addCleanup(observation.close)
        return observation

    def test_one_actual_delivery_and_a_correlated_barrier(self):
        observation = self.observe()
        self.interface().EnableMicrophone(timeout=3)
        actual = json.loads(str(self.interface().GetStatus(timeout=3)))
        self.assertEqual(actual['actual_fixture_calls'], ['enable'])
        summary = observation.finish(enabled=True)
        self.assertEqual(set(summary), {'method', 'deliveries', 'barrier_observed', 'owner'})
        self.assertEqual(summary['method'], 'EnableMicrophone')
        self.assertEqual(summary['deliveries'], 1)
        self.assertIs(summary['barrier_observed'], True)
        self.assertEqual(summary['owner'], str(self.bus.get_name_owner('org.aloworld.Studio')))

    def test_a_duplicate_is_rejected_even_when_the_final_option_would_match(self):
        observation = self.observe()
        self.interface().EnableMicrophone(timeout=3)
        self.interface().EnableMicrophone(timeout=3)
        self.assertEqual(json.loads(str(self.interface().GetStatus(timeout=3)))['actual_fixture_calls'],
                         ['enable', 'enable'])
        with self.assertRaises(ValueError):
            observation.finish(enabled=True)

    def test_calls_before_the_observation_cannot_answer_it(self):
        self.interface().EnableMicrophone(timeout=3)
        observation = self.observe()
        with self.assertRaises(ValueError):
            observation.finish(enabled=True)

    def test_wrong_or_additional_change_methods_refuse(self):
        for calls in [('DisableMicrophone',), ('EnableMicrophone', 'Finish'),
                      ('EnableMicrophone', 'DisableMicrophone')]:
            with self.subTest(calls=calls):
                observation = self.observe()
                for member in calls:
                    getattr(self.interface(), member)(timeout=3)
                with self.assertRaises(ValueError):
                    observation.finish(enabled=True)
                observation.close()

    def test_the_unique_owner_alias_cannot_hide_a_duplicate(self):
        observation = self.observe()
        self.interface().EnableMicrophone(timeout=3)
        owner = str(self.bus.get_name_owner('org.aloworld.Studio'))
        self.interface(owner).EnableMicrophone(timeout=3)
        with self.assertRaises(ValueError):
            observation.finish(enabled=True)

    def test_an_unqualified_actual_delivery_cannot_hide_a_duplicate(self):
        observation = self.observe()
        self.interface().EnableMicrophone(timeout=3)
        request = dbus.lowlevel.MethodCallMessage('org.aloworld.Studio',
                                                  '/org/aloworld/Studio/Recorder', None,
                                                  'EnableMicrophone')
        reply = self.bus.send_message_with_reply_and_block(request, 3)
        self.assertEqual(reply.get_type(), dbus.lowlevel.MESSAGE_TYPE_METHOD_RETURN)
        self.assertEqual(json.loads(str(self.interface().GetStatus(timeout=3)))['actual_fixture_calls'],
                         ['enable','enable'])
        with self.assertRaises(ValueError):
            observation.finish(enabled=True)

    def test_disable_and_interleaved_status_reads_are_observed(self):
        observation = self.observe()
        self.interface().GetStatus(timeout=3)
        self.interface().DisableMicrophone(timeout=3)
        self.interface().GetStatus(timeout=3)
        summary = observation.finish(enabled=False)
        self.assertEqual((summary['method'], summary['deliveries']), ('DisableMicrophone', 1))

    def test_owner_replacement_invalidates_the_observation(self):
        observation = self.observe()
        self.interface().EnableMicrophone(timeout=3)
        self.stop_service()
        self.start_service()
        with self.assertRaises(ValueError):
            observation.finish(enabled=True)

    def test_missing_owner_refuses_without_starting_studio(self):
        self.stop_service()
        with self.assertRaises((ValueError, dbus.DBusException)):
            self.observe()
        self.assertFalse(self.bus.name_has_owner('org.aloworld.Studio'))

    def test_finish_and_close_cannot_be_reused_to_certify_a_later_action(self):
        observation = self.observe()
        self.interface().EnableMicrophone(timeout=3)
        observation.finish(enabled=True)
        with self.assertRaises(ValueError):
            observation.finish(enabled=True)
        observation.close()
        observation.close()
        with self.assertRaises(ValueError):
            observation.finish(enabled=True)

    def test_expected_action_is_a_boolean_not_a_coercion(self):
        for enabled in (None, 'false', 0, 1):
            with self.subTest(enabled=enabled):
                observation = self.observe()
                self.interface().EnableMicrophone(timeout=3)
                with self.assertRaises(ValueError):
                    observation.finish(enabled=enabled)
                observation.close()

    def test_an_actual_method_error_is_not_successful_delivery(self):
        self.stop_service()
        self.start_service(SERVICE.replace("self.calls.append('enable')",
                                           "raise dbus.exceptions.DBusException('fixture refusal', name='org.aloworld.Studio.Refused')"))
        observation = self.observe()
        with self.assertRaises(dbus.DBusException):
            self.interface().EnableMicrophone(timeout=3)
        with self.assertRaises(ValueError):
            observation.finish(enabled=True)

    def test_a_no_reply_request_cannot_certify_a_successful_reply(self):
        observation = self.observe()
        request = dbus.lowlevel.MethodCallMessage('org.aloworld.Studio',
                                                  '/org/aloworld/Studio/Recorder',
                                                  'org.aloworld.Studio.Recorder1',
                                                  'EnableMicrophone')
        request.set_no_reply(True)
        self.bus.send_message(request)
        self.bus.flush()
        self.assertEqual(json.loads(str(self.interface().GetStatus(timeout=3)))['actual_fixture_calls'],
                         ['enable'])
        with self.assertRaises(ValueError):
            observation.finish(enabled=True)

    def test_a_signature_error_cannot_be_hidden_beside_one_valid_delivery(self):
        observation = self.observe()
        self.interface().EnableMicrophone(timeout=3)
        with self.assertRaises(dbus.DBusException):
            self.interface().EnableMicrophone(dbus.String('not an offered argument'), timeout=3)
        self.assertEqual(json.loads(str(self.interface().GetStatus(timeout=3)))['actual_fixture_calls'],
                         ['enable'])
        with self.assertRaises(ValueError):
            observation.finish(enabled=True)

    def test_wrong_object_or_interface_calls_do_not_substitute_for_delivery(self):
        for path, interface in [('/org/aloworld/Studio/Other', 'org.aloworld.Studio.Recorder1'),
                                ('/org/aloworld/Studio/Recorder', 'org.aloworld.Studio.Other')]:
            with self.subTest(path=path, interface=interface):
                observation = self.observe()
                other = dbus.Interface(self.bus.get_object('org.aloworld.Studio', path, introspect=False),
                                       interface)
                with self.assertRaises(dbus.DBusException):
                    other.EnableMicrophone(timeout=3)
                with self.assertRaises(ValueError):
                    observation.finish(enabled=True)
                observation.close()

    def test_a_failed_status_barrier_cannot_certify_a_successful_change(self):
        self.stop_service()
        self.start_service(SERVICE.replace("return json.dumps({'actual_fixture_calls': self.calls})",
                                           "raise dbus.exceptions.DBusException('fixture status refusal', name='org.aloworld.Studio.Refused')"))
        observation = self.observe()
        self.interface().EnableMicrophone(timeout=3)
        with self.assertRaises(ValueError):
            observation.finish(enabled=True)

    def test_a_silent_barrier_refuses_within_the_explicit_bound(self):
        self.stop_service()
        self.start_service('import time\n' + SERVICE.replace(
            "return json.dumps({'actual_fixture_calls': self.calls})",
            "time.sleep(3); return json.dumps({'actual_fixture_calls': self.calls})"))
        observation = DeliveryObservation(timeout=0.15)
        self.addCleanup(observation.close)
        self.interface().EnableMicrophone(timeout=3)
        started = time.monotonic()
        with self.assertRaises(ValueError):
            observation.finish(enabled=True)
        self.assertLess(time.monotonic() - started, 1.5,
                        'a silent application must not hold the controller indefinitely')

    def test_timeout_requires_a_positive_finite_number_with_a_fixed_upper_bound(self):
        for timeout in (None, True, False, '0.1', 0, -0.1, float('nan'),
                        float('inf'), -float('inf'), 10.001):
            with self.subTest(timeout=timeout):
                with self.assertRaises(ValueError):
                    observation = DeliveryObservation(timeout=timeout)
                    observation.close()

    def test_a_burst_cannot_drop_extra_deliveries_and_report_one(self):
        observation = self.observe()
        for _ in range(257):
            self.interface().EnableMicrophone(timeout=3)
        self.assertEqual(len(json.loads(str(self.interface().GetStatus(timeout=3)))['actual_fixture_calls']),
                         257)
        with self.assertRaises(ValueError):
            observation.finish(enabled=True)

    def test_overflowing_the_bounded_trace_cannot_certify_one_early_change(self):
        # The private instrument has a fixed finite message budget. Even read
        # noise must not silently overwrite/drop evidence and leave success.
        observation = self.observe()
        self.interface().EnableMicrophone(timeout=3)
        for _ in range(4097):
            self.interface().GetStatus(timeout=3)
        with self.assertRaises(ValueError):
            observation.finish(enabled=True)

    def test_a_disconnected_private_bus_refuses_instead_of_certifying_cached_calls(self):
        # A nested dbus-run-session owns this daemon. Never kill the test runner's bus.
        import delivery_observation
        module_dir = str(pathlib.Path(delivery_observation.__file__).parent)
        script = f'''
import os, signal, subprocess, sys, time
sys.path.insert(0, {module_dir!r})
import dbus
from delivery_observation import DeliveryObservation
service = subprocess.Popen([sys.executable, '-I', '-B', '-c', {SERVICE!r}],
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
try:
    assert service.stdout.readline() == 'ready\\n'
    bus = dbus.SessionBus(private=True)
    observation = DeliveryObservation(timeout=0.2)
    recorder = dbus.Interface(bus.get_object('org.aloworld.Studio', '/org/aloworld/Studio/Recorder', introspect=False),
                              'org.aloworld.Studio.Recorder1')
    recorder.EnableMicrophone(timeout=2)
    driver = dbus.Interface(bus.get_object('org.freedesktop.DBus', '/org/freedesktop/DBus'), 'org.freedesktop.DBus')
    daemon_pid = int(driver.GetConnectionUnixProcessID('org.freedesktop.DBus'))
    os.kill(daemon_pid, signal.SIGTERM)
    started = time.monotonic()
    try:
        observation.finish(enabled=True)
    except ValueError:
        assert time.monotonic() - started < 1.5
        print('disconnection refused', flush=True)
    else:
        raise AssertionError('a lost monitor cannot certify delivery')
    finally:
        observation.close()
finally:
    service.terminate()
    service.communicate(timeout=2)
'''
        completed = subprocess.run(['dbus-run-session', '--', sys.executable, '-I', '-B', '-c', script],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=7)
        self.assertEqual(completed.returncode, 0, completed.stderr[-4000:])
        self.assertEqual(completed.stdout, 'disconnection refused\n')


if __name__ == '__main__':
    unittest.main()
