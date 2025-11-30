use std::fs::{File, remove_file};
use std::io::{self, BufReader, BufWriter, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use age::{Encryptor, x25519};
use anyhow::Result;

pub fn encrypt_file<P: AsRef<Path>>(input_file_path: P, public_key: String) -> Result<()> {
    let output_filename: PathBuf = input_file_path.as_ref().with_extension("tar.age");

    let recipient: x25519::Recipient = x25519::Recipient::from_str(&public_key).map_err(|e| {
        Error::new(
            ErrorKind::InvalidInput,
            format!("invalid age recipient \"{public_key}\": {e}"),
        )
    })?;

    let encryptor = Encryptor::with_recipients(std::iter::once(&recipient as &dyn age::Recipient))
        .expect("recipient iterator is non-empty");

    let mut input_file: BufReader<File> = BufReader::new(File::open(&input_file_path)?);
    let output_file: BufWriter<File> = BufWriter::new(File::create(output_filename)?);

    let mut encrypted_output_file: age::stream::StreamWriter<BufWriter<File>> = encryptor
        .wrap_output(output_file)
        .map_err(|err| Error::new(ErrorKind::Other, err.to_string()))?;

    io::copy(&mut input_file, &mut encrypted_output_file)?;
    encrypted_output_file.finish()?;

    remove_file(input_file_path)?;

    return Ok(());
}
