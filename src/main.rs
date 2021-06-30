use std::env::args;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub enum MyErrors {
    CannotReadDirectory,
}

fn main() {
    let command = args().nth(1);

    match command.as_ref().map(String::as_ref) {
        Some("list") => list(),
        Some("recent") => recent(),
        Some("help") => help(),
        None => help(),
        _ => create(),
    }
}

fn list() {
    unimplemented!();
}

fn list_files(dir: PathBuf) -> Result<Vec<PathBuf>, MyErrors> {
    let mut all_files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_dir() {
                        if let Ok(mut res) = list_files(entry.path()) {
                            all_files.append(&mut res);
                        }
                    } else {
                        all_files.push(entry.path());
                    }
                }
            }
        }
    } else {
        return Err(MyErrors::CannotReadDirectory);
    }

    return Ok(all_files);
}

fn recent() {
    unimplemented!();
}

fn help() {
    unimplemented!();
}

fn create() {
    unimplemented!();
}
