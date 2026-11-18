"""Fresh-profile audio preflight and source-only restoration for the private VM."""

import configparser


def require_automatic_source(nodes, *, state=''):
    """Refuse an existing source preference or history before the fixture runs."""
    if not isinstance(nodes, list) or any(not isinstance(node, dict) for node in nodes):
        raise ValueError('the actual PipeWire metadata inventory is missing')
    defaults = [node for node in nodes if node.get('type') == 'PipeWire:Interface:Metadata'
                and isinstance(node.get('props'), dict)
                and node['props'].get('metadata.name') == 'default']
    if len(defaults) != 1 or not isinstance(defaults[0].get('metadata'), list):
        raise ValueError('the default metadata is not ready or is ambiguous')
    for entry in defaults[0]['metadata']:
        if not isinstance(entry, dict) or not isinstance(entry.get('key'), str):
            raise ValueError('the default metadata is malformed')
        if entry['key'].startswith('default.configured.audio.source'):
            raise ValueError('use a fresh profile with no configured microphone preference')
    if not isinstance(state, str):
        raise ValueError('the observed audio preference state is invalid')
    parser = configparser.RawConfigParser()
    try:
        parser.read_string(state)
    except configparser.Error as error:
        raise ValueError('the persisted audio preference state is malformed') from error
    for section in parser.sections():
        if any(key.startswith('default.configured.audio.source') for key in parser[section]):
            raise ValueError('use a fresh profile with no previous microphone preferences')


def clear_reference_source(command):
    """Restore automatic source selection without changing sink/video choices.

    WirePlumber's documented settings ID 1 clears Audio/Source only:
    https://pipewire.pages.freedesktop.org/wireplumber/man/wpctl.html
    """
    command('wpctl', 'clear-default', '1')
