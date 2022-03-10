use std::env::args;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

use chrono::Datelike;
use chrono::TimeZone;

#[derive(Debug)]
pub enum MyErrors {
    FileNotFound,
    MissingCommand,
    FileAlreadyExists,
    CannotReadDirectory,
    InvalidBoundaryLine,
    CannotCreateFile(std::io::Error),
    CannotWriteToFile(std::io::Error),
    CannotParseCustomDate(chrono::ParseError),
    CannotConvertToLocalTime,
    InvalidCustomDatetime,
}

fn main() -> Result<(), MyErrors> {
    let command = args().nth(1);

    match command.as_ref().map(String::as_ref) {
        Some("open") => open()?,
        Some("list") => list()?,
        Some("recent") => recent()?,
        Some("search") => search()?,
        Some("help") => help()?,
        None => help()?,
        _ => create()?,
    };

    Ok(())
}

fn get_command() -> Result<String, MyErrors> {
    let command = match args().nth(1) {
        Some(value) => value,
        None => return Err(MyErrors::MissingCommand),
    };
    Ok(command)
}

fn get_command_option() -> Result<String, MyErrors> {
    let option = match args().nth(2) {
        Some(value) => value,
        None => return Err(MyErrors::MissingCommand),
    };
    Ok(option)
}

fn latest_company_file(company: &str) -> Result<PathBuf, MyErrors> {
    let query = ["-", &company.to_lowercase(), "."].concat();
    let root = PathBuf::from("/tmp/interviews");
    let mut stack = Vec::with_capacity(32);
    stack.push(root);
    let mut latest: Option<(String, PathBuf)> = None;

    while let Some(current_dir) = stack.pop() {
        let entries = match fs::read_dir(&current_dir) {
            Ok(it) => it,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            if file_type.is_dir() {
                stack.push(entry.path());
                continue;
            }

            if !file_type.is_file() {
                continue;
            }

            let name_os = entry.file_name();
            let Some(name) = name_os.to_str() else {
                continue;
            };
            if !(name.ends_with(".md") || name.ends_with(".eml")) || !name.contains(&query) {
                continue;
            }

            if let Some((best_name, _)) = &latest {
                if name <= best_name.as_str() {
                    continue;
                }
            }

            latest = Some((name.to_owned(), entry.path()));
        }
    }

    latest.map(|(_, path)| path).ok_or(MyErrors::FileNotFound)
}

fn open() -> Result<(), MyErrors> {
    let company = get_command_option()?;
    let path = latest_company_file(&company)?;

    let mut marker = 0;
    let mut boundary = String::from("--");
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let mut index = 0;

    loop {
        line.clear();
        let bytes = match reader.read_line(&mut line) {
            Ok(n) => n,
            Err(_) => break,
        };
        if bytes == 0 {
            break;
        }

        let trimmed = line.trim_end_matches(['\r', '\n']);

        if boundary.len() > 2 && trimmed == boundary {
            marker = index;
        } else if trimmed.contains("; boundary=") {
            // Extract the email boundary code from the header.
            if let Some(mark) = trimmed.chars().position(|x| x == '=') {
                if let Some(value) = trimmed.get(mark + 1..) {
                    boundary.push_str(value);
                } else {
                    return Err(MyErrors::InvalidBoundaryLine);
                }
            } else {
                // Could not find an email boundary header anywhere.
                return Err(MyErrors::InvalidBoundaryLine);
            }
        }

        index += 1;
    }

    let file_arg = format!("{}:{}", path.display(), marker);

    Command::new("subl")
        .arg(file_arg)
        .spawn()
        .expect("cannot spawn subl");

    Ok(())
}

fn list() -> Result<(), MyErrors> {
    let company = get_command_option()?;
    let files = list_company_files(&company)?;

    for file in files {
        println!("$EDITOR {:?}", file);
    }

    Ok(())
}

