//! Integrate concurrent main updates and publish only the tested combined tree.
use crate::Result;

/// Replay unpublished work over advancing main and retest before a normal push.
/// Verification may repair and re-gate the combined tree. An unresolved error,
/// conflict or unchanged remote after a push failure preserves work and halts.
pub fn publish(
    base: &str,
    mut git: impl FnMut(&[&str]) -> Result<String>,
    mut verify: impl FnMut() -> Result<()>,
) -> Result<String> {
    let mut verified_remote = base.to_owned();
    for _ in 0..3 {
        git(&["fetch", "origin", "main"])?;
        let remote = git(&["rev-parse", "refs/remotes/origin/main"])?;
        if remote != verified_remote {
            git(&["merge-base", "--is-ancestor", base.trim(), remote.trim()])
                .map_err(|_| "Published main history changed; review before integrating")?;
            git(&["rebase", "refs/remotes/origin/main"])
                .map_err(|error| format!("Cannot integrate main: {error}. Resolve the conflict and rerun gates; unpublished work is preserved."))?;
            verify()?;
            if !git(&["status", "--porcelain"])?.trim().is_empty() {
                return Err("Integration checks changed the working tree; nothing pushed".into());
            }
            verified_remote = remote;
        }
        let head = git(&["rev-parse", "HEAD"])?;
        if head == verified_remote {
            // Another contributor already integrated the same work.
            return Ok(head);
        }
        match git(&["push", "origin", "HEAD:refs/heads/main"]) {
            Ok(_) => return Ok(head),
            Err(error) => {
                // Retry only an actual race. Auth, connectivity and branch
                // protection failures are actionable errors, not progress.
                git(&["fetch", "origin", "main"])?;
                if git(&["rev-parse", "refs/remotes/origin/main"])? == verified_remote {
                    return Err(error);
                }
            }
        }
    }
    Err(
        "Main advanced during three publication attempts; tested local work is preserved for retry"
            .into(),
    )
}

#[cfg(test)]
mod tests;
