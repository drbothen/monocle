//! Integration tests for `HomeUnresolvable` error path — VP-021 (S-015).
//!
//! Covers AC-005, AC-006: `metadata()` and `enrich()` return
//! `Err(EngineMetadataError::HomeUnresolvable)` and log `E-ENG-001` when all four
//! home-env vars (`HOME`, `USERPROFILE`, `HOMEDRIVE`, `HOMEPATH`) are unset.
//! Uses `temp-env 0.3` `async_with_vars` for env isolation.
//! Tests written by test-writer agent; bodies are placeholders pending RED phase.
