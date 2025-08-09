// src/commands/mod.rs
pub mod create;
pub mod delete;
pub mod write;
pub mod read;
pub mod list;

pub fn run_command(cmd: &str, args: &[&str]) {
    match cmd {
        "create" => create::run(args),
        "delete" => delete::run(args),
        "write" => write::run(args),
        "read" => read::run(args),
        "list" => list::run(args),
        _ => crate::println!("Unknown command: {}", cmd),
    }
}