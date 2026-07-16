#![allow(dead_code)]

use ccstatuskit::probes::{MemInfo, Probes};
use std::collections::HashMap;
use std::path::Path;

/// Deterministic probe stub: fixed clock, canned memory, canned command
/// outputs keyed by `"program arg1 arg2 …"`.
#[derive(Default)]
pub struct FakeProbes {
    pub hm: (u8, u8),
    pub mem: Option<MemInfo>,
    pub cmd: HashMap<String, String>,
}

impl FakeProbes {
    pub fn with_cmd(mut self, invocation: &str, stdout: &str) -> Self {
        self.cmd.insert(invocation.to_string(), stdout.to_string());
        self
    }
}

impl Probes for FakeProbes {
    fn local_hm(&self) -> (u8, u8) {
        self.hm
    }

    fn memory(&self) -> Option<MemInfo> {
        self.mem
    }

    fn run(
        &self,
        program: &str,
        args: &[&str],
        _envs: &[(&str, &str)],
        _cwd: Option<&Path>,
    ) -> Option<String> {
        let key = std::iter::once(program)
            .chain(args.iter().copied())
            .collect::<Vec<_>>()
            .join(" ");
        self.cmd.get(&key).cloned()
    }
}
