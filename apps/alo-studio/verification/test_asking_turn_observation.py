"""Observe process exit, not model success, on a test-only systemd-shaped bus.

The injected session bus is a synthetic fixture, never an installed asking
turn. The actual controller must use the observer's fixed system-bus default.
One-time WSL systemd255 observation established final Service exit properties
precede inactive/dead and UnitRemoved, and startup signals can trail snapshots.
"""

import json
import os
import select
import subprocess
import sys
import time
import unittest

import dbus

from asking_turn_observation import AskingTurnObservation


NAME = 'alo-asking@11-1000-alo.service'
OTHER = 'alo-asking@12-1000-alo.service'
INVOCATION = 'f33736a8035e47928569eeb33d4b8e92'
UNIT = 'org.freedesktop.systemd1.Unit'
SERVICE = 'org.freedesktop.systemd1.Service'


FIXTURE = r'''
import json, sys
import dbus, dbus.service, dbus.lowlevel
from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib
DBusGMainLoop(set_as_default=True)
bus = dbus.SessionBus(private=True)
owner = dbus.service.BusName('org.freedesktop.systemd1', bus, do_not_queue=True)
UNIT = 'org.freedesktop.systemd1.Unit'
SERVICE = 'org.freedesktop.systemd1.Service'
def path(name):
    return '/org/freedesktop/systemd1/unit/' + ''.join(c if c.isascii() and c.isalnum() else '_%02x' % ord(c) for c in name)
def typed(key, value):
    if isinstance(value,dict) and '__wire__' in value:
        constructors = {'u':dbus.UInt32,'t':dbus.UInt64,'x':dbus.Int64,
                        'i':dbus.Int32,'b':dbus.Boolean,'d':dbus.Double,
                        'au':lambda items:dbus.Array([dbus.UInt32(x) for x in items],signature='u')}
        return constructors[value['__wire__']](value['value'])
    if key == 'InvocationID':
        return dbus.Array([dbus.Byte(x) for x in value], signature='y') if isinstance(value, list) else value
    if key in ('MainPID','ExecMainPID'):
        return dbus.UInt32(value) if type(value) is int and value >= 0 else value
    if key in ('ExecMainCode','ExecMainStatus'):
        return dbus.Int32(value) if type(value) is int else value
    if key.endswith('TimestampMonotonic'):
        return dbus.UInt64(value) if type(value) is int and value >= 0 else value
    if key == 'ExecStart':
        return dbus.Array([dbus.Struct((entry[0], dbus.Array(entry[1], signature='s'), False,
                                       dbus.UInt64(0), dbus.UInt64(0), dbus.UInt64(0), dbus.UInt64(0),
                                       dbus.UInt32(0), dbus.Int32(0), dbus.Int32(0)), signature='sasbttttuii')
                           for entry in value], signature='(sasbttttuii)')
    return value
class Entry(dbus.service.Object):
    def __init__(self, name, overrides):
        super().__init__(owner, path(name))
        self.name = name
        self.props = {
            UNIT: {'Id': name, 'ActiveState':'active', 'SubState':'running',
                   'InvocationID': list(bytes.fromhex('f33736a8035e47928569eeb33d4b8e92'))},
            SERVICE: {'Type':'exec', 'User':'alo-agent', 'Group':'alo-agent',
                      'MainPID':2710198, 'ExecMainPID':2710198,
                      'ExecMainStartTimestampMonotonic':477946100265,
                      'ExecMainExitTimestampMonotonic':0, 'ExecMainCode':0,
                      'ExecMainStatus':0, 'Result':'success',
                      'ExecStart':[['/usr/bin/alo-agent',['/usr/bin/alo-agent']]]}}
        for interface, changed in overrides.items():
            for key in changed.pop('__missing__',[]):
                self.props[interface].pop(key,None)
            self.props[interface].update(changed)
    @dbus.service.method('org.freedesktop.DBus.Properties', in_signature='s', out_signature='a{sv}')
    def GetAll(self, interface):
        return {k:typed(k,v) for k,v in self.props[str(interface)].items()}
    @dbus.service.method('org.freedesktop.DBus.Properties', in_signature='ss', out_signature='v')
    def Get(self, interface, prop):
        return typed(str(prop),self.props[str(interface)][str(prop)])
    @dbus.service.signal('org.freedesktop.DBus.Properties', signature='sa{sv}as')
    def PropertiesChanged(self, interface, changed, invalidated):
        pass
class Manager(dbus.service.Object):
    def __init__(self):
        super().__init__(owner, '/org/freedesktop/systemd1')
        self.units = {}
        self.subscribed = False
        self.race = ''
    @dbus.service.method('org.freedesktop.systemd1.Manager', in_signature='', out_signature='')
    def Subscribe(self):
        self.subscribed = True
        if self.race == 'subscribe':
            self.race = ''
            self.Add('alo-asking@11-1000-alo.service','{}')
            self.Remove('alo-asking@11-1000-alo.service')
    @dbus.service.method('org.freedesktop.systemd1.Manager', in_signature='', out_signature='')
    def Unsubscribe(self):
        self.subscribed = False
    @dbus.service.method('org.freedesktop.systemd1.Manager', in_signature='', out_signature='a(ssssssouso)')
    def ListUnits(self):
        if not self.subscribed:
            raise dbus.exceptions.DBusException('must subscribe before snapshot')
        rows = [(name,'fixture','loaded', entry.props[UNIT]['ActiveState'],entry.props[UNIT]['SubState'],
                 '',path(name),dbus.UInt32(0),'','/') for name,entry in self.units.items()]
        if self.race == 'list':
            self.race = ''
            self.Add('alo-asking@11-1000-alo.service','{}')
        return rows
    @dbus.service.method('org.freedesktop.systemd1.Manager', in_signature='s', out_signature='o')
    def GetUnit(self, name):
        if str(name) not in self.units:
            raise dbus.exceptions.DBusException('unit absent')
        return path(str(name))
    @dbus.service.signal('org.freedesktop.systemd1.Manager', signature='so')
    def UnitNew(self, name, object_path):
        pass
    @dbus.service.signal('org.freedesktop.systemd1.Manager', signature='so')
    def UnitRemoved(self, name, object_path):
        pass
    @dbus.service.method('org.aloworld.Studio.LifecycleFixture', in_signature='ss', out_signature='')
    def Add(self, name, overrides):
        name = str(name)
        self.units[name] = Entry(name,json.loads(str(overrides)))
        self.UnitNew(name,path(name))
    @dbus.service.method('org.aloworld.Studio.LifecycleFixture', in_signature='sss', out_signature='')
    def Change(self, name, interface, changed):
        entry = self.units[str(name)]
        values = json.loads(str(changed))
        entry.props[str(interface)].update(values)
        entry.PropertiesChanged(interface,{k:typed(k,v) for k,v in values.items()},dbus.Array([],signature='s'))
    @dbus.service.method('org.aloworld.Studio.LifecycleFixture', in_signature='ssss', out_signature='')
    def Invalidate(self, name, interface, hidden, invalidated):
        entry = self.units[str(name)]
        values = json.loads(str(hidden))
        for key in values.pop('__missing__',[]):
            entry.props[str(interface)].pop(key,None)
        entry.props[str(interface)].update(values)
        entry.PropertiesChanged(interface,dbus.Dictionary({},signature='sv'),
                                dbus.Array(json.loads(str(invalidated)),signature='s'))
    @dbus.service.method('org.aloworld.Studio.LifecycleFixture', in_signature='s', out_signature='')
    def Race(self, when):
        self.race = str(when)
    @dbus.service.method('org.aloworld.Studio.LifecycleFixture', in_signature='su', out_signature='')
    def Flood(self, name, count):
        for _ in range(int(count)):
            self.units[str(name)].PropertiesChanged(SERVICE,{'Result':'success'},dbus.Array([],signature='s'))
    @dbus.service.method('org.aloworld.Studio.LifecycleFixture', in_signature='s', out_signature='')
    def Remove(self, name):
        name = str(name)
        entry = self.units.pop(name)
        entry.remove_from_connection()
        self.UnitRemoved(name,path(name))
    @dbus.service.method('org.aloworld.Studio.LifecycleFixture', in_signature='s', out_signature='')
    def BadSignal(self, name):
        message = dbus.lowlevel.SignalMessage(path(str(name)), 'org.freedesktop.DBus.Properties', 'PropertiesChanged')
        message.append('not the documented property signature',signature='s')
        bus.send_message(message)
    @dbus.service.method('org.aloworld.Studio.LifecycleFixture', in_signature='s', out_signature='')
    def AnnounceAgain(self, name):
        self.UnitNew(name,path(str(name)))
    @dbus.service.method('org.aloworld.Studio.LifecycleFixture', in_signature='', out_signature='')
    def CycleOwner(self):
        bus.release_name('org.freedesktop.systemd1')
        bus.request_name('org.freedesktop.systemd1',dbus.bus.NAME_FLAG_DO_NOT_QUEUE)
manager = Manager()
print('ready',flush=True)
GLib.MainLoop().run()
'''


