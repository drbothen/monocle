// semgrep-fixture: monocle-non-exhaustive-struct-audit-completeness
// This file exists ONLY as a semgrep fixture corpus target (SS-conventions §Semgrep Coverage Hardening).
// It is NOT part of the Rust workspace. Expected findings: 2 (one per pattern-either arm).
//
// Shape A — minimal: no intervening attribute (tests first pattern-either arm).
// Shape B — production-code shape: #[derive(...)] interposed between #[non_exhaustive]
//   and pub struct (tests second pattern-either arm). This shape mirrors every real
//   monocle production struct. See SS-conventions §Semgrep Coverage Hardening F-R32-2.

// Shape A: minimal form (no intervening attribute)
#[non_exhaustive]
pub struct AuditFixtureMinimal {
    pub field: u32,
}

// Shape B: production-code form (#[derive(...)] interposed)
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct AuditFixtureDerived {
    pub field: u32,
}
