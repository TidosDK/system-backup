use std::fs::{self, File, remove_dir_all};
use std::io;
use std::path::{Component, Path, PathBuf};

use chrono::Local;
use tar::Builder;
use walkdir::WalkDir;

static BACKUP_FOLDER_PATH: &str = "laptop-backup";

fn main() {
    let paths: Vec<&str> = vec![
        "/home/matty/.ssh/",
        "/home/matty/.kube/",
        "/home/matty/Downloads/test/",
    ];

    for path in &paths {
        if let Err(err) = backup_files_from_folder(path) {
            eprintln!("{:?}", err);
        }
    }

    zip_files_in_folder(BACKUP_FOLDER_PATH);
}

pub fn backup_files_from_folder<T: AsRef<Path>>(source_path: T) -> io::Result<()> {
    let source_path: PathBuf = PathBuf::from(source_path.as_ref());
    let backup_folder_path: PathBuf = PathBuf::from(BACKUP_FOLDER_PATH);

    if !source_path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("source path '{}' is not absolute", source_path.display()),
        ));
    }

    if !source_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("source path '{}' is not a directory", source_path.display()),
        ));
    }

    let relative_source_path: PathBuf = PathBuf::from(
        source_path
            .strip_prefix(Component::RootDir)
            .unwrap_or(&source_path), // unwrap_or returns the default value if the strip_prefix was not able to remove any RootDir component.
    );

    let full_backup_folder_path: PathBuf = backup_folder_path.join(relative_source_path);

    fs::create_dir_all(&full_backup_folder_path)?;

    for entry in fs::read_dir(&source_path)? {
        let file_path = entry?.path();
        copy_file_from_folder(file_path, &full_backup_folder_path)?;
    }

    return Ok(());
}

fn copy_file_from_folder(file: PathBuf, destination_folder: &PathBuf) -> io::Result<()> {
    if file.is_dir() {
        return backup_folder(file); // The "file" it is actually a folder in this context.
    }

    // Skip non-regular files (symlinks, etc.)
    if !file.is_file() {
        eprintln!(
            "Skipping non-regular file '{}': not a regular file",
            file.display()
        );
        return Ok(());
    }
    let mut file_destination: PathBuf = destination_folder.to_path_buf();

    if let Some(file_name) = file.file_name() {
        file_destination.push(file_name);
    } else {
        eprintln!("Skipping '{}': path has no final component", file.display());
        return Ok(());
    };

    if let Err(err) = fs::copy(&file, &file_destination) {
        eprintln!(
            "Failed to copy {} → {}: {}",
            file.display(),
            file_destination.display(),
            err
        );
    }

    return Ok(());
}

fn backup_folder(folder: PathBuf) -> io::Result<()> {
    if !folder.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("source path '{}' is not a directory", folder.display()),
        ));
    }

    let full_backup_folder_path: PathBuf = create_folder_in_backup_structure(&folder)?;

    for file in WalkDir::new(&folder).max_depth(1) {
        let file = file?;
        if PathBuf::from(file.path()).is_dir() {
            if file.path().canonicalize()? != folder.canonicalize()? {
                backup_folder(file.path().to_path_buf())?;
            }
            continue;
        }
        let entry_path = file.path().to_path_buf();

        copy_file_from_folder(entry_path.to_path_buf(), &full_backup_folder_path)?;
    }

    return Ok(());
}

fn create_folder_in_backup_structure<T: AsRef<Path>>(source_path_folder: T) -> io::Result<PathBuf> {
    let source_path_folder: PathBuf = PathBuf::from(source_path_folder.as_ref());
    let backup_folder_path: PathBuf = PathBuf::from(BACKUP_FOLDER_PATH);

    let relative_source_path: PathBuf = PathBuf::from(
        source_path_folder
            .strip_prefix(Component::RootDir)
            .unwrap_or(&source_path_folder), // unwrap_or returns the default value if the strip_prefix was not able to remove any RootDir component.
    );

    let full_backup_folder_path: PathBuf = backup_folder_path.join(relative_source_path);

    fs::create_dir_all(&full_backup_folder_path)?;

    return Ok(full_backup_folder_path);
}

pub fn zip_files_in_folder(folder_path: &str) {
    let folder = Path::new(folder_path);
    let now = Local::now();

    let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    let filename: String = format!("{}-{}.tar", folder.display(), timestamp);

    let tar_file = match File::create(&filename) {
        Ok(file) => file,
        Err(err) => {
            eprintln!(
                "Failed to create the archive file named {}: {}",
                filename, err
            );
            return;
        }
    };

    let mut archive: Builder<File> = Builder::new(tar_file);

    println!("Archiving everything in the folder {}", folder_path);

    if let Err(err) = archive.append_dir_all(folder_path, folder_path) {
        eprintln!(
            "Failed to append directory {} to archive: {}",
            folder_path, err
        );
        return;
    }

    if let Err(err) = archive.finish() {
        eprintln!("Failed to finish archive {}: {}", filename, err);
        return;
    }

    if let Err(err) = remove_dir_all(folder_path) {
        println!("Failed to remove backup folder {}: {}", folder_path, err);
    }
}
