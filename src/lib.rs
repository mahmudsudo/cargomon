//! Cargomon: A Rust implementation inspired by nodemon
//!
//! This crate provides functionality to watch for file changes in a Rust project,
//! automatically rebuild the project, and run the resulting executable.

use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::process::Command;
use std::time::Duration;
use std::io::{self, Write};

/// Starts the Cargomon file watcher and build/run loop.
///
/// This function sets up a file watcher for the current directory and its subdirectories.
/// When changes are detected, it rebuilds the project and runs the resulting executable.
///
/// # Examples
///
/// ```no_run
/// // In your main.rs file:
/// fn main() {
///     cargomon::run();
/// }
/// ```
pub fn run() {
    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();

    // Watch the current directory recursively
    watcher.watch(".", RecursiveMode::Recursive).unwrap();

    println!("Watching for changes. Press Ctrl+C to exit.");

    loop {
        match rx.recv() {
            Ok(_) => {
                println!("Change detected. Rebuilding...");
                
                // Run cargo build
                let output = Command::new("cargo")
                    .arg("build")
                    .output()
                    .expect("Failed to execute cargo build");

                if output.status.success() {
                    println!("Build successful. Running the program...");
                    
                    // Find the executable
                    let executable_path = find_executable();
                    
                    // Run the executable
                    let run_output = Command::new(executable_path)
                        .output()
                        .expect("Failed to run the program");

                    if run_output.status.success() {
                        io::stdout().write_all(&run_output.stdout).unwrap();
                        println!("Program executed successfully.");
                    } else {
                        io::stderr().write_all(&run_output.stderr).unwrap();
                        println!("Program execution failed.");
                    }
                } else {
                    println!("Build failed. Error output:");
                    io::stderr().write_all(&output.stderr).unwrap();
                }
                
                println!("\nContinuing to watch for changes...");
            }
            Err(e) => println!("Watch error: {:?}", e),
        }
    }
}

fn find_executable() -> String {
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    let package_name = cargo_toml
        .lines()
        .find(|line| line.starts_with("name ="))
        .and_then(|line| line.split('=').nth(1))
        .map(|name| name.trim().trim_matches('"'))
        .expect("Failed to find package name in Cargo.toml");

    format!("target/debug/{}", package_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_executable() {
        // This test assumes that the current crate is named "cargomon"
        let executable_path = find_executable();
        assert_eq!(executable_path, "target/debug/cargomon");
    }
}
