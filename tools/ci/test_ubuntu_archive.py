"""Independent APT source rewrites using strings and temporary directories only.

rewrite_sources(text) changes HTTP(S) Ubuntu archive URI tokens and the runner's
exact mirror+file:/etc/apt/apt-mirrors.txt token to the central HTTPS archive.
configure(apt_root) processes the
root sources.list and immediate sources.list.d/*.list and *.sources files,
writes only changed files and returns the number of changed files. No test
contacts an archive, changes host APT state or invokes a package manager.
"""

import os
from pathlib import Path
import tempfile
import unittest

from ubuntu_archive import configure, rewrite_sources


CENTRAL = 'https://archive.ubuntu.com/ubuntu'
RUNNER_MIRROR = 'mirror+file:/etc/apt/apt-mirrors.txt'


class SourceText(unittest.TestCase):
    def test_exact_runner_mirror_list_reaches_central_in_each_source_syntax(self):
        for before, expected in (
            (f'deb [arch=amd64 signed-by=/usr/share/keyrings/ubuntu.gpg] {RUNNER_MIRROR} noble main\n',
             f'deb [arch=amd64 signed-by=/usr/share/keyrings/ubuntu.gpg] {CENTRAL} noble main\n'),
            (f'Types: deb deb-src\nURIs: {RUNNER_MIRROR}\nSuites: noble noble-updates\n'
             'Components: main restricted\nSigned-By: /usr/share/keyrings/ubuntu.gpg\n',
             f'Types: deb deb-src\nURIs: {CENTRAL}\nSuites: noble noble-updates\n'
             'Components: main restricted\nSigned-By: /usr/share/keyrings/ubuntu.gpg\n'),
            (f'Types:deb\nURIs:{RUNNER_MIRROR}\nSuites:noble\nComponents:main\n',
             f'Types:deb\nURIs:{CENTRAL}\nSuites:noble\nComponents:main\n'),
            (f'URIs: https://example.org/repository\n {RUNNER_MIRROR}\nSuites: noble\n',
             f'URIs: https://example.org/repository\n {CENTRAL}\nSuites: noble\n'),
        ):
            with self.subTest(source=before):
                self.assertEqual(rewrite_sources(before), expected)
                self.assertEqual(rewrite_sources(expected), expected)

    def test_runner_mirror_is_exact_and_only_active_source_values_change(self):
        for uri in (
            'file:/etc/apt/apt-mirrors.txt',
            'mirror+file:/etc/apt/apt-security-mirrors.txt',
            'mirror+file:/etc/apt/other-mirrors.txt',
            'mirror+file:///etc/apt/apt-mirrors.txt',
            'mirror+file:/etc/apt/apt-mirrors.txt/',
            'mirror+file:/etc/apt/apt-mirrors.txt.evil',
            'mirror+file:/etc/apt/apt-mirrors.txt?other=1',
            'mirror+file:/etc/apt/apt-mirrors.txt#other',
            'mirror+https://example.org/etc/apt/apt-mirrors.txt',
            'https://example.org/mirror+file:/etc/apt/apt-mirrors.txt',
        ):
            with self.subTest(uri=uri):
                for text in (f'deb {uri} noble main\n', f'URIs:{uri}\n'):
                    self.assertEqual(rewrite_sources(text), text)
        text = (f'# deb {RUNNER_MIRROR} noble main\n'
                f'# URIs:{RUNNER_MIRROR}\nDescription: {RUNNER_MIRROR}\n'
                f'Signed-By: {RUNNER_MIRROR}\n'
                'URIs: http://security.ubuntu.com/ubuntu/\nSuites: noble-security\n')
        self.assertEqual(rewrite_sources(text), text)
        self.assertEqual(rewrite_sources(f'deb {RUNNER_MIRROR} noble main # {RUNNER_MIRROR}\n'),
                         f'deb {CENTRAL} noble main # {RUNNER_MIRROR}\n')

    def test_http_https_country_and_ec2_archive_uris_reach_the_canonical_archive(self):
        for uri in (
            'http://archive.ubuntu.com/ubuntu',
            'http://archive.ubuntu.com/ubuntu/',
            'http://gb.archive.ubuntu.com/ubuntu',
            'https://fr.archive.ubuntu.com/ubuntu/',
            'http://us-east-2.ec2.archive.ubuntu.com/ubuntu',
            'https://eu-central-1.ec2.archive.ubuntu.com/ubuntu/',
            'http://ap-southeast-2.ec2.archive.ubuntu.com/ubuntu',
        ):
            with self.subTest(uri=uri):
                self.assertEqual(rewrite_sources(f'URIs: {uri}\n'), f'URIs: {CENTRAL}\n')
                self.assertEqual(rewrite_sources(f'deb {uri} noble main\n'),
                                 f'deb {CENTRAL} noble main\n')

    def test_deb822_rewrites_each_archive_uri_preserving_all_other_fields(self):
        before = (
            '# Ubuntu primary source\n'
            'Types: deb deb-src\n'
            'URIs: http://us-east-2.ec2.archive.ubuntu.com/ubuntu/\n'
            'Suites: noble noble-updates noble-backports\n'
            'Components: main restricted universe multiverse\n'
            'Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg\n'
            'Enabled: yes\n\n'
            '# Keep security on its independent endpoint\n'
            'Types: deb\n'
            'URIs: http://security.ubuntu.com/ubuntu/\n'
            'Suites: noble-security\n'
            'Components: main restricted universe multiverse\n'
            'Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg\n'
        )
        expected = before.replace('http://us-east-2.ec2.archive.ubuntu.com/ubuntu/', CENTRAL)
        self.assertEqual(rewrite_sources(before), expected)

    def test_deb822_multiple_and_continued_uris_preserve_spacing_and_foreign_repositories(self):
        before = (
            'URIs:\thttp://gb.archive.ubuntu.com/ubuntu/  https://example.org/repository\n'
            ' https://eu-west-1.ec2.archive.ubuntu.com/ubuntu\n'
            'Suites: noble\nComponents: main\n'
        )
        expected = (
            f'URIs:\t{CENTRAL}  https://example.org/repository\n'
            f' {CENTRAL}\n'
            'Suites: noble\nComponents: main\n'
        )
        self.assertEqual(rewrite_sources(before), expected)

    def test_deb822_uri_field_without_space_after_colon_remains_a_uri_field(self):
        # apt_pkg.TagSection accepts this field syntax; the label is not part
        # of its first URI token. No package-manager dependency is needed here.
        for value in ('http://gb.archive.ubuntu.com/ubuntu',
                      'https://us-east-2.ec2.archive.ubuntu.com/ubuntu/'):
            with self.subTest(uri=value):
                before = f'Types:deb\nURIs:{value}\nSuites:noble\nComponents:main\n'
                expected = f'Types:deb\nURIs:{CENTRAL}\nSuites:noble\nComponents:main\n'
                self.assertEqual(rewrite_sources(before), expected)

    def test_legacy_types_options_tabs_and_inline_comments_survive(self):
        before = (
            'deb [arch=amd64 signed-by=/usr/share/keyrings/ubuntu.gpg] '
            'http://us-east-2.ec2.archive.ubuntu.com/ubuntu/ noble main restricted '
            '# http://gb.archive.ubuntu.com/ubuntu is commentary\n'
            '  deb-src\t[arch=arm64]\thttps://fr.archive.ubuntu.com/ubuntu\t'
            'noble-updates universe\n'
        )
        expected = (
            'deb [arch=amd64 signed-by=/usr/share/keyrings/ubuntu.gpg] '
            f'{CENTRAL} noble main restricted '
            '# http://gb.archive.ubuntu.com/ubuntu is commentary\n'
            f'  deb-src\t[arch=arm64]\t{CENTRAL}\tnoble-updates universe\n'
        )
        self.assertEqual(rewrite_sources(before), expected)

    def test_comments_other_deb822_fields_and_security_are_unchanged(self):
        text = (
            '# URIs: http://gb.archive.ubuntu.com/ubuntu\n'
            '  # deb http://gb.archive.ubuntu.com/ubuntu noble main\n'
            'Description: URIs: http://gb.archive.ubuntu.com/ubuntu\n'
            'Signed-By: https://archive.ubuntu.com/ubuntu\n'
            'URIs: http://security.ubuntu.com/ubuntu\n'
            'Suites: noble-security\n\n'
            'deb https://security.ubuntu.com/ubuntu/ noble-security main\n'
            'deb https://ppa.launchpadcontent.net/example/tool/ubuntu noble main\n'
        )
        self.assertEqual(rewrite_sources(text), text)

    def test_similar_hosts_embedded_urls_and_non_archive_paths_are_not_matches(self):
        for uri in (
            'http://archive.ubuntu.com.evil.example/ubuntu',
            'http://gb.archive.ubuntu.com.evil.example/ubuntu',
            'http://us-east-2.ec2.archive.ubuntu.com.evil.example/ubuntu',
            'http://notarchive.ubuntu.com/ubuntu',
            'http://evil.example/archive.ubuntu.com/ubuntu',
            'http://evil.example/http://archive.ubuntu.com/ubuntu',
            'http://archive.ubuntu.com@evil.example/ubuntu',
            'http://archive.ubuntu.com/ubuntu-extra',
            'http://archive.ubuntu.com/ubuntu/pool',
            'http://archive.ubuntu.com/ubuntu?mirror=other',
        ):
            with self.subTest(uri=uri):
                for text in (f'URIs: {uri}\n', f'deb {uri} noble main\n'):
                    self.assertEqual(rewrite_sources(text), text)

    def test_central_https_is_idempotent_and_newline_form_is_preserved(self):
        central = f'URIs: {CENTRAL}\nSuites: noble\n'
        self.assertEqual(rewrite_sources(central), central)
        self.assertEqual(rewrite_sources(rewrite_sources(central)), central)
        before = 'Types: deb\r\nURIs: http://gb.archive.ubuntu.com/ubuntu/\r\nSuites: noble'
        expected = f'Types: deb\r\nURIs: {CENTRAL}\r\nSuites: noble'
        self.assertEqual(rewrite_sources(before), expected)
        self.assertEqual(rewrite_sources(rewrite_sources(before)), expected)


