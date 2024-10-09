use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::process::Command;
use std::time::Duration;
use std::path::Path;

pub fn run() {
    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();

    // Watch the current directory recursively
    watcher.watch(".", RecursiveMode::Recursive).unwrap();

    println!("Watching for changes. Press Ctrl+C to exit.");

    loop {
        match rx.recv() {
            Ok(_) => {
                println!("Change detected. Rebuilding and running...");
                
                // Run cargo build
                let build_status = Command::new("cargo")
                    .arg("build")
                    .status()
                    .expect("Failed to execute cargo build");

                if build_status.success() {
                    println!("Build successful. Running the program...");
                    
                    // Find the executable
                    let executable_path = find_executable();
                    
                    // Run the executable
                    let run_status = Command::new(executable_path)
                        .status()
                        .expect("Failed to run the program");

                    if run_status.success() {
                        println!("Program executed successfully.");
                    } else {
                        println!("Program execution failed.");
                    }
                } else {
                    println!("Build failed.");
                }
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
