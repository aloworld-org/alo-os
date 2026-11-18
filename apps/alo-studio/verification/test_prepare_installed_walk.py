"""Independent requirements for private installed-walk artifact preparation.

API: prepare_installed_walk.prepare(normal_dir, reference_dir, output_dir,
                                   expected_source).
Arguments are pathlib.Path directories and a 40-character lowercase Git SHA.
Validation refusals raise ValueError. The return value is unspecified.

Normal checksums.txt names exactly alo-studio-0.1.0.flatpak, source.txt,
features.txt and sdk-versions.txt. Reference reference-checksums.txt names
exactly studio-reference.flatpak, studio-session, source.txt,
reference-features.txt and sdk-versions.txt. Each line is a lowercase SHA256,
two spaces and a simple basename, terminated by newline. Neither checksums
manifest includes itself. Hashes verify original producer names before copying.

On success the fresh output contains studio.flatpak, fixture.flatpak,
studio-session, source.txt, features.txt, sdk-versions.txt and checksums.txt.
The output checksums cover exactly those six payloads under their new names.
Source/features/SDK bytes copied to output are the normal artifact's records;
reference identity and SDK agreement must also be validated. Inputs stay intact.
On any validation failure the output is absent; an existing output is refused
and preserved. A destination inside either original artifact is not fresh input.

All package and zipapp bytes below are inert TEST FIXTURES. These tests neither
create real Flatpaks nor execute a controller, install software or publish files.
"""

import hashlib
import os
import pathlib
import re
import stat
import tempfile
import unittest

from prepare_installed_walk import prepare


SOURCE = '0123456789abcdef0123456789abcdef01234567'
NORMAL_FEATURES = b'normal; verification disabled\n'
REFERENCE_FEATURES = b'private reference only; verification enabled; never the normal application\n'
SDK = b'runtime=org.gnome.Platform/x86_64/49\nsdk=org.gnome.Sdk/x86_64/49\n'
NORMAL_FILES = ('alo-studio-0.1.0.flatpak', 'source.txt', 'features.txt', 'sdk-versions.txt')
REFERENCE_FILES = ('studio-reference.flatpak', 'studio-session', 'source.txt',
                   'reference-features.txt', 'sdk-versions.txt')
OUTPUT_FILES = {'studio.flatpak', 'fixture.flatpak', 'studio-session', 'source.txt',
                'features.txt', 'sdk-versions.txt'}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def manifest(directory, names):
    return ''.join(f'{digest(directory / name)}  {name}\n' for name in names)


def snapshot(directory):
    result = {}
    for path in sorted(directory.rglob('*')):
        name = path.relative_to(directory).as_posix()
        if path.is_symlink():
            result[name] = ('symlink', os.readlink(path))
        elif path.is_file():
            result[name] = ('file', path.read_bytes(), stat.S_IMODE(path.stat().st_mode))
        else:
            result[name] = ('directory',)
    return result


