use std::fs::{File, remove_dir_all};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use tar::Builder;

pub fn zip_files_in_folder(backup_folder_path: impl AsRef<Path>) -> Result<PathBuf> {
    let folder: &Path = backup_folder_path.as_ref();

    if !PathBuf::from(folder).exists() {
        bail!("failed to create backup folder {}", folder.display());
    }

    let filename: String = format!("{}.tar", folder.display());

    let tar_file = match File::create(&filename) {
        Ok(file) => file,
        Err(_) => {
            bail!("failed to backup file named {}", filename);
        }
    };

    let mut archive: Builder<File> = Builder::new(tar_file);

    archive
        .append_dir_all(folder, folder)
        .with_context(|| format!("failed to append directory {} to archive", folder.display()))?;

    archive.finish()?;

    remove_dir_all(folder)
        .with_context(|| format!("failed to remove backup directory {}", folder.display()))?;

    return Ok(PathBuf::from(filename));
}
