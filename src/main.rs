use std::env::args;

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

fn recent() {
    unimplemented!();
}

fn help() {
    unimplemented!();
}

fn create() {
    unimplemented!();
}
