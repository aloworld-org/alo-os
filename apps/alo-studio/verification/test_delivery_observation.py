"""Test-first delivery observations on a private session bus, never installed proof."""

import json
import os
import pathlib
import select
import subprocess
import sys
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

    def start_service(self):
        self.service = subprocess.Popen([sys.executable, '-I', '-B', '-c', SERVICE],
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


if __name__ == '__main__':
    unittest.main()
