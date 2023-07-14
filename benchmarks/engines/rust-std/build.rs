use std::process::Command;

use regex_lite::Regex;

fn main() {
    let version = rustc_version().unwrap_or("unknown".to_string());
    println!("cargo:rustc-env=RUSTC_VERSION={}", version);
}

fn rustc_version() -> Option<String> {
    let re = Regex::new(
        r"^rustc (?<version>\d+\.\d+\.\d+(?:-\S+)) \((?<rev>[0-9a-f]+)",
    )
    .unwrap();

    let output = Command::new("rustc").arg("--version").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let first_line = stdout.lines().next()?;
    let caps = re.captures(first_line)?;
    Some(format!("{} {}", &caps["version"], &caps["rev"]))
}
