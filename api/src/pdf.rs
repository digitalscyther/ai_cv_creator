use std::{env, fs};
use std::io::{Error, Write};
use tempfile::NamedTempFile;
use tokio::process::Command;

pub async fn generate_pdf(text: &str, output_temp: &NamedTempFile) -> Result<(), Error> {
    let program = env::var("MDPROOF_FP").expect("MDPROOF_FP must be set");

    let mut input_temp = NamedTempFile::new().unwrap();
    input_temp.write_all(text.as_bytes()).unwrap();

    let input_temp_filepath = input_temp.path().to_str().unwrap().to_string();
    let output_temp_filepath = output_temp.path().to_str().unwrap().to_string();

    let mut command = Command::new(program);
    command.args(&[&input_temp_filepath, "-o", &output_temp_filepath]);

    let child = command.spawn()?;
    let _ = child.wait_with_output().await?;

    fs::remove_file(&input_temp_filepath).unwrap();

    Ok(())
}