use std::env::args;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

use chrono::Datelike;

#[derive(Debug)]
pub enum MyErrors {
    FileNotFound,
    MissingCommand,
    FileAlreadyExists,
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
    let year = chrono::Local::now().year();
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
    let files = list_company_files(company)?;
    let path = match files.iter().last() {
        Some(res) => res,
        None => return Err(MyErrors::FileNotFound),
    };
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines().filter_map(|x| x.ok()) {
        if line.starts_with("Description: ") {
            notes.description = line.chars().skip(13).collect();
            continue;
        }

        if line.starts_with("Employment: ") {
            notes.employment = line.chars().skip(12).collect();
            continue;
        }

        if line.starts_with("Headquarters: ") {
            notes.headquarters = line.chars().skip(14).collect();
            continue;
        }

        if line.starts_with("Industry: ") {
            notes.industry = line.chars().skip(10).collect();
            continue;
        }

        if line.starts_with("TechStack: ") {
            notes.techstack = line.chars().skip(11).collect();
            continue;
        }

        if line.starts_with("Website: ") {
            notes.website = line.chars().skip(9).collect();
            continue;
        }
    }

    Ok(notes)
}

fn create() -> Result<(), MyErrors> {
    let now = chrono::Local::now();
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



--{boundary}
Author: me
Comment: invitation accepted
Content-Transfer-Encoding: quoted-printable
Content-Type: text/plain; charset=UTF-8
Date: {basic_date}

Thanks. Here’s my calendar: https://cixtor.com/calendar

--{boundary}
Author: them
Comment: job description
Content-Disposition: attachment; filename=\"job.md\"
Content-Transfer-Encoding: quoted-printable
Content-Type: text/markdown; charset=UTF-8



--{boundary}--
"
    );

    println!("{}", output);

    Ok(())
}
