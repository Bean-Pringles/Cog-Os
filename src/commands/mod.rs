use crate::println;

pub mod echo;

pub fn run_command(line: &str) {
    let line = line.trim();

    let mut parts = line.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let args = parts.next().unwrap_or("");

    match cmd {
        "echo" => echo::run(args),
        _ => println!("Unknown command: {}", cmd),
    }
}