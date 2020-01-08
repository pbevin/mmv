use std::env;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::Path;
use std::fs;
use std::process::{Command, exit};
use tempfile::NamedTempFile;

#[derive(Debug)]
enum Error {
    IOError(io::Error),
    WrongLength,
    RenameError(String, String, String),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IOError(err)
    }
}


fn bulk_rename(names: Vec<String>) -> Result<(), Error> {
    let new_names = get_new_names(&names)?;
    let changes = calc_changes(&names, &new_names)?;
    if changes.is_empty() {
        println!("No changes.");
    } else {
        exec_changes(changes)?;
    }
    return Ok(());
}

fn calc_changes(names: &Vec<String>, new_names: &Vec<String>) -> Result<Vec<(String, String)>, Error> {
    if names.len() != new_names.len() {
        for name in new_names {
            println!("{}", name);
        }
        return Err(Error::WrongLength);
    } else {
        return Ok(names
                  .iter()
                  .zip(new_names)
                  .filter(|(a, b)| a != b)
                  .map(|(a, b)| (a.to_string(), b.to_string()))
                  .collect());
    }
}

fn exec_changes(changes: Vec<(String, String)>) -> Result<(), Error> {
    for (name, new_name) in changes {
        let result = fs::rename(&name, &new_name);
        if let Err(e) = result {
            println!("{:?}", e);
            return Err(Error::RenameError(name, new_name, e.to_string()));
        }
    }

    return Ok(());
}

fn get_new_names(names: &Vec<String>) -> Result<Vec<String>, io::Error> {
    let mut file = NamedTempFile::new()?;
    for name in names {
        writeln!(file, "{}", name)?;
    }

    file.as_file_mut().sync_all()?;
    invoke_editor(file.path())?;
    file.as_file_mut().seek(SeekFrom::Start(0))?;

    let file = file.into_file();
    let reader = BufReader::new(file);
    let new_names = reader.lines().map(|line| line.unwrap()).collect();
    return Ok(new_names);
}

fn invoke_editor(path: &Path) -> Result<(), io::Error> {
    let editor = env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());

    Command::new(editor)
        .arg(path)
        .status()
        .expect("failed to execute process");

    return Ok(());
}

fn main() {
    let names = env::args()
        .skip(1)
        .collect::<Vec<String>>();

    if names.is_empty() {
        println!("Usage: {} [path...]", env::args().nth(0).unwrap());
        return;
    }

    let result = bulk_rename(names);
    match result {
        Ok(_) => {}
        Err(err) => {
            println!("{:?}", err);
            exit(1);
        }
    }
}
