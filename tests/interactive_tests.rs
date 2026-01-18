use expectrl::spawn;
use std::time::Duration;

#[test]
fn test_repl_help_command() {
    // Get the path to the binary
    let bin_path = assert_cmd::cargo::cargo_bin("tq");

    // Spawn the process in a pseudo-terminal
    // We rely on the .env file for connection details
    // We disable syntax highlighting and paging to avoid ANSI codes and scrolling issues
    let cmd = format!(
        "{} repl --no-syntax-highlight --no-pager",
        bin_path.display()
    );

    let mut p = spawn(cmd).expect("Failed to spawn tq");

    // Set a timeout to avoid hanging if something goes wrong
    p.set_expect_timeout(Some(Duration::from_secs(10)));

    // Expect the banner first to verify startup
    p.expect("Connected to").expect("Failed to find banner");
    
    // Wait a bit for initialization
    std::thread::sleep(Duration::from_secs(1));

    // Send quit command
    p.send_line("/quit").expect("Failed to send quit");

    // Expect exit message
    p.expect("Goodbye!").expect("Failed to find exit message");
}
