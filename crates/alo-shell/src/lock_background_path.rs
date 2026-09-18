//! Resolve the policy-selected image path, including bounded sorted rotations.
use crate::RenderError;
use alo_appearance::{Background, Fitting, Of};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
/// Select only the named image or the model-selected sorted rotation entry.
pub(crate) fn selected(
    chosen: &Background,
    running: Duration,
    shipped: &Path,
) -> Result<(PathBuf, Fitting), RenderError> {
    match chosen {
        Background::Picture(p) => Ok((
            match p.of() {
                Of::Shipped(name) => shipped.join(format!("{name}.png")),
                Of::File(path) => path.clone(),
            },
            p.fitting(),
        )),
        Background::Rotating(r) => {
            let mut paths = Vec::new();
            for (n, entry) in std::fs::read_dir(r.where_they_are())
                .map_err(image_error)?
                .enumerate()
            {
                if n >= 4096 {
                    return Err(RenderError::LockScene);
                }
                let entry = entry.map_err(image_error)?;
                if entry.file_type().map_err(image_error)?.is_file()
                    && entry.path().extension().is_some_and(|e| {
                        e.eq_ignore_ascii_case("png")
                            || e.eq_ignore_ascii_case("jpg")
                            || e.eq_ignore_ascii_case("jpeg")
                    })
                {
                    paths.push(entry.path());
                }
            }
            paths.sort();
            let which = r
                .showing(paths.len(), running)
                .ok_or(RenderError::LockScene)?;
            Ok((
                paths.get(which).ok_or(RenderError::LockScene)?.clone(),
                r.fitting(),
            ))
        }
        Background::Colour(_) => Err(RenderError::LockScene),
    }
}

/// Keep diagnostic image errors out of the translated UI.
fn image_error(error: impl std::fmt::Display) -> RenderError {
    RenderError::Submission(format!("lock background: {error}"))
}
