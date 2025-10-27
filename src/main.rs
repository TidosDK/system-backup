// TODO: Handle panics instead of using unwrap.

use std::fs::{self, File};
use std::path::{Path, PathBuf};

use chrono::Local;
use tar::Builder;

fn main() {
    let backup_folder: &str = "laptop-backup";

    backup_files_from("/home/matty/.ssh/", backup_folder);

    zip_files_in_folder(backup_folder);
}

pub fn backup_files_from(folder_path: &str, backup_folder: &str) {
    let folder_path: PathBuf = PathBuf::from(folder_path);

    if let Ok(_) = fs::create_dir_all(backup_folder) {
        println!("Created folder {:?}", backup_folder);
    }

    if !Path::new(backup_folder).exists() {
        eprint!("The folder {:?} could not be created", backup_folder);
        return;
    }

    if folder_path.is_dir() {
        if let Ok(entries) = fs::read_dir(&folder_path) {
            for entry_result in entries {
                if let Ok(entry) = entry_result {
                    let path_to_file: PathBuf = entry.path();
                    copy_file_from(path_to_file, backup_folder);
                } else if let Err(e) = entry_result {
                    eprintln!("Error reading entry: {}", e);
                }
            }
        } else {
            eprintln!("Failed to read directory {:?}", folder_path);
        }
    } else {
        eprint!("The folder, {} is not a folder", folder_path.display());
    }
}

// TODO: It only copies files, not folders. Look at the "WalkDir" crate.
fn copy_file_from(file: PathBuf, destination_folder: &str) {
    let destination_folder: PathBuf = PathBuf::from(destination_folder);

    if file.is_file() {
        let mut file_destination = destination_folder.to_path_buf();
        file_destination.push(file.file_name().unwrap());

        if let Err(e) = fs::copy(&file, &file_destination) {
            eprintln!("Failed to copy {}: {}", file.display(), e)
        }
    }
}

fn zip_files_in_folder(folder_path: &str) {
    let folder = Path::new(folder_path);
    let now = Local::now();

    let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    let filename: String = format!("{}-{}.tar", folder.display(), timestamp);

    let tar_file = match File::create(&filename) {
        Ok(file) => file,
        Err(e) => {
            eprintln!(
                "Failed to create the archive file named {}: {}",
                filename, e
            );
            return;
        }
    };

    let mut archive: Builder<File> = Builder::new(tar_file);

    println!("Archiving everything in the folder {}", folder_path);

    if let Err(e) = archive.append_dir_all(folder_path, folder_path) {
        eprintln!(
            "Failed to append directory {} to archive: {}",
            folder_path, e
        );
        return;
    }

    if let Err(e) = archive.finish() {
        eprintln!("Failed to finish archive {}: {}", filename, e);
        return;
    }
}
