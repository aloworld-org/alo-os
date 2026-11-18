//! Bounded acknowledgements from the person controller, never policy writes.

#[cfg(unix)]
use std::{io::Write, os::fd::AsFd, time::Instant};

/// Flush a fixed stage token, then require its exact newline-terminated reply.
///
/// The caller owns the only reader of `input`. Each byte is polled against the
/// same absolute deadline, so an incomplete line cannot restart the allowance.
/// The output is a caller-owned control pipe, not an agent or approval channel.
///
/// # Errors
/// Refuse expired deadlines, I/O failures, EOF and any non-exact acknowledgement.
#[cfg(unix)]
pub fn exchange_stage(
    stage: crate::studio_agent_probe::Stage,
    input: &impl AsFd,
    output: &mut impl Write,
    deadline: Instant,
) -> Result<(), String> {
    use rustix::event::{PollFd, PollFlags, Timespec};

    let expected = format!("{}\n", crate::studio_agent_launch::stage_token(stage));
    if deadline <= Instant::now() {
        return Err("the native boundary deadline has expired".to_owned());
    }
    output
        .write_all(expected.as_bytes())
        .and_then(|()| output.flush())
        .map_err(|_| "the native boundary could not be announced".to_owned())?;
    for wanted in expected.bytes() {
        let left = deadline
            .checked_duration_since(Instant::now())
            .filter(|left| !left.is_zero())
            .ok_or_else(|| "the native boundary deadline has expired".to_owned())?;
        let timeout = Timespec::try_from(left)
            .map_err(|_| "the native boundary deadline is invalid".to_owned())?;
        let mut polling = [PollFd::new(input, PollFlags::IN)];
        if rustix::event::poll(&mut polling, Some(&timeout))
            .map_err(|_| "the native boundary input could not be polled".to_owned())?
            == 0
        {
            return Err("the native boundary deadline has expired".to_owned());
        }
        if !polling
            .first()
            .is_some_and(|fd| fd.revents().contains(PollFlags::IN))
        {
            return Err("the native boundary input closed or failed".to_owned());
        }
        let mut observed = [0_u8; 1];
        if rustix::io::read(input, &mut observed)
            .map_err(|_| "the native boundary input could not be read".to_owned())?
            != 1
            || observed.first().copied() != Some(wanted)
        {
            return Err("the native boundary acknowledgement did not match".to_owned());
        }
    }
    Ok(())
}