class ConfigureSources(unittest.TestCase):
    def test_runner_mirror_sources_change_without_reading_or_rewriting_mirror_lists(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            sources = root / 'sources.list.d'
            sources.mkdir()
            before = {
                root / 'sources.list': f'deb {RUNNER_MIRROR} noble main\n',
                sources / 'ubuntu.sources': f'URIs: {RUNNER_MIRROR}\r\nSuites: noble\r\n',
                sources / 'compact.sources': f'URIs:{RUNNER_MIRROR}\nSuites:noble\n',
            }
            unchanged = {
                root / 'apt-mirrors.txt': b'http://us-east-2.ec2.archive.ubuntu.com/ubuntu\n',
                sources / 'security.sources': b'URIs: mirror+file:/etc/apt/apt-security-mirrors.txt\n',
                sources / 'other.list': b'deb file:/etc/apt/apt-mirrors.txt noble main\n',
                sources / 'foreign.sources': b'URIs: mirror+https://example.org/mirrors.txt\n',
            }
            for path, data in before.items():
                path.write_bytes(data.encode())
            for path, data in unchanged.items():
                path.write_bytes(data)
                os.utime(path, ns=(1_000_000_000, 1_000_000_000))
            metadata = {path: (path.stat().st_ino, path.stat().st_mtime_ns) for path in unchanged}
            self.assertEqual(configure(root), 3)
            for path, data in before.items():
                self.assertEqual(path.read_bytes(), data.replace(RUNNER_MIRROR, CENTRAL).encode())
            for path, data in unchanged.items():
                self.assertEqual(path.read_bytes(), data)
                self.assertEqual((path.stat().st_ino, path.stat().st_mtime_ns), metadata[path])
            after = {path: (path.stat().st_ino, path.stat().st_mtime_ns) for path in before}
            self.assertEqual(configure(root), 0)
            self.assertEqual({path: (path.stat().st_ino, path.stat().st_mtime_ns) for path in before}, after)

    def test_only_root_list_and_immediate_source_files_change_and_count_once_each(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            sources = root / 'sources.list.d'
            sources.mkdir()
            before = {
                root / 'sources.list': b'deb http://gb.archive.ubuntu.com/ubuntu noble main\n',
                sources / 'ubuntu.sources': (
                    b'Types: deb\r\nURIs: http://us-east-2.ec2.archive.ubuntu.com/ubuntu/\r\n'
                    b'Suites: noble noble-updates\r\nComponents: main\r\n'
                    b'Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg\r\n'),
                sources / 'extra.list': b'deb-src http://fr.archive.ubuntu.com/ubuntu/ noble main\n',
            }
            unchanged = {
                sources / 'security.sources': b'URIs: http://security.ubuntu.com/ubuntu/\n',
                sources / 'central.list': b'deb https://archive.ubuntu.com/ubuntu noble main\n',
                sources / 'foreign.sources': b'URIs: https://example.org/repository\n',
                sources / 'disabled.list.save': b'deb http://gb.archive.ubuntu.com/ubuntu noble main\n',
                root / 'arbitrary.list': b'deb http://gb.archive.ubuntu.com/ubuntu noble main\n',
                sources / 'nested' / 'old.list': b'deb http://gb.archive.ubuntu.com/ubuntu noble main\n',
            }
            for path, data in {**before, **unchanged}.items():
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
                os.utime(path, ns=(1_000_000_000, 1_000_000_000))
            metadata = {path: (path.stat().st_ino, path.stat().st_mtime_ns) for path in unchanged}
            self.assertEqual(configure(root), 3)
            self.assertEqual((root / 'sources.list').read_bytes(),
                             b'deb https://archive.ubuntu.com/ubuntu noble main\n')
            self.assertEqual((sources / 'extra.list').read_bytes(),
                             b'deb-src https://archive.ubuntu.com/ubuntu noble main\n')
            self.assertEqual((sources / 'ubuntu.sources').read_bytes(),
                b'Types: deb\r\nURIs: https://archive.ubuntu.com/ubuntu\r\n'
                b'Suites: noble noble-updates\r\nComponents: main\r\n'
                b'Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg\r\n')
            for path, data in unchanged.items():
                self.assertEqual(path.read_bytes(), data)
                self.assertEqual((path.stat().st_ino, path.stat().st_mtime_ns), metadata[path],
                                 'unchanged files must not be rewritten')
            after = {path: (path.stat().st_ino, path.stat().st_mtime_ns) for path in before}
            self.assertEqual(configure(root), 0)
            self.assertEqual({path: (path.stat().st_ino, path.stat().st_mtime_ns) for path in before}, after)

    def test_absent_optional_files_create_nothing_and_return_zero(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            self.assertEqual(configure(root), 0)
            self.assertEqual(list(root.iterdir()), [])


if __name__ == '__main__':
    unittest.main()
