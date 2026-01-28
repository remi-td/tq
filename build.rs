use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the output directory (where the binary will be built)
    let out_dir = env::var("OUT_DIR")?;
    let target_dir = PathBuf::from(&out_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .ok_or("Failed to determine target directory from OUT_DIR")?
        .to_path_buf();

    // Determine the library name based on platform
    let lib_name = if cfg!(target_os = "macos") {
        "teradatasql.dylib"
    } else if cfg!(target_os = "windows") {
        "teradatasql.dll"
    } else {
        "teradatasql.so"
    };

    // Find the teradatarustapi checkout in cargo cache
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| "Could not determine home directory (HOME or USERPROFILE not set)")?;
    let cargo_home = env::var("CARGO_HOME").unwrap_or_else(|_| format!("{}/.cargo", home));
    let git_checkouts = PathBuf::from(cargo_home).join("git/checkouts");

    // Search for teradatarustapi checkout
    if let Ok(entries) = fs::read_dir(&git_checkouts) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir()
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.starts_with("teradatarustapi-"))
                    .unwrap_or(false)
            {
                // Found the teradatarustapi directory, now find the actual checkout
                if let Ok(subdirs) = fs::read_dir(&path) {
                    for subdir in subdirs.filter_map(Result::ok) {
                        let lib_source = subdir.path().join(lib_name);
                        if lib_source.exists() {
                            // Copy the library to the target directory
                            let lib_dest = target_dir.join(lib_name);
                            if let Err(e) = fs::copy(&lib_source, &lib_dest) {
                                println!(
                                    "cargo:warning=Failed to copy {} to target: {}",
                                    lib_name, e
                                );
                            } else {
                                // Successfully copied - no warning needed for success case
                                // Sprint 28: Removed success warning to provide clean REPL startup

                                // Write the library directory to a file that can be included at compile time
                                let lib_dir_file =
                                    PathBuf::from(&out_dir).join("teradata_lib_dir.txt");
                                fs::write(&lib_dir_file, target_dir.to_string_lossy().as_bytes())
                                    .ok();

                                println!(
                                    "cargo:rustc-env=TERADATA_LIB_DIR={}",
                                    target_dir.display()
                                );
                                println!("cargo:rerun-if-changed=build.rs");
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }

    println!("cargo:warning=Could not find teradatasql library in cargo cache");
    println!(
        "cargo:warning=You may need to manually copy {} to your project directory",
        lib_name
    );

    Ok(())
}
