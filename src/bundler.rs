use std::fs::{self};
use std::path::{Component, Path, PathBuf};

use anyhow::{Result, bail};
use chrono::Local;
use walkdir::WalkDir;

pub fn bundle_paths(
    source_path: impl AsRef<Path>,
    backup_folder_path: impl AsRef<Path>,
) -> Result<PathBuf> {
    let source_path: &Path = source_path.as_ref();
    let timestamped_backup_folder_path: PathBuf =
        generate_bundled_name(backup_folder_path.as_ref())?;

    let full_backup_folder_path: PathBuf =
        build_full_backup_path(source_path, &timestamped_backup_folder_path);

    fs::create_dir_all(&full_backup_folder_path)?;

    if !source_path.is_absolute() {
        bail!("source path '{}' is not absolute", source_path.display());
    }

    bundle_element(
        source_path,
        full_backup_folder_path,
        &timestamped_backup_folder_path,
    )?;

    return Ok(timestamped_backup_folder_path);
}

fn bundle_element(
    source_path: impl AsRef<Path>,
    destination_path: impl AsRef<Path>,
    original_element_path: impl AsRef<Path>,
) -> Result<()> {
    let source_path: &Path = source_path.as_ref();
    let destination_path: &Path = destination_path.as_ref();
    let original_element_path: &Path = original_element_path.as_ref();

    // element is a folder:
    if source_path.is_dir() {
        for entry in fs::read_dir(&source_path)? {
            let entry: PathBuf = entry?.path();
            if entry.is_dir() {
                bundle_folder(entry, original_element_path)?;
            } else {
                copy_file_from_folder(&entry, destination_path)?;
            }
        }
        return Ok(());
    }

    // element is a file:
    bundle_file(source_path, destination_path)?;
    return Ok(());
}

fn bundle_folder(folder: impl AsRef<Path>, backup_folder_path: impl AsRef<Path>) -> Result<()> {
    let folder: &Path = folder.as_ref();
    let backup_folder_path: &Path = backup_folder_path.as_ref();

    if !folder.is_dir() {
        bail!("path '{}' is not a folder", folder.display());
    }

    let full_backup_folder_path: PathBuf =
        create_folder_in_backup_structure(&folder, backup_folder_path)?;

    for file in WalkDir::new(&folder).max_depth(1) {
        let file = file?;
        if PathBuf::from(file.path()).is_dir() {
            if file.path().canonicalize()? != folder.canonicalize()? {
                bundle_folder(file.path().to_path_buf(), backup_folder_path)?;
            }
            continue;
        }
        let entry_path = file.path().to_path_buf();

        copy_file_from_folder(entry_path, &full_backup_folder_path)?;
    }

    return Ok(());
}

fn bundle_file(source_path: impl AsRef<Path>, backup_folder_path: impl AsRef<Path>) -> Result<()> {
    let source_path: &Path = source_path.as_ref();
    let backup_folder_path: &Path = backup_folder_path.as_ref();

    if !source_path.is_file() {
        bail!("path '{}' is not a file", source_path.display());
    }

    let full_backup_folder_path: PathBuf = build_full_backup_path(source_path, backup_folder_path);

    let Some(parent_dir) = full_backup_folder_path.parent() else {
        bail!(
            "internal error extracting parent from {}",
            full_backup_folder_path.display()
        );
    };

    fs::create_dir_all(&parent_dir)?;

    copy_file_from_folder(source_path, &parent_dir.to_path_buf())?;

    return Ok(());
}

fn copy_file_from_folder(
    file: impl AsRef<Path>,
    destination_folder: impl AsRef<Path>,
) -> Result<()> {
    let file: &Path = file.as_ref();

    if file.is_dir() {
        bail!("path '{}' is not a file", file.display());
    }

    if !file.is_file() {
        println!("skipping non-regular file {}", file.display());
        return Ok(());
    }

    let mut file_destination: PathBuf = destination_folder.as_ref().to_path_buf();

    if let Some(file_name) = file.file_name() {
        file_destination.push(file_name);
    } else {
        println!(
            "skipping file {}: path has no final component",
            file.display()
        );
        return Ok(());
    };

    if let Err(err) = fs::copy(&file, &file_destination) {
        eprintln!(
            "failed to copy file {} → {}: {}",
            file.display(),
            file_destination.display(),
            err
        );
    }

    return Ok(());
}

fn create_folder_in_backup_structure(
    source_path_folder: impl AsRef<Path>,
    backup_folder_path: impl AsRef<Path>,
) -> Result<PathBuf> {
    let source_path_folder: PathBuf = PathBuf::from(source_path_folder.as_ref());
    let backup_folder_path: PathBuf = PathBuf::from(backup_folder_path.as_ref());

    let relative_source_path: PathBuf = PathBuf::from(
        source_path_folder
            .strip_prefix(Component::RootDir)
            .unwrap_or(&source_path_folder), // unwrap_or returns the default value if the strip_prefix was not able to remove any RootDir component.
    );

    let full_backup_folder_path: PathBuf = backup_folder_path.join(relative_source_path);

    fs::create_dir_all(&full_backup_folder_path)?;

    return Ok(full_backup_folder_path);
}

fn build_full_backup_path(
    source_path: impl AsRef<Path>,
    backup_folder_path: impl AsRef<Path>,
) -> PathBuf {
    let source_path: &Path = source_path.as_ref();
    let backup_folder_path: &Path = backup_folder_path.as_ref();

    let relative_source_path: &Path = source_path
        .strip_prefix(Component::RootDir)
        .unwrap_or(source_path);

    return backup_folder_path.join(relative_source_path);
}

fn generate_bundled_name(folder_path: impl AsRef<Path>) -> Result<PathBuf> {
    let folder_path: PathBuf = PathBuf::from(folder_path.as_ref());
    let now: chrono::DateTime<Local> = Local::now();

    let timestamp: String = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    let folder_name: PathBuf = PathBuf::from(format!("{}-{}", folder_path.display(), timestamp));

    return Ok(folder_name);
}
