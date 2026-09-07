//! Strict worker result parsing; a prose mention of completion cannot restart work.
use crate::Result;

pub fn expected_remote(remote: &str) -> bool {
    matches!(
        remote,
        "https://github.com/aloworld-org/alo-os.git"
            | "https://github.com/aloworld-org/alo-os"
            | "git@github.com:aloworld-org/alo-os.git"
    )
}

pub fn done_title(text: &str) -> Result<&str> {
    let mut lines = text.lines();
    if lines.next() != Some("STEP DONE") {
        return Err(format!(
            "Worker did not finish a step; see result.txt: {}",
            text.chars().take(500).collect::<String>()
        )
        .into());
    }
    let title = lines.next().ok_or("Missing commit title")?;
    if title.len() > 120
        || title.chars().any(char::is_control)
        || !title.contains(": ")
        || ![
            "feat(",
            "fix(",
            "test(",
            "docs(",
            "build(",
            "chore(",
            "refactor(",
        ]
        .iter()
        .any(|prefix| title.starts_with(prefix))
    {
        return Err("Invalid conventional commit title".into());
    }
    Ok(title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_literal_done_report_can_publish() {
        assert!(done_title("STEP DONE\nfeat(shell): draw windows\nVerified").is_ok());
        for text in [
            "STEP BLOCKED\nNeed hardware",
            "We reached STEP DONE",
            "STEP DONE",
            "STEP DONE\n--amend",
            "STEP DONE\nfeat(shell): bad\ttitle",
        ] {
            assert!(done_title(text).is_err());
        }
    }

    #[test]
    fn only_the_owners_os_remote_is_allowed() {
        assert!(expected_remote(
            "https://github.com/aloworld-org/alo-os.git"
        ));
        assert!(!expected_remote(
            "https://github.com/aloworld-org/alo-workplace.git"
        ));
        assert!(!expected_remote(
            "https://github.com/aloworld-org/alo-os.git.evil"
        ));
    }

    #[test]
    fn lock_excludes_a_second_supervisor_and_releases_on_close() -> Result<()> {
        let directory = std::env::temp_dir().join(format!("dev-loop-lock-{}", std::process::id()));
        std::fs::create_dir_all(&directory)?;
        let path = directory.join("lock");
        let first = std::fs::File::create(&path)?;
        first.try_lock()?;
        let second = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)?;
        assert!(second.try_lock().is_err());
        drop(first);
        second.try_lock()?;
        drop(second);
        std::fs::remove_file(path)?;
        std::fs::remove_dir(directory)?;
        Ok(())
    }
}
