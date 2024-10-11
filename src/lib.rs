//! Cargomon: A Rust implementation inspired by nodemon
//!
//! This crate provides functionality to watch for file changes in a Rust project,
//! automatically rebuild the project, and run the resulting executable.
//!
//! # Features
//!
//! - File watching: Monitors specified directories for changes
//! - Automatic rebuilding: Triggers a rebuild when changes are detected
//! - Executable running: Runs the built executable after a successful build
//! - Debouncing: Prevents multiple rebuilds for rapid successive file changes
//! - Colored output: Provides visually distinct console messages for better readability
//!
//! # Usage
//!
//! To use Cargomon in your project, add it as a dependency in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! cargomon = "0.1.0"
//! ```
//!
//! Then, in your `main.rs` file:
//!
//! ```no_run
//! fn main() {
//!     cargomon::run();
//! }
//! ```
//!
//! # Command-line Options
//!
//! Cargomon supports the following command-line options:
//!
//! - `--watch-path` or `-w`: Specifies the directory to watch for changes (default: ".")
//! - `--debounce-secs` or `-d`: Sets the debounce time in seconds (default: 2)
//!
//! Example usage:
//!
//! ```sh
//! cargo run -- --watch-path ./src --debounce-secs 5
//! ```

use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::process::Command;
use std::time::{Duration, Instant};
use std::io::{self, Write};
use std::path::PathBuf;
use colored::*;
use structopt::StructOpt;

/// Command-line options for Cargomon
#[derive(Debug, StructOpt)]
#[structopt(name = "cargomon", about = "A Rust implementation inspired by nodemon")]
struct Opt {
    /// The directory to watch for changes
    #[structopt(short, long, default_value = ".")]
    watch_path: String,

    /// The debounce time in seconds
    #[structopt(short, long, default_value = "2")]
    debounce_secs: u64,

    /// Display help information
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Display detailed help information
    Help,
}

/// Starts the Cargomon file watcher and build/run loop.
///
/// This function sets up a file watcher for the specified directory and its subdirectories.
/// When changes are detected, it rebuilds the project and runs the resulting executable.
///
/// # Behavior
///
/// 1. Watches the specified directory for file changes
/// 2. When changes are detected, waits for the debounce period
/// 3. Rebuilds the project using `cargo build`
/// 4. If the build is successful, runs the resulting executable
/// 5. Displays colored output for various stages and results
///
/// # Panics
///
/// This function will panic if:
/// - It fails to set up the file watcher
/// - It fails to execute `cargo build`
/// - It fails to run the built executable
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
    let opt = Opt::from_args();

    if let Some(Command::Help) = opt.cmd {
        display_help();
        return;
    }

    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

    watcher.watch(&opt.watch_path, RecursiveMode::Recursive).unwrap();

    println!("{}", "Watching for changes. Press Ctrl+C to exit.".green());

    let mut last_build_time = Instant::now();

    loop {
        match rx.recv() {
            Ok(_) => {
                if last_build_time.elapsed() < Duration::from_secs(opt.debounce_secs) {
                    continue;
                }
                last_build_time = Instant::now();

                println!("{}", "Change detected. Rebuilding...".yellow());
                
                let output = Command::new("cargo")
                    .arg("build")
                    .output()
                    .expect("Failed to execute cargo build");

                if output.status.success() {
                    println!("{}", "Build successful. Running the program...".green());
                    
                    let executable_path = find_executable();
                    
                    let run_output = Command::new(&executable_path)
                        .output()
                        .expect("Failed to run the program");

                    if run_output.status.success() {
                        io::stdout().write_all(&run_output.stdout).unwrap();
                        println!("{}", "Program executed successfully.".green());
                    } else {
                        io::stderr().write_all(&run_output.stderr).unwrap();
                        println!("{}", "Program execution failed.".red());
                    }
                } else {
                    println!("{}", "Build failed. Error output:".red());
                    io::stderr().write_all(&output.stderr).unwrap();
                }
                
                println!("\n{}", "Continuing to watch for changes...".green());
            }
            Err(e) => println!("{}", format!("Watch error: {:?}", e).red()),
        }
    }
}

fn display_help() {
    println!("{}", "Cargomon: A Rust implementation inspired by nodemon".green());
    println!("{}", "Usage: cargomon [OPTIONS] [SUBCOMMAND]".yellow());
    println!("\nOptions:");
    println!("  -w, --watch-path <PATH>    The directory to watch for changes (default: \".\")")
    println!("  -d, --debounce-secs <SECS> The debounce time in seconds (default: 2)")
    println!("  -h, --help                 Print help information");
    println!("  -V, --version              Print version information");
    println!("\nSubcommands:");
    println!("  help    Display this help message");
    println!("\nDescription:");
    println!("Cargomon watches your Rust project for file changes and automatically");
    println!("rebuilds and runs your application. It helps streamline the development");
    println!("process by eliminating the need to manually recompile and restart your");
    println!("application after each change.");
    println!("\nExamples:");
    println!("  cargomon");
    println!("  cargomon --watch-path ./src --debounce-secs 5");
    println!("  cargomon help");
}

fn find_executable() -> String {
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    let package_name = cargo_toml
        .lines()
        .find(|line| line.starts_with("name ="))
        .and_then(|line| line.split('=').nth(1))
        .map(|name| name.trim().trim_matches('"'))
        .expect("Failed to find package name in Cargo.toml");

    let mut path = PathBuf::from("target");
    path.push("debug");
    path.push(if cfg!(windows) {
        format!("{}.exe", package_name)
    } else {
        package_name.to_string()
    });

    path.to_str().expect("Failed to convert path to string").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_find_executable() {
        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create a mock Cargo.toml file
        let cargo_toml_path = temp_path.join("Cargo.toml");
        let mut cargo_toml = File::create(cargo_toml_path).unwrap();
        writeln!(cargo_toml, "[package]\nname = \"test_project\"").unwrap();

        // Change the current directory to the temporary directory
        std::env::set_current_dir(temp_path).unwrap();

        // Run the find_executable function
        let executable_path = find_executable();

        // Check the result
        let expected_path = if cfg!(windows) {
            String::from(r"target\debug\test_project.exe")
        } else {
            String::from("target/debug/test_project")
        };
        assert_eq!(executable_path, expected_path);
    }

    #[test]
    #[should_panic(expected = "Failed to read Cargo.toml")]
    fn test_find_executable_no_cargo_toml() {
        // Create a temporary directory without a Cargo.toml file
        let temp_dir = tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // This should panic because there's no Cargo.toml file
        find_executable();
    }

    #[test]
    #[should_panic(expected = "Failed to find package name in Cargo.toml")]
    fn test_find_executable_invalid_cargo_toml() {
        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create an invalid Cargo.toml file
        let cargo_toml_path = temp_path.join("Cargo.toml");
        let mut cargo_toml = File::create(cargo_toml_path).unwrap();
        writeln!(cargo_toml, "[package]\n# Missing name field").unwrap();

        // Change the current directory to the temporary directory
        std::env::set_current_dir(temp_path).unwrap();

        // This should panic because the Cargo.toml file is invalid
        find_executable();
    }

    #[test]
    fn test_opt_default_values() {
        let opt = Opt::from_iter(&["test"]);
        assert_eq!(opt.watch_path, ".");
        assert_eq!(opt.debounce_secs, 2);
    }

    #[test]
    fn test_opt_custom_values() {
        let opt = Opt::from_iter(&["test", "--watch-path", "./src", "--debounce-secs", "5"]);
        assert_eq!(opt.watch_path, "./src");
        assert_eq!(opt.debounce_secs, 5);
    }
}
