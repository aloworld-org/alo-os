"""Person-side native permission choices for the disposable Studio walk."""

BASE = ('alo-app', 'alo', '--in', 'Settings')
RUNNING = 'Running code'
ACCESS = 'What alo can use'
ASK = 'After showing you the command and waiting for you'
FULL = 'Straight away, without asking'
SEALED = 'In a sealed box that sees only this project'
STUDIO = 'alo Studio'
BACK = 'Take back'
TOKENS = ('ask-before-grant', 'granted', 'revoked', 'full', 'ask-again')
EMPTY = ('Nothing is granted right now. No agent and no application has been '
         'granted anything on this machine, and there is nothing here to revoke')
CONFIRMED = ('alo can now use alo Studio. It is in the list below, and you can '
             'take it back there')


class NativeSettings:
    """Change only a verified fresh test profile through its visible controls.

    Each action is sent once, followed by fresh readback. No files or agent
    approvals are written. Failed readback refuses the boundary; the caller
    must not acknowledge it to the agent. Cleanup handles our grant and policy
    independently and refuses ambiguous controls or a different displayed grant.
    The disposable profile must be exclusive: native read/controls cannot
    distinguish an external replacement folder bearing the same display name.
    """

    def __init__(self, command):
        self.command = command
        self.prepared = False
        self.stage = 0
        self.owns_grant = False
        self.owns_policy = False

    def rows(self):
        rows = []
        for line in self.command(*BASE, 'controls').splitlines():
            row = line.split('\t')
            if len(row) != 3:
                raise ValueError('Settings returned a malformed control')
            rows.append(row)
        return rows

    def control(self, role, name, rows=None):
        matches = [row for row in self.rows() if row[1] == name] if rows is None else [
            row for row in rows if row[1] == name]
        if len(matches) != 1 or matches[0][0] != role:
            raise ValueError(f'Settings must offer exactly one {role}: {name}')
        flags = matches[0][2].split(',') if matches[0][2] else []
        allowed = {'tab': {'selected'}, 'radio_button': {'on', 'off'}, 'button': set()}[role]
        if len(flags) != len(set(flags)) or not set(flags) <= allowed:
            raise ValueError(f'Settings control is unavailable or malformed: {name}')
        if role == 'radio_button' and len(flags) != 1:
            raise ValueError(f'Settings choice has no unambiguous state: {name}')
        return flags

    def press(self, role, name):
        self.control(role, name)
        self.command(*BASE, 'press', role, name)

    def page(self, name):
        self.press('tab', name)
        if self.control('tab', name) != ['selected']:
            raise ValueError(f'Settings did not select the requested page: {name}')

    def policy(self):
        self.page(RUNNING)
        rows = self.rows()
        selected = [name for name in (SEALED, ASK, FULL)
                    if self.control('radio_button', name, rows) == ['on']]
        if len(selected) != 1:
            raise ValueError('Settings has no unambiguous running-code policy')
        return selected[0]

    def set_policy(self, desired):
        current = self.policy()
        if current not in (ASK, FULL):
            raise ValueError('The private walk does not own this policy change')
        if current != desired:
            # Retain ownership before sending: a failing response can follow a
            # completed native action, so cleanup must inspect the real state.
            self.owns_policy = True
            self.press('radio_button', desired)
        if self.policy() != desired:
            raise ValueError('The running-code choice was not confirmed')

    def empty(self):
        self.page(ACCESS)
        rows = self.rows()
        text = self.command(*BASE, 'read')
        if any(row[1] == BACK for row in rows) or EMPTY not in text:
            raise ValueError('The private profile must have no grants or paired computers')

    def own_grant(self):
        self.page(ACCESS)
        self.control('button', BACK)
        lines = [line.strip() for line in self.command(*BASE, 'read').splitlines()]
        if (not self.owns_grant or lines.count(STUDIO) != 1
                or 'Computers paired with this one' in lines):
            raise ValueError('The private Studio grant is absent or ambiguous')

    def prepare(self):
        if self.prepared:
            raise ValueError('The native permission walk was already prepared')
        if self.policy() != FULL:
            raise ValueError('The fresh private walk must start with full trust')
        self.empty()
        self.prepared = True

    def boundary(self, token):
        if not self.prepared or self.stage >= len(TOKENS) or token != TOKENS[self.stage]:
            raise ValueError('The native permission boundary is out of order')
        if token in ('ask-before-grant', 'ask-again', 'full'):
            self.empty()
            self.set_policy(FULL if token == 'full' else ASK)
            self.empty()
        elif token == 'granted':
            self.empty()
            self.press('button', 'Give alo an application')
            self.press('button', STUDIO)
            self.control('button', 'For an hour')
            self.owns_grant = True
            self.command(*BASE, 'press', 'button', 'For an hour')
            self.own_grant()
            if CONFIRMED not in self.command(*BASE, 'read'):
                raise ValueError('The native Studio grant was not confirmed')
        else:
            self.revoke()
        self.stage += 1

    def revoke(self):
        self.own_grant()
        self.press('button', BACK)
        self.empty()
        self.owns_grant = False

    def restore(self):
        failure = None
        if self.owns_grant:
            try:
                self.revoke()
            except BaseException as error:
                failure = error
        if self.owns_policy:
            try:
                self.set_policy(FULL)
                self.owns_policy = False
            except BaseException as error:
                if failure is None:
                    failure = error
                else:
                    failure.add_note(f'Additional native policy cleanup failure: {error}')
        if failure is not None:
            raise failure
