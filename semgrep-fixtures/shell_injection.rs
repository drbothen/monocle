// semgrep-fixture: monocle-no-shell-injection
// This file exists ONLY as a semgrep fixture corpus target (SS-conventions §Semgrep Coverage Hardening).
// It is NOT part of the Rust workspace. Expected findings: 2 (one per pattern-either arm).

use std::process::Command;

fn fixture_arm_1_sh() {
    // Arm 1: Command::new("sh")
    let _ = Command::new("sh").arg("-c").arg("echo hi");
}

fn fixture_arm_2_bash() {
    // Arm 2: Command::new("bash")
    let _ = Command::new("bash").arg("-c").arg("echo hi");
}