class PrepareInstalledWalk(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='studio-prepare-test-')
        self.addCleanup(self.temporary.cleanup)
        self.root = pathlib.Path(self.temporary.name)
        self.normal = self.root / 'normal'
        self.reference = self.root / 'reference'
        self.output = self.root / 'prepared'
        self.normal.mkdir()
        self.reference.mkdir()
        records = {'source.txt': (SOURCE + '\n').encode(), 'sdk-versions.txt': SDK}
        normal = dict(records, **{'alo-studio-0.1.0.flatpak': b'test-only normal package\n',
                                 'features.txt': NORMAL_FEATURES})
        reference = dict(records, **{'studio-reference.flatpak': b'test-only reference package\n',
                                    'studio-session': b'test-only zipapp bytes\x00\xff',
                                    'reference-features.txt': REFERENCE_FEATURES})
        for directory, files in [(self.normal, normal), (self.reference, reference)]:
            for name, content in files.items():
                (directory / name).write_bytes(content)
        self.rehash()

    def rehash(self):
        (self.normal / 'checksums.txt').write_text(manifest(self.normal, NORMAL_FILES))
        (self.reference / 'reference-checksums.txt').write_text(
            manifest(self.reference, REFERENCE_FILES))

    def refuse(self, *, expected=SOURCE, output=None):
        output = output if output is not None else self.output
        before = snapshot(self.normal), snapshot(self.reference)
        existed = output.exists() or output.is_symlink()
        with self.assertRaises(ValueError):
            prepare(self.normal, self.reference, output, expected)
        self.assertEqual((snapshot(self.normal), snapshot(self.reference)), before)
        if not existed:
            self.assertFalse(output.exists())
            self.assertFalse(output.is_symlink())

    def test_verified_payloads_are_copied_under_canonical_names_with_six_checksums(self):
        before = snapshot(self.normal), snapshot(self.reference)
        prepare(self.normal, self.reference, self.output, SOURCE)
        self.assertEqual({path.name for path in self.output.iterdir()}, OUTPUT_FILES | {'checksums.txt'})
        sources = {'studio.flatpak': self.normal / 'alo-studio-0.1.0.flatpak',
                   'fixture.flatpak': self.reference / 'studio-reference.flatpak',
                   'studio-session': self.reference / 'studio-session',
                   'source.txt': self.normal / 'source.txt',
                   'features.txt': self.normal / 'features.txt',
                   'sdk-versions.txt': self.normal / 'sdk-versions.txt'}
        for name, original in sources.items():
            with self.subTest(name=name):
                self.assertFalse((self.output / name).is_symlink())
                self.assertEqual((self.output / name).read_bytes(), original.read_bytes())
        lines = (self.output / 'checksums.txt').read_text().splitlines()
        self.assertEqual(len(lines), 6)
        seen = set()
        for line in lines:
            match = re.fullmatch(r'([0-9a-f]{64})  ([A-Za-z0-9_.-]+)', line)
            self.assertIsNotNone(match, line)
            checksum, name = match.groups()
            self.assertNotIn(name, seen)
            seen.add(name)
            self.assertEqual(checksum, digest(self.output / name))
        self.assertEqual(seen, OUTPUT_FILES)
        self.assertEqual((snapshot(self.normal), snapshot(self.reference)), before)

    def test_manifest_order_does_not_change_which_original_files_are_verified(self):
        (self.normal / 'checksums.txt').write_text(manifest(self.normal, reversed(NORMAL_FILES)))
        (self.reference / 'reference-checksums.txt').write_text(
            manifest(self.reference, reversed(REFERENCE_FILES)))
        prepare(self.normal, self.reference, self.output, SOURCE)
        self.assertEqual((self.output / 'features.txt').read_bytes(), NORMAL_FEATURES)

    def test_copied_outputs_do_not_alias_the_original_files(self):
        before = snapshot(self.normal), snapshot(self.reference)
        prepare(self.normal, self.reference, self.output, SOURCE)
        for name in OUTPUT_FILES:
            (self.output / name).write_bytes(b'changed only in prepared output')
        self.assertEqual((snapshot(self.normal), snapshot(self.reference)), before)

    def test_every_original_payload_and_record_is_hash_verified(self):
        for directory, names in [(self.normal, NORMAL_FILES), (self.reference, REFERENCE_FILES)]:
            for name in names:
                with self.subTest(directory=directory.name, name=name):
                    path = directory / name
                    original = path.read_bytes()
                    path.write_bytes(original + b'changed after producer hashing')
                    self.refuse()
                    path.write_bytes(original)

    def test_missing_payloads_are_refused_without_partial_output(self):
        for directory, names in [(self.normal, NORMAL_FILES), (self.reference, REFERENCE_FILES)]:
            for name in names:
                with self.subTest(directory=directory.name, name=name):
                    path = directory / name
                    original = path.read_bytes()
                    path.unlink()
                    self.refuse()
                    path.write_bytes(original)

    def test_both_original_manifests_are_required(self):
        for path in [self.normal / 'checksums.txt', self.reference / 'reference-checksums.txt']:
            with self.subTest(manifest=path.name):
                original = path.read_bytes()
                path.unlink()
                self.refuse()
                path.write_bytes(original)

    def test_missing_duplicate_and_extra_manifest_entries_are_refused(self):
        for directory, filename in [(self.normal, 'checksums.txt'),
                                    (self.reference, 'reference-checksums.txt')]:
            path = directory / filename
            original = path.read_text()
            lines = original.splitlines(keepends=True)
            extra = directory / 'extra.txt'
            extra.write_bytes(b'not part of the producer inventory')
            cases = ['', ''.join(lines[1:]), original + lines[0],
                     original + f'{digest(extra)}  extra.txt\n']
            for changed in cases:
                with self.subTest(manifest=filename, contents=changed):
                    path.write_text(changed)
                    self.refuse()
            path.write_text(original)
            extra.unlink()

    def test_manifest_paths_must_be_simple_exact_inventory_basenames(self):
        for directory, filename in [(self.normal, 'checksums.txt'),
                                    (self.reference, 'reference-checksums.txt')]:
            path = directory / filename
            original = path.read_text()
            for name in ['./source.txt', '../source.txt', 'sub/source.txt',
                         '/tmp/source.txt', r'C:\source.txt', r'..\source.txt',
                         'Source.txt', 'source.txt/..', 'source.txt extra']:
                with self.subTest(manifest=filename, name=name):
                    path.write_text(original.replace('  source.txt\n', f'  {name}\n'))
                    self.refuse()
            path.write_text(original)

    def test_only_canonical_lowercase_sha256sum_lines_are_accepted(self):
        for directory, filename in [(self.normal, 'checksums.txt'),
                                    (self.reference, 'reference-checksums.txt')]:
            path = directory / filename
            original = path.read_text()
            checksum, name = original.splitlines()[0].split('  ')
            malformed = [f'{checksum.upper()}  {name}\n',
                         f'{checksum[:-1]}  {name}\n', f'{checksum}0  {name}\n',
                         f'{"g" * 64}  {name}\n', f'{checksum} {name}\n',
                         f'{checksum}\t{name}\n', f'{checksum} *{name}\n',
                         f'{checksum}   {name}\n', f'{checksum}  \n']
            remaining = ''.join(original.splitlines(keepends=True)[1:])
            for line in malformed:
                with self.subTest(manifest=filename, line=line):
                    path.write_text(line + remaining)
                    self.refuse()
            for changed in [original.rstrip('\n'), original + '\n']:
                with self.subTest(manifest=filename, noncanonical_ending=changed):
                    path.write_text(changed)
                    self.refuse()
            path.write_text(original)

    def test_a_well_formed_wrong_hash_is_refused(self):
        for directory, filename in [(self.normal, 'checksums.txt'),
                                    (self.reference, 'reference-checksums.txt')]:
            path = directory / filename
            original = path.read_text()
            path.write_text('0' * 64 + original[64:])
            self.refuse()
            path.write_text(original)

    def test_both_sources_must_match_the_requested_revision_even_with_valid_hashes(self):
        for directories in [(self.normal,), (self.reference,), (self.normal, self.reference)]:
            with self.subTest(directories=[path.name for path in directories]):
                for directory in directories:
                    (directory / 'source.txt').write_text('f' * 40 + '\n')
                self.rehash()
                self.refuse()
                for directory in directories:
                    (directory / 'source.txt').write_text(SOURCE + '\n')
                self.rehash()

    def test_expected_source_is_an_exact_lowercase_full_git_sha(self):
        for expected in [None, 123, '', 'a' * 39, 'a' * 41, 'A' * 40,
                         'g' * 40, SOURCE + '\n', ' ' + SOURCE]:
            with self.subTest(expected=expected):
                self.refuse(expected=expected)

    def test_source_records_cannot_claim_two_revisions_or_non_sha_identity(self):
        for directory in [self.normal, self.reference]:
            for source in ['', 'main\n', SOURCE.upper() + '\n', SOURCE + '\n' + 'f' * 40 + '\n']:
                with self.subTest(directory=directory.name, source=source):
                    (directory / 'source.txt').write_text(source)
                    self.rehash()
                    self.refuse()
            (directory / 'source.txt').write_text(SOURCE + '\n')
            self.rehash()

    def test_normal_and_reference_feature_records_are_exact_and_not_interchangeable(self):
        for directory, filename, valid, other in [
                (self.normal, 'features.txt', NORMAL_FEATURES, REFERENCE_FEATURES),
                (self.reference, 'reference-features.txt', REFERENCE_FEATURES, NORMAL_FEATURES)]:
            for changed in [other, b'', valid.rstrip(b'\n'), valid + b'\n',
                            valid.replace(b'\n', b'\r\n'), b' ' + valid,
                            valid + b'additional feature\n']:
                with self.subTest(directory=directory.name, content=changed):
                    (directory / filename).write_bytes(changed)
                    self.rehash()
                    self.refuse()
            (directory / filename).write_bytes(valid)
            self.rehash()

    def test_runtime_or_sdk_disagreement_is_refused_even_when_both_manifests_verify(self):
        for field in [b'org.gnome.Platform', b'org.gnome.Sdk']:
            with self.subTest(field=field):
                changed = SDK.replace(field + b'/x86_64/49', field + b'/x86_64/50')
                (self.reference / 'sdk-versions.txt').write_bytes(changed)
                self.rehash()
                self.refuse()
        (self.reference / 'sdk-versions.txt').write_bytes(SDK)
        self.rehash()

    def test_payload_and_record_symlinks_are_refused_despite_matching_hashes(self):
        for directory, names in [(self.normal, NORMAL_FILES), (self.reference, REFERENCE_FILES)]:
            for name in names:
                with self.subTest(directory=directory.name, name=name):
                    path = directory / name
                    original = path.read_bytes()
                    outside = self.root / 'outside-payload'
                    outside.write_bytes(original)
                    path.unlink()
                    path.symlink_to(outside)
                    self.refuse()
                    self.assertEqual(outside.read_bytes(), original)
                    path.unlink()
                    path.write_bytes(original)

    def test_symlink_manifests_are_not_trusted_as_original_producer_records(self):
        for path in [self.normal / 'checksums.txt', self.reference / 'reference-checksums.txt']:
            with self.subTest(manifest=path.name):
                original = path.read_bytes()
                outside = self.root / 'outside-manifest'
                outside.write_bytes(original)
                path.unlink()
                path.symlink_to(outside)
                self.refuse()
                self.assertEqual(outside.read_bytes(), original)
                path.unlink()
                path.write_bytes(original)

    def test_a_manifest_entry_must_be_a_regular_file_not_a_directory(self):
        for directory, names in [(self.normal, NORMAL_FILES), (self.reference, REFERENCE_FILES)]:
            for name in names:
                with self.subTest(directory=directory.name, name=name):
                    path = directory / name
                    original = path.read_bytes()
                    path.unlink()
                    path.mkdir()
                    self.refuse()
                    path.rmdir()
                    path.write_bytes(original)

    def test_preexisting_output_directory_and_file_are_preserved(self):
        self.output.mkdir()
        sentinel = self.output / 'belongs-to-someone-else'
        sentinel.write_bytes(b'preserve this')
        before = snapshot(self.output)
        self.refuse()
        self.assertEqual(snapshot(self.output), before)
        sentinel.unlink()
        self.output.rmdir()
        self.output.write_bytes(b'existing output file')
        self.refuse()
        self.assertEqual(self.output.read_bytes(), b'existing output file')

    def test_a_preexisting_empty_output_is_still_refused(self):
        self.output.mkdir()
        self.refuse()
        self.assertEqual(list(self.output.iterdir()), [])

    def test_existing_and_dangling_output_symlinks_are_refused(self):
        for exists in [False, True]:
            with self.subTest(target_exists=exists):
                target = self.root / 'unrelated-output'
                if exists:
                    target.mkdir()
                self.output.symlink_to(target, target_is_directory=True)
                self.refuse()
                self.assertTrue(self.output.is_symlink())
                if exists:
                    self.assertEqual(list(target.iterdir()), [])
                else:
                    self.assertFalse(target.exists())
                self.output.unlink()

    def test_preparation_does_not_write_inside_either_original_artifact(self):
        for directory in [self.normal, self.reference]:
            with self.subTest(directory=directory.name):
                self.refuse(output=directory / 'prepared')


if __name__ == '__main__':
    unittest.main()
