//! Side-effect boundary. Modules read the OS only through this trait so
//! tests can inject deterministic fakes.

mod real;

pub use real::RealProbes;

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemInfo {
    pub used_kb: u64,
    pub total_kb: u64,
}

pub trait Probes: Send + Sync {
    /// Local wall-clock time as (hour, minute).
    fn local_hm(&self) -> (u8, u8);

    /// System memory usage, if the platform exposes it.
    fn memory(&self) -> Option<MemInfo>;

    /// Runs a command and returns its stdout when it exits 0.
    fn run(
        &self,
        program: &str,
        args: &[&str],
        envs: &[(&str, &str)],
        cwd: Option<&Path>,
    ) -> Option<String>;
}
