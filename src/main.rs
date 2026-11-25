mod archiver;
mod bundler;
mod config_handler;
mod encryption;

use std::{path::PathBuf, process};

use crate::{
    archiver::zip_files_in_folder,
    bundler::{bundle_paths, generate_bundled_name},
    config_handler::{load_paths_from_file, load_public_key_from_file},
    encryption::encrypt_file,
};

static PATHS_FILE: &str = "paths.txt";
static PUBLIC_KEY_FILE: &str = "public_key.txt";
static BACKUP_FOLDER_PATH: &str = "./system-backup";

fn main() {
    let paths: Vec<String> = load_paths_from_file(PATHS_FILE).expect("failed to load path file");
    let public_encryption_key: String =
        load_public_key_from_file(PUBLIC_KEY_FILE).expect("failed to load public key");

    let mut backup_succeeded: bool = false;

    let backup_folder_path = generate_bundled_name(BACKUP_FOLDER_PATH).unwrap_or_else(|err| {
        eprintln!("Failed to create backup folder path: {:?}", err);
        process::exit(1);
    });

    // Bundle all paths into a single folder
    for path in paths {
        match bundle_paths(path, &backup_folder_path) {
            Ok(()) => backup_succeeded = true,
            Err(err) => {
                eprintln!("Error retrieving file/folder: {:?}", err);
            }
        }
    }

    if !backup_succeeded {
        eprintln!("no backups were successfully initialized");
        process::exit(1);
    };

    // Archive all paths
    let archive_file: PathBuf = zip_files_in_folder(&backup_folder_path).unwrap_or_else(|err| {
        eprintln!("Failed to create archive: {:?}", err);
        process::exit(1);
    });

    // Encrypt the archive
    if let Err(err) = encrypt_file(archive_file, public_encryption_key) {
        eprintln!("Encryption failed: {err}");
        process::exit(1);
    }
}
