use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[test]
fn test_cargomon_detects_changes() {
    // Start cargomon in a separate thread
    thread::spawn(|| {
        cargomon::run();
    });

    // Give cargomon some time to start
    thread::sleep(Duration::from_secs(2));

    // Create a dummy file
    let mut file = File::create("dummy.rs").unwrap();
    file.write_all(b"fn main() { println!(\"Hello, World!\"); }").unwrap();

    // Give cargomon some time to detect the change
    thread::sleep(Duration::from_secs(5));

    // Check if the file was compiled
    assert!(std::path::Path::new("target/debug/dummy").exists());

    // Clean up
    std::fs::remove_file("dummy.rs").unwrap();
}

#[test]
fn test_find_executable() {
    // This test assumes that the current crate is named "cargomon"
    let executable_path = cargomon::find_executable();
    assert_eq!(executable_path, "target/debug/cargomon");
}
