// semgrep-fixture: monocle-no-raw-env-mutation-in-tests
// This file exists ONLY as a semgrep fixture corpus target (SS-conventions §Semgrep Coverage Hardening).
// It is NOT part of the Rust workspace. Expected findings: 4 (one per pattern-either arm).
// File is placed under semgrep-fixtures/tests/ to match the paths.include pattern "**/tests/**/*.rs".

use std::env;

fn fixture_arm_1_std_env_set_var() {
    // Arm 1: std::env::set_var
    std::env::set_var("HOME", "/tmp");
}

fn fixture_arm_2_std_env_remove_var() {
    // Arm 2: std::env::remove_var
    std::env::remove_var("HOME");
}

fn fixture_arm_3_env_set_var() {
    // Arm 3: env::set_var (use-alias form)
    env::set_var("HOME", "/tmp");
}

fn fixture_arm_4_env_remove_var() {
    // Arm 4: env::remove_var (use-alias form)
    env::remove_var("HOME");
}
