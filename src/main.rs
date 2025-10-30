use std::fs::{self, File, remove_dir_all};
use std::io;
use std::path::{Component, Path, PathBuf};

use chrono::Local;
use tar::Builder;

fn main() {
    let backup_folder_path: &str = "laptop-backup";

    let paths: Vec<&str> = vec![
        "/home/matty/.ssh/",
        "/home/matty/.kube/",
        "/home/matty/Downloads/test/",
    ];

    for path in &paths {
        if let Err(err) = backup_files_from(path, &backup_folder_path) {
            eprintln!("{:?}", err);
        }
    }

    zip_files_in_folder(backup_folder_path);
}

pub fn backup_files_from<T: AsRef<Path>>(source_path: T, backup_folder_path: T) -> io::Result<()> {
    let source_path: PathBuf = PathBuf::from(source_path.as_ref());
    let backup_folder_path: PathBuf = PathBuf::from(backup_folder_path.as_ref());

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
        copy_file_from(file_path, &full_backup_folder_path);
    }

    return Ok(());
}

// TODO: It only copies files, not folders. Look at the "WalkDir" crate.
fn copy_file_from(file: PathBuf, destination_folder: &PathBuf) {
    if !file.is_file() {
        eprintln!("Skipping '{}': not a file", file.display());
        return;
    }

    let mut file_destination: PathBuf = destination_folder.to_path_buf();

    if let Some(file_name) = file.file_name() {
        file_destination.push(file_name);
    } else {
        eprintln!("Skipping '{}': path has no final component", file.display());
        return;
    };

    if let Err(err) = fs::copy(&file, &file_destination) {
        eprintln!(
            "Failed to copy {} → {}: {}",
            file.display(),
            file_destination.display(),
            err
        );
    }
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