fn list_files(dir: PathBuf) -> Result<Vec<PathBuf>, MyErrors> {
    let mut all_files = Vec::with_capacity(128);
    let mut stack = Vec::with_capacity(32);
    stack.push(dir);

    while let Some(current_dir) = stack.pop() {
        let entries = fs::read_dir(current_dir).map_err(|_| MyErrors::CannotReadDirectory)?;
        for entry in entries.flatten() {
            let path = entry.path();
            match entry.file_type() {
                Ok(ft) if ft.is_dir() => stack.push(path),
                Ok(ft) if ft.is_file() => all_files.push(path),
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }

    all_files.sort_unstable();
    Ok(all_files)
}

fn list_company_files(company: &str) -> Result<Vec<PathBuf>, MyErrors> {
    let mut files = Vec::with_capacity(64);
    let query = ["-", &company.to_lowercase(), "."].concat();
    let root = PathBuf::from("/tmp/interviews");

    let mut stack = Vec::with_capacity(32);
    stack.push(root);
    while let Some(current_dir) = stack.pop() {
        let entries = match fs::read_dir(&current_dir) {
            Ok(it) => it,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            if file_type.is_dir() {
                stack.push(entry.path());
                continue;
            }

            if !file_type.is_file() {
                continue;
            }

            let name_os = entry.file_name();
            let Some(name) = name_os.to_str() else {
                continue;
            };
            if !(name.ends_with(".md") || name.ends_with(".eml")) || !name.contains(&query) {
                continue;
            }

            files.push(entry.path());
        }
    }

    files.sort_unstable();
    Ok(files)
}

fn recent() -> Result<(), MyErrors> {
    let year = chrono::Local::now().year();
    let folder = ["/tmp/interviews/", &year.to_string()].concat();
    let mut stack = vec![PathBuf::from(folder)];
    let mut heap: BinaryHeap<Reverse<PathBuf>> = BinaryHeap::with_capacity(11);

    while let Some(current_dir) = stack.pop() {
        let entries = match fs::read_dir(&current_dir) {
            Ok(it) => it,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            match entry.file_type() {
                Ok(ft) if ft.is_dir() => stack.push(path),
                Ok(ft) if ft.is_file() => {
                    heap.push(Reverse(path));
                    if heap.len() > 10 {
                        heap.pop(); // drop oldest (smallest) path so heap keeps the most recent lexicographically
                    }
                }
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }

    let mut last_ten: Vec<_> = heap.into_iter().collect();
    last_ten.sort_unstable_by(|a, b| a.0.cmp(&b.0));

    for Reverse(entry) in last_ten {
        println!("$EDITOR {:?}", entry);
    }

    Ok(())
}

fn search() -> Result<(), MyErrors> {
    Ok(())
}

fn help() -> Result<(), MyErrors> {
    println!("Usage:");
    println!("  interview [command] [options]");
    println!("");
    println!("Example:");
    println!("  interview [company]         Creates a new .eml file for Company");
    println!("  interview [company] [date]  Same as above but on a specific date");
    println!("  interview help              Prints this message");
    println!("  interview list [company]    Prints all .eml files for Company");
    println!("  interview open [company]    Opens the most recent .eml file for Company");
    println!("  interview recent            Prints the most recent ten files");
    println!("  interview search [query]    Prints files that contain the query");

    Ok(())
}

fn generate_boundary() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"0123456789abcdef";
    let mut rng = rand::thread_rng();
    let mut text = String::with_capacity(28);

    for _ in 0..28 {
        let idx = rng.gen_range(0..CHARSET.len());
        text.push(CHARSET[idx] as char);
    }

    text
}

#[derive(Debug)]
struct CompanyNotes {
    description: String,
    employment: String,
    headquarters: String,
    industry: String,
    techstack: String,
    website: String,
}

impl CompanyNotes {
    fn new() -> CompanyNotes {
        return CompanyNotes {
            description: String::new(),
            employment: String::from("fulltime, on-site, CITY"),
            headquarters: String::from("CITY, STATE, COUNTRY"),
            industry: String::new(),
            techstack: String::new(),
            website: String::from("URL"),
        };
    }
}

fn previous_company_notes(company: &str) -> Result<CompanyNotes, MyErrors> {
    let mut notes = CompanyNotes::new();
    let path = latest_company_file(company)?;
    let file = File::open(&path).unwrap();
    let mut reader = BufReader::new(file);
    let mut line = String::new();

    let mut remaining = 6;
    let mut description_seen = false;
    let mut employment_seen = false;
    let mut headquarters_seen = false;
    let mut industry_seen = false;
    let mut techstack_seen = false;
    let mut website_seen = false;

    loop {
        line.clear();
        let bytes = match reader.read_line(&mut line) {
            Ok(n) => n,
            Err(_) => break,
        };
        if bytes == 0 {
            break;
        }

        let trimmed = line.trim_end_matches(['\r', '\n']);

        if remaining == 0 {
            break;
        }

        if !description_seen {
            if let Some(value) = trimmed.strip_prefix("Description: ") {
                notes.description = value.to_owned();
                description_seen = true;
                remaining -= 1;
                continue;
            }
        }

        if !employment_seen {
            if let Some(value) = trimmed.strip_prefix("Employment: ") {
                notes.employment = value.to_owned();
                employment_seen = true;
                remaining -= 1;
                continue;
            }
        }

        if !headquarters_seen {
            if let Some(value) = trimmed.strip_prefix("Headquarters: ") {
                notes.headquarters = value.to_owned();
                headquarters_seen = true;
                remaining -= 1;
                continue;
            }
        }

        if !industry_seen {
            if let Some(value) = trimmed.strip_prefix("Industry: ") {
                notes.industry = value.to_owned();
                industry_seen = true;
                remaining -= 1;
                continue;
            }
        }

        if !techstack_seen {
            if let Some(value) = trimmed.strip_prefix("TechStack: ") {
                notes.techstack = value.to_owned();
                techstack_seen = true;
                remaining -= 1;
                continue;
            }
        }

        if !website_seen {
            if let Some(value) = trimmed.strip_prefix("Website: ") {
                notes.website = value.to_owned();
                website_seen = true;
                remaining -= 1;
                continue;
            }
        }
    }

    Ok(notes)
}

fn read_custom_date() -> Result<chrono::DateTime<chrono::Local>, MyErrors> {
    if let Ok(mut text) = get_command_option() {
        if text.len() == 11 && text.starts_with("today@") {
            // Support shorthand data inputs: today@15:04
            let date = chrono::Local::now().format("%Y-%m-%d");
            let hour = &text[6..];
            text = format!("{}T{}", date, hour);
        }
        let tformat: &str = match text.len() {
            16 => "%Y-%m-%dT%H:%M",    // 2006-01-02T15:04
            19 => "%Y-%m-%dT%H:%M:%S", // 2006-01-02T15:04:05
            _ => return Err(MyErrors::InvalidCustomDatetime), // anything else
        };
        let naive = match chrono::NaiveDateTime::parse_from_str(&text, &tformat) {
            Ok(value) => value,
            Err(e) => return Err(MyErrors::CannotParseCustomDate(e)),
        };
        let custom_time = match chrono::Local.from_local_datetime(&naive).single() {
            Some(value) => value,
            None => return Err(MyErrors::CannotConvertToLocalTime),
        };
        return Ok(custom_time);
    }

    Ok(chrono::Local::now())
}

fn create() -> Result<(), MyErrors> {
    let now = read_custom_date()?;
    let company = get_command()?;
    let shortdate = now.format("%Y%m%dT%H%M%S");
    let basic_date = now.format("%Y-%m-%dT%H:%M:%SZ");
    let human_date = now.format("%a, %d %b %Y %H:%M:%S %z");
    let company_short = company.replace(" ", "-").to_lowercase();
    let filename = format!("/tmp/interviews/{}/{}-{}.eml", now.year(), shortdate, company_short);
    let file_arg = format!("{}:24", filename);

    let mut notes = CompanyNotes::new();
    let boundary = generate_boundary();

    // Attempt to fill common metadata from previous notes.
    if let Ok(prev_notes) = previous_company_notes(&company) {
        notes = prev_notes;
    }

    // Define detached variables to allow string interpolation.
    let description = notes.description;
    let employment = notes.employment;
    let headquarters = notes.headquarters;
    let industry = notes.industry;
    let techstack = notes.techstack;
    let website = notes.website;

    let file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&filename)
    {
        Ok(myfile) => myfile,
        Err(e) => return Err(MyErrors::CannotCreateFile(e)),
    };
    let mut writer = BufWriter::new(file);

    if let Err(e) = write!(
        writer,
        "MIME-Version: 1.0
Date: {human_date}
Message-ID: <{shortdate}-{company_short}@local.test>
Subject: {company}, Software Engineer
From: jobs@{company_short}.com
To: cixtor@linkedin.test
Content-Type: multipart/mixed; boundary={boundary}
Description: {description}
Employment: {employment}
Headquarters: {headquarters}
Industry: {industry}
JobPostURL: URL
Salary: Unknown|USD $0-$999999
TechStack: {techstack}
Website: {website}

--{boundary}
Author: them
Comment: invitation received
Content-Transfer-Encoding: quoted-printable
Content-Type: text/plain; charset=UTF-8
Date: {basic_date}

...

--{boundary}
Author: me
Comment: shared resume
Content-Transfer-Encoding: quoted-printable
Content-Type: text/plain; charset=UTF-8
Date: {basic_date}

/shared resume/

--{boundary}
Author: them
Comment: job description
Content-Disposition: attachment; filename=\"job.md\"
Content-Transfer-Encoding: quoted-printable
Content-Type: text/markdown; charset=UTF-8



--{boundary}--
"
    ) {
        return Err(MyErrors::CannotWriteToFile(e));
    }

    if let Err(e) = writer.flush() {
        return Err(MyErrors::CannotWriteToFile(e));
    }

    Command::new("subl")
        .arg(file_arg)
        .spawn()
        .expect("cannot spawn subl");

    Ok(())
}
