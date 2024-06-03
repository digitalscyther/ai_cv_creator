use std::fs;
use std::io::{Error, Write};
use tempfile::NamedTempFile;
use tokio::process::Command;

pub async fn generate_pdf(text: &str, output: &str, program: &str) -> Result<(), Error> {
    let output_temp = NamedTempFile::new().unwrap();
    let mut input_temp = NamedTempFile::new().unwrap();
    input_temp.write_all(text.as_bytes()).unwrap();

    let input_temp_filepath = input_temp.path().to_str().unwrap().to_string();
    let output_temp_filepath = output_temp.path().to_str().unwrap().to_string();

    let mut command = Command::new(program);
    command.args(&[&input_temp_filepath, "-o", &output_temp_filepath]);

    let child = command.spawn()?;
    let _ = child.wait_with_output().await?;

    fs::copy(&output_temp_filepath, output).unwrap();

    fs::remove_file(&input_temp_filepath).unwrap();
    fs::remove_file(&output_temp_filepath).unwrap();

    Ok(())
}