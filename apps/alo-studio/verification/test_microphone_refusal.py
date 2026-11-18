"""A real record's stopped shape can establish refusal, never its approval ID.

Synthetic literal entries use alo_record::Happened::Stopped {agent,what,how}.
Entry::refused stores how={"at-the-moment":<person-visible reason>}; it has NO
from_approval field. That number belongs to separate native Waiting evidence.
verify_microphone_refusal(path, checkpoint, *, enabled:bool) returns exactly
agent,verb,recorded_refusals,how,at_full_access. This certifies one recorded
Ask-policy last-moment refusal, not native input, single delivery or app state.
The same bounded regular-file/checkpoint checks as successful deltas apply.
"""

from pathlib import Path
import tempfile
import unittest

import record_delta
from test_record_delta import HEADER, line, ran, read_noise


def refused(*, enabled=False):
    return {'at':{'secs_since_epoch':1760000001,'nanos_since_epoch':12},
        'happened':{'stopped':{'agent':'alo','what':{
            'verb':'studio.enable_microphone' if enabled else 'studio.disable_microphone',
            'effect':'change','arguments':{},
            'sentence':('enable' if enabled else 'disable')+' the microphone for this alo Studio recording'},
            'how':{'at-the-moment':'alo is no longer allowed to use this application'}}}}


class MicrophoneRefusal(unittest.TestCase):
    def setUp(self):
        temporary=tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.path=Path(temporary.name)/'record.jsonl'
        self.path.write_bytes(HEADER)

    def append(self,*entries):
        with self.path.open('ab') as output:
            for entry in entries:
                output.write(line(entry))

    def verify(self,checkpoint,*,enabled=False):
        return record_delta.verify_microphone_refusal(self.path,checkpoint,enabled=enabled)

    def test_one_last_moment_refusal_has_no_invented_approval_or_delivery_claim(self):
        for enabled in (False,True):
            with self.subTest(enabled=enabled):
                self.path.write_bytes(HEADER)
                self.append(refused(enabled=enabled))  # Earlier evidence is outside delta.
                checkpoint=record_delta.snapshot(self.path)
                self.append(read_noise(),refused(enabled=enabled),read_noise())
                before=self.path.read_bytes()
                self.assertEqual(self.verify(checkpoint,enabled=enabled),{
                    'agent':'alo','verb':'studio.enable_microphone' if enabled else 'studio.disable_microphone',
                    'recorded_refusals':1,'how':'at-the-moment','at_full_access':False})
                self.assertEqual(self.path.read_bytes(),before)

    def test_cancellation_or_a_preproposal_refusal_cannot_prove_stale_approval_refusal(self):
        for how in ('by-the-person',{'before-anybody-was-asked':'no access'}):
            with self.subTest(how=how):
                self.path.write_bytes(HEADER)
                checkpoint=record_delta.snapshot(self.path)
                entry=refused()
                entry['happened']['stopped']['how']=how
                self.append(entry)
                with self.assertRaises(ValueError):
                    self.verify(checkpoint)

    def test_wrong_action_or_agent_and_duplicate_refusals_do_not_match(self):
        for change in ('wrong-action','wrong-agent','duplicate','other-studio-change'):
            with self.subTest(change=change):
                self.path.write_bytes(HEADER)
                checkpoint=record_delta.snapshot(self.path)
                entry=refused()
                if change=='wrong-action': entry['happened']['stopped']['what']['verb']='studio.pause'
                if change=='wrong-agent': entry['happened']['stopped']['agent']='other'
                self.append(entry)
                if change=='duplicate': self.append(refused())
                if change=='other-studio-change': self.append(refused(enabled=True))
                with self.assertRaises(ValueError):
                    self.verify(checkpoint)

    def test_a_successful_studio_change_beside_refusal_invalidates_withdrawal_evidence(self):
        checkpoint=record_delta.snapshot(self.path)
        self.append(refused(),ran(enabled=False,full=True))
        with self.assertRaises(ValueError):
            self.verify(checkpoint)

    def test_refusal_requires_ask_policy_and_a_nonempty_actual_reason(self):
        for location,bad in [('flag',True),('flag',0),('how',{'at-the-moment':''}),
                             ('how',{'at-the-moment':False}),('how',{'at-the-moment':5}),
                             ('how',{'at-the-moment':'no','by-the-person':None})]:
            with self.subTest(location=location,bad=bad):
                self.path.write_bytes(HEADER)
                checkpoint=record_delta.snapshot(self.path)
                entry=refused()
                if location=='flag': entry['at_full_access']=bad
                else: entry['happened']['stopped']['how']=bad
                self.append(entry)
                with self.assertRaises(ValueError):
                    self.verify(checkpoint)

    def test_read_noise_or_preexisting_refusal_does_not_supply_a_new_refusal(self):
        self.append(refused())
        checkpoint=record_delta.snapshot(self.path)
        self.append(read_noise())
        with self.assertRaises(ValueError):
            self.verify(checkpoint)

    def test_enabled_is_a_strict_boolean(self):
        checkpoint=record_delta.snapshot(self.path)
        self.append(refused())
        for enabled in (None,0,1,'false'):
            with self.subTest(enabled=enabled):
                with self.assertRaises(ValueError):
                    self.verify(checkpoint,enabled=enabled)

    def test_prefix_rewrite_or_an_unfinished_append_cannot_certify_refusal(self):
        for fault in ('rewrite','partial','rotation'):
            with self.subTest(fault=fault):
                self.path.write_bytes(HEADER)
                checkpoint=record_delta.snapshot(self.path)
                self.append(refused())
                if fault=='rewrite': self.path.write_bytes(b'{"format":2}\n'+line(refused()))
                if fault=='partial': self.path.write_bytes(self.path.read_bytes()+b'{"at":')
                if fault=='rotation':
                    self.path.rename(self.path.with_name('old.jsonl'))
                    self.path.write_bytes(HEADER+line(refused()))
                with self.assertRaises((ValueError,OSError)):
                    self.verify(checkpoint)


if __name__=='__main__':
    unittest.main()
