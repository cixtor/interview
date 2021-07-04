use std::env::args;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;

use chrono::Datelike;

#[derive(Debug)]
pub enum MyErrors {
    FileNotFound,
    MissingCommand,
    CannotReadDirectory,
    InvalidBoundaryLine,
}

fn main() -> Result<(), MyErrors> {
    let command = args().nth(1);

    match command.as_ref().map(String::as_ref) {
        Some("open") => open()?,
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

fn open() -> Result<(), MyErrors> {
    let company = get_command_option()?;
    let files = list_company_files(company)?;

    let path = match files.iter().last() {
        Some(res) => res,
        None => return Err(MyErrors::FileNotFound),
    };

    let mut marker = 0;
    let mut boundary = String::from("--");
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        let line = line.unwrap();

        // Find the last occurrence of the email boundary.
        if boundary.len() > 2 && line.eq(&boundary) {
            marker = index;
            continue;
        }

        // Skip lines that are not a boundary header.
        if !line.contains("; boundary=") {
            continue;
        }

        // Extract the email boundary code from the header.
        if let Some(mark) = line.chars().position(|x| x == '=') {
            boundary.push_str(line.get(mark + 1..).unwrap());
            continue;
        }

        // Could not find an email boundary header anywhere.
        return Err(MyErrors::InvalidBoundaryLine);
    }

    println!("subl {:?}:{:?}", path, marker);

    Ok(())
}

fn list() -> Result<(), MyErrors> {
    let company = get_command_option()?;
    let files = list_company_files(company)?;

    for file in files {
        println!("subl {:?}", file);
    }

    Ok(())
}

fn list_files(dir: PathBuf) -> Result<Vec<PathBuf>, MyErrors> {
    let mut all_files = Vec::new();
    let entries = fs::read_dir(dir).map_err(|_| MyErrors::CannotReadDirectory)?;

    entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok().map(|meta| (entry.path(), meta)))
        .for_each(|(path, metadata)| {
            if metadata.is_dir() {
                if let Ok(mut res) = list_files(path) {
                    all_files.append(&mut res);
                }
            } else {
                all_files.push(path);
            }
        });

    all_files.sort();

    return Ok(all_files);
}

fn list_company_files(company: String) -> Result<Vec<PathBuf>, MyErrors> {
    let mut files = Vec::new();
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
                files.push(path)
            }
        }
    }

    Ok(files)
}

fn recent() -> Result<(), MyErrors> {
    let year = chrono::Utc::now().year();
    let folder = ["/tmp/interviews/", &year.to_string()].concat();
    let root = PathBuf::from(folder);

    if let Ok(all_files) = list_files(root) {
        let last_ten = all_files.iter().rev().take(10).rev();
        for entry in last_ten {
            println!("subl {:?}", entry);
        }
    }

    Ok(())
}

fn help() -> Result<(), MyErrors> {
    unimplemented!();
}

fn create() -> Result<(), MyErrors> {
    unimplemented!();
}
