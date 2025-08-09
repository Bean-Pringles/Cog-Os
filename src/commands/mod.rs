// src/commands/mod.rs
pub mod echo;
pub mod delete;
pub mod list;
pub mod write;
pub mod create;
pub mod read;

pub fn run_command(line: &str) {
    let line = line.trim();
    let mut parts = line.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let args = parts.next().unwrap_or("");

    match cmd {
        "echo" => echo::run(args),
        "read" => read::run(args),
        "write" => write::run(args),
        "create" => create::run(args),
        "delete" => delete::run(args),
        "list" => list::run(args),
        _ => crate::println!("Unknown command: {}", cmd),
    }
}