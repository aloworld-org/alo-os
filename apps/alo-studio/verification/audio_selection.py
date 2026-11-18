"""Fresh-profile audio preflight and source-only restoration for the private VM."""

import configparser


def require_automatic_source(nodes, *, state=''):
    """Refuse an existing source preference or history before the fixture runs."""
    if not automatic_source_ready(nodes, state=state):
        raise ValueError('the default metadata is not ready')


def automatic_source_ready(nodes, *, state=''):
    """Validate preferences; return False only when default metadata is absent.

    pw-dump omits empty metadata stores. An owned fixture may populate one,
    but it must be observed and validated before selecting a microphone.
    Persisted history is checked even when the live store is not listed.
    """
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
    if not isinstance(nodes, list) or any(not isinstance(node, dict) for node in nodes):
        raise ValueError('the actual PipeWire metadata inventory is missing')
    stores = [node for node in nodes if node.get('type') == 'PipeWire:Interface:Metadata']
    for store in stores:
        props = store.get('props')
        if (not isinstance(props, dict) or not isinstance(props.get('metadata.name'), str)
                or not props['metadata.name']):
            raise ValueError('the actual PipeWire metadata properties are malformed')
    defaults = [store for store in stores if store['props']['metadata.name'] == 'default']
    if not defaults:
        return False
    if len(defaults) != 1 or not isinstance(defaults[0].get('metadata'), list):
        raise ValueError('the default metadata is malformed or ambiguous')
    for entry in defaults[0]['metadata']:
        if not isinstance(entry, dict) or not isinstance(entry.get('key'), str):
            raise ValueError('the default metadata is malformed')
        if entry['key'].startswith('default.configured.audio.source'):
            raise ValueError('use a fresh profile with no configured microphone preference')
    return True


def clear_reference_source(command):
    """Restore automatic source selection without changing sink/video choices.

    WirePlumber's documented settings ID 1 clears Audio/Source only:
    https://pipewire.pages.freedesktop.org/wireplumber/man/wpctl.html
    """
    command('wpctl', 'clear-default', '1')
