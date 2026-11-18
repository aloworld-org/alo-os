"""Strict process-owned native UI selection for the private installed walk."""


def activate(tree, process, matches):
    """Activate exactly one usable named control, refusing ambiguous trees.

    tree is a real AT-SPI connection in the installed walk. Unit fixtures supply
    observed node facts through the same small interface. No coordinates,
    keyboard shortcuts or direct portal consent replies are manufactured.
    """
    if type(process) is not int or process <= 0:
        raise ValueError('the native dialog owner is unavailable')
    pending = list(tree.roots())
    seen = set()
    candidates = []
    while pending:
        node = pending.pop()
        if node in seen:
            continue
        seen.add(node)
        if len(seen) > 512:
            raise ValueError('the native dialog tree exceeded the walk limit')
        if tree.owner(node[0]) != process:
            continue
        facts = tree.facts(node)
        pending.extend(facts['children'])
        if not {8, 24, 25}.issubset(facts['states']) or not matches(facts['name']):
            continue
        actions = [index for index, name in enumerate(facts['actions'])
                   if name.casefold() in {'click', 'press', 'activate', 'toggle', 'select'}]
        if actions:
            candidates.append((node, actions[0]))
    if len(candidates) != 1:
        raise ValueError(f'expected one native control, found {len(candidates)}')
    node, index = candidates[0]
    if tree.owner(node[0]) != process:
        raise ValueError('the native dialog owner changed')
    current = tree.facts(node)
    if (not {8, 24, 25}.issubset(current['states']) or not matches(current['name'])
            or index >= len(current['actions'])
            or current['actions'][index].casefold() not in {'click', 'press', 'activate', 'toggle', 'select'}):
        raise ValueError('the native control changed before activation')
    if tree.action(node, index) is not True:
        raise ValueError('the native control refused activation')
