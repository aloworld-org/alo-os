//! Refuse private agent observations made with retained person privileges.

/// Require the disposable image's complete, unprivileged agent credentials.
///
/// This validates an observation, not its origin. The private launcher must
/// read its own bounded `/proc/self/status` after dropping privileges; supplied
/// text alone cannot establish kernel separation or an installed acceptance.
///
/// # Errors
/// Missing, duplicated or malformed credential fields, another identity,
/// retained capabilities/groups, or permission to acquire new privileges.
pub fn require_confinement(status: &str) -> Result<(), &'static str> {
    if status.len() > 65_536 {
        return Err("process status exceeds the private probe limit");
    }
    for field in ["Uid", "Gid"] {
        let ids: Vec<_> = one_field(status, field)?.split_whitespace().collect();
        if ids.len() != 4 || ids.iter().any(|id| id.parse::<u32>() != Ok(60_989)) {
            return Err("all process identities must be the disposable agent");
        }
    }
    for field in ["CapInh", "CapEff", "CapPrm", "CapAmb", "CapBnd"] {
        let value = one_field(status, field)?;
        if value.is_empty()
            || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
            || u64::from_str_radix(value, 16) != Ok(0)
        {
            return Err("the private agent must retain no capabilities");
        }
    }
    if !one_field(status, "Groups")?.is_empty() {
        return Err("the private agent must retain no supplementary groups");
    }
    if one_field(status, "NoNewPrivs")? != "1" {
        return Err("the private agent must not acquire new privileges");
    }
    Ok(())
}

/// Kernel fields must be present once; neither first nor last value wins.
fn one_field<'a>(status: &'a str, wanted: &str) -> Result<&'a str, &'static str> {
    let mut fields = status.lines().filter_map(|line| {
        let (name, value) = line.split_once(':')?;
        (name == wanted).then_some(value.trim())
    });
    let value = fields.next().ok_or("process credential field is missing")?;
    if fields.next().is_some() {
        return Err("process credential field is duplicated");
    }
    Ok(value)
}