@unittest.skipUnless(os.environ.get('STUDIO_PRIVATE_UI_SESSION') == '1',
                     'requires an explicitly isolated verification session bus')
class AskingLifecycle(unittest.TestCase):
    def setUp(self):
        self.process = subprocess.Popen([sys.executable, '-I', '-B', '-c', FIXTURE],
                                        stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.addCleanup(self.stop_fixture)
        ready, _, _ = select.select([self.process.stdout], [], [], 5)
        self.assertTrue(ready, 'independent systemd-shaped fixture did not start')
        self.assertEqual(self.process.stdout.readline(), 'ready\n')
        self.bus = dbus.SessionBus(private=True)
        self.bus.set_exit_on_disconnect(False)
        self.addCleanup(self.bus.close)
        self.control = dbus.Interface(self.bus.get_object('org.freedesktop.systemd1',
                                      '/org/freedesktop/systemd1'), 'org.aloworld.Studio.LifecycleFixture')

    def stop_fixture(self):
        self.process.terminate()
        self.process.communicate(timeout=5)

    def add(self, name=NAME, overrides=None):
        self.control.Add(name, json.dumps(overrides or {}), timeout=2)

    def change(self, interface, values, name=NAME):
        self.control.Change(name, interface, json.dumps(values), timeout=2)

    def observe(self, timeout=0.2):
        observer = AskingTurnObservation(bus_connection=self.bus, timeout=timeout)
        self.addCleanup(observer.close)
        return observer

    def pending(self):
        observer = self.observe()
        self.add()
        identity = observer.capture_pending()
        self.assertEqual(set(identity), {'unit','path','invocation_id','pid','start_us'})
        self.assertEqual(identity, {'unit':NAME,
            'path':'/org/freedesktop/systemd1/unit/alo_2dasking_4011_2d1000_2dalo_2eservice',
            'invocation_id':INVOCATION,'pid':2710198,'start_us':477946100265})
        return observer, identity

    def end(self, **overrides):
        self.change(SERVICE, {'MainPID':0,'ExecMainPID':2710198,'Result':'success',
                    'ExecMainCode':1,'ExecMainStatus':0,
                    'ExecMainStartTimestampMonotonic':477946100265,
                    'ExecMainExitTimestampMonotonic':477946805484, **overrides})
        self.change(UNIT, {'ActiveState':'inactive','SubState':'dead'})

    def test_cached_real_exit_facts_survive_unit_garbage_collection(self):
        observer, identity = self.pending()
        self.end()
        self.control.Remove(NAME, timeout=2)
        actual = observer.finish()
        self.assertEqual(actual, {**identity,'result':'success','exit_code':1,
                                  'exit_status':0,'exit_us':477946805484})
        self.assertEqual(set(actual), set(identity) | {'result','exit_code','exit_status','exit_us'})

    def test_existing_asking_unit_refuses_the_initial_baseline(self):
        self.add()
        with self.assertRaises(ValueError):
            self.observe()

    def test_two_fresh_units_cannot_be_assigned_to_this_request(self):
        observer = self.observe()
        self.add()
        self.add(OTHER)
        with self.assertRaises(ValueError):
            observer.capture_pending()

    def test_unrelated_units_do_not_substitute_for_a_real_asking_unit(self):
        observer = self.observe()
        self.add('studio-private-fixture.service')
        started = time.monotonic()
        with self.assertRaises(ValueError):
            observer.capture_pending()
        self.assertLess(time.monotonic()-started, 4)

    def test_missing_or_invalid_pending_identity_refuses(self):
        cases = [(SERVICE,field,bad) for field,bad in [
            ('User','alo'),('Group','alo'),('MainPID',0),('MainPID',True),('MainPID','2710198'),
            ('ExecMainPID',99),('ExecMainStartTimestampMonotonic',0),
            ('ExecStart',[['/usr/bin/alo-agent',['/usr/bin/alo-agent','--as=root']]]),
            ('ExecStart',[['/usr/bin/other',['/usr/bin/other']]])]]
        cases += [(UNIT,'InvocationID',[]),(UNIT,'InvocationID',[0]*16),
                  (UNIT,'InvocationID',[1]*15),(UNIT,'ActiveState','inactive'),
                  (UNIT,'SubState','dead')]
        for interface, field, bad in cases:
            with self.subTest(interface=interface,field=field,bad=bad):
                observer = self.observe()
                self.add(overrides={interface:{field:bad}})
                with self.assertRaises(ValueError):
                    observer.capture_pending()
                observer.close()
                self.control.Remove(NAME,timeout=2)

    def test_missing_required_identity_fields_refuse(self):
        for interface, field in [(UNIT,'InvocationID'),(UNIT,'Id'),(SERVICE,'User'),
                                 (SERVICE,'Group'),(SERVICE,'MainPID'),(SERVICE,'ExecMainPID'),
                                 (SERVICE,'ExecMainStartTimestampMonotonic'),(SERVICE,'ExecStart')]:
            with self.subTest(interface=interface,field=field):
                observer = self.observe()
                self.add(overrides={interface:{'__missing__':[field]}})
                with self.assertRaises(ValueError):
                    observer.capture_pending()
                observer.close()
                self.control.Remove(NAME,timeout=2)

    def test_stale_inactive_notification_without_exit_facts_is_not_completion(self):
        observer, _ = self.pending()
        self.change(UNIT, {'ActiveState':'inactive','SubState':'dead'})
        self.change(UNIT, {'ActiveState':'active','SubState':'running'})
        self.end()
        self.assertEqual(observer.finish()['exit_status'], 0)

    def test_gc_without_cached_exit_facts_refuses(self):
        observer, _ = self.pending()
        self.control.Remove(NAME,timeout=2)
        with self.assertRaises(ValueError):
            observer.finish()

    def test_inactive_or_zero_status_without_an_actual_exit_refuses(self):
        observer, _ = self.pending()
        self.change(SERVICE, {'MainPID':0,'ExecMainStatus':0,'Result':'success'})
        self.change(UNIT, {'ActiveState':'inactive','SubState':'dead'})
        with self.assertRaises(ValueError):
            observer.finish()

    def test_unsuccessful_or_mismatched_terminal_facts_refuse(self):
        for overrides in [ {'Result':'exit-code'}, {'ExecMainCode':2}, {'ExecMainStatus':1},
                           {'ExecMainPID':2710199}, {'MainPID':2710198},
                           {'ExecMainStartTimestampMonotonic':477946100266},
                           {'ExecMainExitTimestampMonotonic':0},
                           {'ExecMainExitTimestampMonotonic':477946100264} ]:
            with self.subTest(overrides=overrides):
                observer, _ = self.pending()
                self.end(**overrides)
                with self.assertRaises(ValueError):
                    observer.finish()
                observer.close()
                self.control.Remove(NAME,timeout=2)

    def test_invocation_replacement_is_sticky_even_if_old_identity_reappears(self):
        observer, _ = self.pending()
        self.change(UNIT, {'InvocationID':[8]*16})
        self.change(UNIT, {'InvocationID':list(bytes.fromhex(INVOCATION))})
        self.end()
        with self.assertRaises(ValueError):
            observer.finish()

    def test_a_second_asking_unit_during_the_window_refuses_even_if_removed(self):
        observer, _ = self.pending()
        self.add(OTHER)
        self.control.Remove(OTHER,timeout=2)
        self.end()
        with self.assertRaises(ValueError):
            observer.finish()

    def test_duplicate_new_announcement_cannot_reset_the_selected_identity(self):
        observer, _ = self.pending()
        self.control.AnnounceAgain(NAME,timeout=2)
        self.end()
        with self.assertRaises(ValueError):
            observer.finish()

    def test_removed_then_recreated_same_path_cannot_supply_the_original_exit(self):
        observer, _ = self.pending()
        self.control.Remove(NAME,timeout=2)
        self.add()
        self.end()
        with self.assertRaises(ValueError):
            observer.finish()

    def test_systemd_owner_loss_cannot_be_hidden_by_reacquisition(self):
        observer, _ = self.pending()
        self.control.CycleOwner(timeout=2)
        self.end()
        with self.assertRaises(ValueError):
            observer.finish()

    def test_wrong_signature_at_the_selected_path_is_not_terminal_evidence(self):
        observer, _ = self.pending()
        self.control.BadSignal(NAME,timeout=2)
        self.end()
        with self.assertRaises(ValueError):
            observer.finish()

    def test_no_exit_is_bounded_and_does_not_become_success(self):
        observer, _ = self.pending()
        started = time.monotonic()
        with self.assertRaises(ValueError):
            observer.finish()
        self.assertLess(time.monotonic()-started, 1.5)

    def test_closed_or_finished_observations_cannot_be_reused(self):
        observer, _ = self.pending()
        self.end()
        observer.finish()
        for action in (observer.capture_pending,observer.finish):
            with self.assertRaises(ValueError):
                action()
        observer.close()
        observer.close()
        with self.assertRaises(ValueError):
            observer.finish()

    def test_finish_before_capturing_the_pending_turn_refuses(self):
        observer = self.observe()
        self.add()
        self.end()
        with self.assertRaises(ValueError):
            observer.finish()

    def test_timeout_is_strictly_positive_finite_and_at_most_five_minutes(self):
        for timeout in (None,False,True,'1',0,-1,float('nan'),float('inf'),300.001):
            with self.subTest(timeout=timeout):
                with self.assertRaises(ValueError):
                    observer = AskingTurnObservation(bus_connection=self.bus,timeout=timeout)
                    observer.close()

    def test_numeric_wire_types_cannot_be_coerced_into_pending_identity(self):
        cases = [(SERVICE,'MainPID','t',2710198),(SERVICE,'MainPID','x',2710198),
                 (SERVICE,'MainPID','d',2710198.0),(SERVICE,'ExecMainPID','t',2710198),
                 (SERVICE,'ExecMainStartTimestampMonotonic','x',477946100265),
                 (UNIT,'InvocationID','au',list(bytes.fromhex(INVOCATION)))]
        for interface,field,wire,value in cases:
            with self.subTest(interface=interface,field=field,wire=wire):
                observer = self.observe()
                self.add(overrides={interface:{field:{'__wire__':wire,'value':value}}})
                with self.assertRaises(ValueError):
                    observer.capture_pending()
                observer.close()
                self.control.Remove(NAME,timeout=2)

    def test_numeric_wire_types_cannot_be_coerced_into_successful_exit_facts(self):
        for field,wire,value in [('MainPID','b',False),('MainPID','t',0),
                                 ('ExecMainPID','t',2710198),('ExecMainCode','u',1),
                                 ('ExecMainStatus','b',False),('ExecMainStatus','t',0),
                                 ('ExecMainExitTimestampMonotonic','x',477946805484)]:
            with self.subTest(field=field,wire=wire):
                observer, _ = self.pending()
                self.end(**{field:{'__wire__':wire,'value':value}})
                with self.assertRaises(ValueError):
                    observer.finish()
                observer.close()
                self.control.Remove(NAME,timeout=2)

    def test_role_and_executable_mutation_are_sticky_even_after_restoration(self):
        for field,bad,good in [('User','alo','alo-agent'),('Group','alo','alo-agent'),
                               ('ExecStart',[['/usr/bin/other',['/usr/bin/other']]],
                                [['/usr/bin/alo-agent',['/usr/bin/alo-agent']]])]:
            with self.subTest(field=field):
                observer, _ = self.pending()
                self.change(SERVICE,{field:bad})
                self.change(SERVICE,{field:good})
                self.end()
                with self.assertRaises(ValueError):
                    observer.finish()
                observer.close()
                self.control.Remove(NAME,timeout=2)

    def test_benign_execstart_invalidation_matches_actual_systemd_exit_signals(self):
        observer, _ = self.pending()
        self.control.Invalidate(NAME,SERVICE,'{}','["ExecStart"]',timeout=2)
        self.end()
        self.control.Remove(NAME,timeout=2)
        self.assertEqual(observer.finish()['exit_status'],0)

    def test_invalidated_changed_or_missing_identity_cannot_reuse_cached_values(self):
        for interface,field,hidden in [(SERVICE,'User',{'User':'alo'}),
                                      (SERVICE,'Group',{'__missing__':['Group']}),
                                      (UNIT,'InvocationID',{'InvocationID':[8]*16})]:
            with self.subTest(interface=interface,field=field):
                observer, _ = self.pending()
                self.control.Invalidate(NAME,interface,json.dumps(hidden),json.dumps([field]),timeout=2)
                self.end()
                with self.assertRaises(ValueError):
                    observer.finish()
                observer.close()
                self.control.Remove(NAME,timeout=2)

    def test_closing_an_observer_does_not_close_the_injected_callers_bus(self):
        observer = self.observe()
        observer.close()
        self.assertTrue(self.bus.get_is_connected())
        self.add()
        self.control.Remove(NAME,timeout=2)

    def test_an_asking_unit_appearing_and_disappearing_during_subscribe_refuses(self):
        self.control.Race('subscribe',timeout=2)
        with self.assertRaises(ValueError):
            observer = self.observe()
            self.add()
            observer.capture_pending()

    def test_unit_appearance_before_an_empty_list_reply_cannot_be_a_fresh_request(self):
        self.control.Race('list',timeout=2)
        with self.assertRaises(ValueError):
            observer = self.observe()
            observer.capture_pending()

    def test_signal_flood_exhaustion_cannot_drop_evidence_and_certify_exit(self):
        observer, _ = self.pending()
        self.control.Flood(NAME,dbus.UInt32(4097),timeout=3)
        self.end()
        started = time.monotonic()
        with self.assertRaises(ValueError):
            observer.finish()
        self.assertLess(time.monotonic()-started,1.5)


if __name__ == '__main__':
    unittest.main()
