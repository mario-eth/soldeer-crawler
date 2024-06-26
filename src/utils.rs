use std::env;
use std::fmt;
use std::fs::{self};
use std::path::PathBuf;

// get the current working directory
pub fn get_current_working_dir() -> std::io::Result<PathBuf> {
    env::current_dir()
}

pub fn read_file_to_string(filename: String) -> Result<String, FileNotFound> {
    let contents: String = match fs::read_to_string(&filename) {
        // If successful return the files text as `contents`.
        // `c` is a local variable.
        Ok(c) => c,
        // Handle the `error` case.
        Err(_) => {
            eprintln!("Could not read file `{}`", &filename);
            return Err(FileNotFound);
        }
    };
    Ok(contents)
}

#[derive(Debug, Clone)]
pub struct FileNotFound;

impl fmt::Display for FileNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "file not found")
    }
}
