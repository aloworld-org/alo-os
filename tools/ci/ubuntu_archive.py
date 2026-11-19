"""Use Ubuntu's central HTTPS archive on disposable verification runners.

Only archive URI tokens change. Suites, package signature configuration,
security endpoints and unrelated repositories retain their original bytes.
"""

from pathlib import Path
import re
from urllib.parse import urlsplit


def archive_uri(match):
    """Normalize an exact Ubuntu archive URI, leaving every other token alone."""
    token = match.group()
    try:
        uri = urlsplit(token)
    except ValueError:
        return token
    if (uri.scheme in ('http', 'https')
            and re.fullmatch(r'(?:[a-z0-9-]+\.)*archive\.ubuntu\.com', uri.netloc)
            and uri.path in ('/ubuntu', '/ubuntu/') and not uri.query and not uri.fragment):
        return 'https://archive.ubuntu.com/ubuntu'
    return token


def rewrite_sources(text):
    """Rewrite only active legacy source lines or deb822 URI fields."""
    output = []
    uris = False
    for line in text.splitlines(keepends=True):
        if line.lstrip().startswith('#'):
            output.append(line)
            continue
        if not line.strip():
            uris = False
        elif not line[0].isspace():
            uris = bool(re.match(r'(?i)URIs:', line))
        active = uris or bool(re.match(r'\s*deb(?:-src)?\s', line))
        if active:
            comment = re.search(r'(?<!\S)#', line)
            boundary = comment.start() if comment else len(line)
            # deb822 permits a value immediately after its field's colon.
            start = len('URIs:') if re.match(r'(?i)URIs:', line) else 0
            line = (line[:start] + re.sub(r'\S+', archive_uri, line[start:boundary])
                    + line[boundary:])
        output.append(line)
    return ''.join(output)


def configure(apt_root):
    """Update the runner's source files and return the number actually changed."""
    root = Path(apt_root)
    sources = root / 'sources.list.d'
    paths = [root / 'sources.list', *sorted(sources.glob('*.list')),
             *sorted(sources.glob('*.sources'))]
    changed = 0
    for path in paths:
        if not path.is_file():
            continue
        before = path.read_bytes()
        after = rewrite_sources(before.decode('utf-8')).encode('utf-8')
        if after != before:
            path.write_bytes(after)
            changed += 1
    return changed


if __name__ == '__main__':
    print(f'Ubuntu archive source files updated: {configure(Path("/etc/apt"))}')
