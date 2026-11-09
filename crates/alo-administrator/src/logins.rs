//! **Who a number is, and whether they are an administrator**, read from the
//! machine's own lists.
//!
//! Read as text, so a test hands in the text and a program hands in the file.
//! Only `/etc/passwd` and `/etc/group` are read: the group a person is made an
//! administrator in is written there by `systemd-sysusers` from the image's
//! `m alo wheel` line (ADR 0150), and the copy of `wheel` the base keeps in
//! `/usr/lib/group` has no members to read.

/// The machine's list of logins.
pub const THE_LOGINS: &str = "/etc/passwd";

/// The machine's list of groups.
pub const THE_GROUPS: &str = "/etc/group";

/// The group whose members polkit treats as administrators on the pinned base
/// (`/usr/share/polkit-1/rules.d/50-default.rules`: `unix-group:wheel`), and
/// that `sudo` asks a password of.
pub const THE_ADMINISTRATORS: &str = "wheel";

/// The agent's own login (ADR 0001 §5), which is never an administrator and is
/// never checked for, whatever any list says.
pub const THE_AGENTS_LOGIN: &str = "alo-agent";

/// The first number an ordinary person's login has on any Linux.
pub const THE_FIRST_PERSON: u32 = 1000;

/// One past the last number an ordinary person's login has: Fedora's
/// `UID_MAX`, and below every number this image gives its own services
/// (`image/usr/lib/sysusers.d/alo.conf` puts them at 60989 and up).
pub const PAST_THE_LAST_PERSON: u32 = 60_000;

/// The login named for `number` in `logins`, a `/etc/passwd`.
#[must_use]
pub fn the_name_of(logins: &str, number: u32) -> Option<&str> {
    logins.lines().find_map(|line| {
        let mut fields = line.split(':');
        let name = fields.next()?;
        let uid = fields.nth(1)?.parse::<u32>().ok()?;
        (uid == number && !name.is_empty()).then_some(name)
    })
}

/// **The person `number` is**, by login name — or [`None`] for a number that
/// is not an ordinary person's: root, a service, the agent, or nobody.
///
/// Every program here asks this of whoever is on the other end of its socket,
/// because each acts *for a person* and none of them for anything else.
#[must_use]
pub fn a_person(logins: &str, number: u32) -> Option<&str> {
    if !(THE_FIRST_PERSON..PAST_THE_LAST_PERSON).contains(&number) {
        return None;
    }
    the_name_of(logins, number).filter(|name| *name != THE_AGENTS_LOGIN)
}

/// Whether `name` is listed as a member of [`THE_ADMINISTRATORS`] in
/// `groups`, a `/etc/group`.
#[must_use]
pub fn is_an_administrator(groups: &str, name: &str) -> bool {
    if name == THE_AGENTS_LOGIN {
        return false;
    }
    groups.lines().any(|line| {
        let mut fields = line.split(':');
        fields.next() == Some(THE_ADMINISTRATORS)
            && fields
                .nth(2)
                .is_some_and(|members| members.split(',').any(|member| member.trim() == name))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The lists as the image writes them, measured on 2026-11-09 in
    /// `localhost/alo-os:dev` after `systemd-sysusers` read `m alo wheel`.
    const LOGINS: &str = "root:x:0:0:Super User:/root:/bin/bash\n\
        alo-agent:x:60989:60989:alo OS agent:/var/lib/alo-agent:/usr/sbin/nologin\n\
        alo:x:1000:1000:alo OS:/home/alo:/bin/bash\n";
    /// The groups, with the person in `wheel`.
    const GROUPS: &str = "wheel:x:10:alo\nalo-agent:x:60989:alo\nalo:x:1000:\n";

    /// **The person is found by number**, and nobody else is a person.
    #[test]
    fn the_person_is_found_and_nobody_else_is_one() {
        assert_eq!(a_person(LOGINS, 1000), Some("alo"));
        assert_eq!(a_person(LOGINS, 0), None, "root is not a person here");
        assert_eq!(a_person(LOGINS, 60989), None, "nor is the agent");
        assert_eq!(a_person(LOGINS, 1001), None, "nor a number nobody has");
        assert_eq!(the_name_of(LOGINS, 0), Some("root"));
    }

    /// **The agent is never a person, even under an ordinary number.** A list
    /// somebody edited is still not a reason to act for the agent.
    #[test]
    fn the_agent_is_never_a_person() {
        let edited = "alo-agent:x:1000:1000::/:/bin/sh\n";
        assert_eq!(a_person(edited, 1000), None);
        assert!(!is_an_administrator(
            "wheel:x:10:alo-agent\n",
            THE_AGENTS_LOGIN
        ));
    }

    /// **An administrator is a member of `wheel`**, and a member of another
    /// group by the same letters is not.
    #[test]
    fn an_administrator_is_a_member_of_wheel() {
        assert!(is_an_administrator(GROUPS, "alo"));
        assert!(!is_an_administrator(GROUPS, "ada"));
        assert!(!is_an_administrator("wheel:x:10:\nalo:x:1000:alo\n", "alo"));
        assert!(is_an_administrator("wheel:x:10:ada,alo\n", "alo"));
        assert!(!is_an_administrator("wheels:x:11:alo\n", "alo"));
    }

    /// **A broken line is no login**, rather than a panic or a guess.
    #[test]
    fn a_broken_line_is_no_login() {
        assert_eq!(the_name_of("alo\n:x:1000:1000\nalo:x:ten:1\n", 1000), None);
        assert!(!is_an_administrator("wheel\n", "alo"));
    }
}
