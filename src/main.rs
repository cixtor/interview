use std::env::args;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
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

fn latest_company_file(company: String) -> Result<PathBuf, MyErrors> {
    let query = ["-", &company.to_lowercase(), "."].concat();
    let root = PathBuf::from("/tmp/interviews");
    let mut stack = vec![root];
    let mut latest: Option<PathBuf> = None;

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
                    let ext_ok = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e == "md" || e == "eml")
                        .unwrap_or(false);
                    if !ext_ok {
                        continue;
                    }

                    let name_matches = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| name.contains(&query))
                        .unwrap_or(false);
                    if !name_matches {
                        continue;
                    }

                    if let Some(best) = &latest {
                        if path <= *best {
                            continue;
                        }
                    }

                    latest = Some(path);
                }
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }

    latest.ok_or(MyErrors::FileNotFound)
}

fn open() -> Result<(), MyErrors> {
    let company = get_command_option()?;
    let path = latest_company_file(company)?;

    let mut marker = 0;
    let mut boundary = String::from("--");
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        if let Ok(line) = line {
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
    }

    let file_arg = format!("{}:{}", path.display().to_string(), marker);

    Command::new("subl")
        .arg(file_arg)
        .spawn()
        .expect("cannot spawn subl");

    Ok(())
}

fn list() -> Result<(), MyErrors> {
    let company = get_command_option()?;
    let files = list_company_files(company)?;

    for file in files {
        println!("$EDITOR {:?}", file);
    }

    Ok(())
}

fn list_files(dir: PathBuf) -> Result<Vec<PathBuf>, MyErrors> {
    let mut all_files = Vec::new();
    let mut stack = vec![dir];

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

    all_files.sort();
    Ok(all_files)
}

fn list_company_files(company: String) -> Result<Vec<PathBuf>, MyErrors> {
    let mut files = Vec::new();
    let query = ["-", &company.to_lowercase(), "."].concat();
    let root = PathBuf::from("/tmp/interviews");

    let mut stack = vec![root];
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
                    let ext_ok = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e == "md" || e == "eml")
                        .unwrap_or(false);
                    if !ext_ok {
                        continue;
                    }

                    let name_matches = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| name.contains(&query))
                        .unwrap_or(false);
                    if name_matches {
                        files.push(path);
                    }
                }
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }

    files.sort();
    Ok(files)
}

fn recent() -> Result<(), MyErrors> {
    let year = chrono::Local::now().year();
    let folder = ["/tmp/interviews/", &year.to_string()].concat();
    let root = PathBuf::from(folder);
    let mut stack = vec![PathBuf::from(folder)];
    let mut heap: BinaryHeap<(Reverse<String>, PathBuf)> = BinaryHeap::new();

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
                    let key = path.to_string_lossy().into_owned();
                    heap.push((Reverse(key), path));
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
    last_ten.sort_by(|a, b| a.0.cmp(&b.0));

    for (_, entry) in last_ten {
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
    let length = CHARSET.len();
    let mut rng = rand::thread_rng();
    let text = (0..28)
        .map(|_| {
            let idx = rng.gen_range(0..length);
            CHARSET[idx] as char
        })
        .collect();
    return text;
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

fn previous_company_notes() -> Result<CompanyNotes, MyErrors> {
    let mut notes = CompanyNotes::new();
    let company = get_command()?;
    let path = latest_company_file(company)?;
    let file = File::open(&path).unwrap();
    let reader = BufReader::new(file);

    let mut remaining = 6;
    let mut description_seen = false;
    let mut employment_seen = false;
    let mut headquarters_seen = false;
    let mut industry_seen = false;
    let mut techstack_seen = false;
    let mut website_seen = false;

    for line in reader.lines().flatten() {
        if remaining == 0 {
            break;
        }

        if !description_seen {
            if let Some(value) = line.strip_prefix("Description: ") {
                notes.description = value.to_owned();
                description_seen = true;
                remaining -= 1;
                continue;
            }
        }

        if !employment_seen {
            if let Some(value) = line.strip_prefix("Employment: ") {
                notes.employment = value.to_owned();
                employment_seen = true;
                remaining -= 1;
                continue;
            }
        }

        if !headquarters_seen {
            if let Some(value) = line.strip_prefix("Headquarters: ") {
                notes.headquarters = value.to_owned();
                headquarters_seen = true;
                remaining -= 1;
                continue;
            }
        }

        if !industry_seen {
            if let Some(value) = line.strip_prefix("Industry: ") {
                notes.industry = value.to_owned();
                industry_seen = true;
                remaining -= 1;
                continue;
            }
        }

        if !techstack_seen {
            if let Some(value) = line.strip_prefix("TechStack: ") {
                notes.techstack = value.to_owned();
                techstack_seen = true;
                remaining -= 1;
                continue;
            }
        }

        if !website_seen {
            if let Some(value) = line.strip_prefix("Website: ") {
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
            let hour = text.replace("today@", "");
            text = format!("{}T{}", date, hour);
        }
        let tformat: String = match text.len() {
            16 => String::from("%Y-%m-%dT%H:%M"),    // 2006-01-02T15:04
            19 => String::from("%Y-%m-%dT%H:%M:%S"), // 2006-01-02T15:04:05
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
    let filename = format!(
        "/tmp/interviews/{}/{}-{}.eml",
        now.year(),
        shortdate,
        company_short
    );
    let file_arg = format!("{}:24", filename.clone());

    if Path::new(&filename).exists() {
        return Err(MyErrors::FileAlreadyExists);
    }

    let mut notes = CompanyNotes::new();
    let boundary = generate_boundary();

    // Attempt to fill common metadata from previous notes.
    if let Ok(prev_notes) = previous_company_notes() {
        notes = prev_notes;
    }

    // Define detached variables to allow string interpolation.
    let description = notes.description;
    let employment = notes.employment;
    let headquarters = notes.headquarters;
    let industry = notes.industry;
    let techstack = notes.techstack;
    let website = notes.website;

    let output = format!(
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
    );

    let mut file = match File::create(filename) {
        Ok(myfile) => myfile,
        Err(e) => return Err(MyErrors::CannotCreateFile(e)),
    };

    if let Err(e) = file.write_all(&output.as_bytes()) {
        return Err(MyErrors::CannotWriteToFile(e));
    }

    Command::new("subl")
        .arg(file_arg)
        .spawn()
        .expect("cannot spawn subl");

    Ok(())
}
