use std::{env, fs};
use std::fs::File;
use std::io::{Error, Write};
use std::path::PathBuf;
use tempfile::{NamedTempFile, tempdir};
use tokio::process::Command;

pub async fn generate_pdf(text: &str, output_temp: &NamedTempFile) -> Result<(), Error> {
    let program = env::var("PROGRAM_FP").expect("PROGRAM_FP must be set");

    let dir = tempdir()?;
    let input_file_name = "my_temp_file.html";
    let input_file_path: PathBuf = dir.path().join(input_file_name);
    let mut input_file = File::create(&input_file_path)?;
    input_file.write_all(text.as_bytes()).unwrap();

    let input_temp_filepath = input_file_path.to_str().unwrap().to_string();
    let output_temp_filepath = output_temp.path().to_str().unwrap().to_string();

    let mut command = Command::new(program);
    command.args(&[&input_temp_filepath, &output_temp_filepath]);

    let child = command.spawn()?;
    let _ = child.wait_with_output().await?;

    fs::remove_file(&input_temp_filepath).unwrap();

    Ok(())
}