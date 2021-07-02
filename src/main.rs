use std::env::args;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub enum MyErrors {
    MissingCommand,
    CannotReadDirectory,
}

fn main() -> Result<(), MyErrors> {
    let command = args().nth(1);

    match command.as_ref().map(String::as_ref) {
        Some("list") => list()?,
        Some("recent") => recent()?,
        Some("help") => help()?,
        None => help()?,
        _ => create()?,
    };

    Ok(())
}

fn get_command_option() -> Result<String, MyErrors> {
    let option = match args().nth(2) {
        Some(value) => value,
        None => return Err(MyErrors::MissingCommand),
    };
    Ok(option)
}

fn list() -> Result<(), MyErrors> {
    let company = get_command_option()?;
    let query = ["-", &company.to_lowercase(), "."].concat();
    let root = PathBuf::from("/tmp/interviews");

    if let Ok(all_files) = list_files(root) {
        for path in all_files {
            if path.display().to_string().contains(&query)
                && path
                    .extension()
                    .map(|x| x == "md" || x == "eml")
                    .unwrap_or(false)
            {
                println!("subl {:?}", path);
            }
        }
    }

    Ok(())
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

    all_files.sort();

    return Ok(all_files);
}

fn recent() -> Result<(), MyErrors> {
    unimplemented!();
}

fn help() -> Result<(), MyErrors> {
    unimplemented!();
}

fn create() -> Result<(), MyErrors> {
    unimplemented!();
}
