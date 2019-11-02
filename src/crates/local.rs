use super::CrateTrait;
use crate::Workspace;
use failure::Error;
use log::info;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub(super) struct Local {
    path: PathBuf,
}

impl Local {
    pub(super) fn new(path: &Path) -> Self {
        Local { path: path.into() }
    }
}

impl CrateTrait for Local {
    fn fetch(&self, _workspace: &Workspace) -> Result<(), Error> {
        // There is no fetch to do for a local crate.
        Ok(())
    }

    fn purge_from_cache(&self, _workspace: &Workspace) -> Result<(), Error> {
        // There is no cache to purge for a local crate.
        Ok(())
    }

    fn copy_source_to(&self, _workspace: &Workspace, dest: &Path) -> Result<(), Error> {
        info!(
            "copying local crate from {} to {}",
            self.path.display(),
            dest.display()
        );
        copy_dir(&self.path, dest)?;
        Ok(())
    }
}

impl std::fmt::Display for Local {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "local crate {}", self.path.display())
    }
}

fn copy_dir(src: &Path, dest: &Path) -> Result<(), Error> {
    let src = crate::utils::normalize_path(src);
    let dest = crate::utils::normalize_path(dest);

    let src_components = src.components().count();
    let mut entries = WalkDir::new(&src).into_iter();
    while let Some(entry) = entries.next() {
        let entry = entry?;

        let mut components = entry.path().components();
        for _ in 0..src_components {
            components.next();
        }
        let path = components.as_path();

        if entry.file_type().is_dir() {
            // don't copy /target directory
            if entry.file_name() == "target" && entry.depth() == 1 {
                info!("ignoring top-level target directory {}", path.display());
                entries.skip_current_dir();
            } else {
                std::fs::create_dir_all(dest.join(path))?;
            }
        } else {
            std::fs::copy(src.join(path), dest.join(path))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use failure::Error;

    #[test]
    fn test_copy_dir() -> Result<(), Error> {
        let tmp_src = tempfile::tempdir()?;
        let tmp_dest = tempfile::tempdir()?;

        // Create some files in the src dir
        std::fs::create_dir(tmp_src.path().join("dir"))?;
        std::fs::write(tmp_src.path().join("foo"), b"Hello world")?;
        std::fs::write(tmp_src.path().join("dir").join("bar"), b"Rustwide")?;

        super::copy_dir(tmp_src.path(), tmp_dest.path())?;

        assert_eq!(std::fs::read(tmp_dest.path().join("foo"))?, b"Hello world");
        assert_eq!(
            std::fs::read(tmp_dest.path().join("dir").join("bar"))?,
            b"Rustwide"
        );

        Ok(())
    }

    #[test]
    fn test_no_copy_target() -> Result<(), Error> {
        use log::debug;
        env_logger::init();

        debug!("debugging works");
        let (src, dest) = (tempfile::tempdir()?, tempfile::tempdir()?);
        debug!("made root dirs");
        std::fs::create_dir(src.path().join("target"))?;
        std::fs::write(src.path().join("target").join("a.out"), b"this is not actually an ELF file")?;
        debug!("made subdirs and files");

        super::copy_dir(src.path(), dest.path())?;
        debug!("copied");

        assert!(!dest.path().join("target").exists());

        Ok(())
    }
}
