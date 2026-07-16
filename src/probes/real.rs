use super::{MemInfo, Probes};
use std::path::Path;
use std::process::Command;

pub struct RealProbes;

impl Probes for RealProbes {
    fn local_hm(&self) -> (u8, u8) {
        use chrono::Timelike;
        let now = chrono::Local::now();
        (now.hour() as u8, now.minute() as u8)
    }

    fn memory(&self) -> Option<MemInfo> {
        memory_impl(self)
    }

    fn run(
        &self,
        program: &str,
        args: &[&str],
        envs: &[(&str, &str)],
        cwd: Option<&Path>,
    ) -> Option<String> {
        let mut cmd = Command::new(program);
        cmd.args(args);
        for (key, value) in envs {
            cmd.env(key, value);
        }
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        let output = cmd.output().ok()?;
        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            None
        }
    }
}

#[cfg(target_os = "linux")]
fn memory_impl(_probes: &RealProbes) -> Option<MemInfo> {
    let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
    let field = |name: &str| -> Option<u64> {
        meminfo
            .lines()
            .find(|l| l.starts_with(name))?
            .split_whitespace()
            .nth(1)?
            .parse()
            .ok()
    };
    let total_kb = field("MemTotal:")?;
    let available_kb = field("MemAvailable:").or_else(|| field("MemFree:"))?;
    Some(MemInfo {
        used_kb: total_kb.saturating_sub(available_kb),
        total_kb,
    })
}

#[cfg(target_os = "macos")]
fn memory_impl(probes: &RealProbes) -> Option<MemInfo> {
    let total_bytes: u64 = probes
        .run("sysctl", &["-n", "hw.memsize"], &[], None)?
        .trim()
        .parse()
        .ok()?;
    // `memory_pressure` reports "System-wide memory free percentage: NN%".
    let pressure = probes.run("memory_pressure", &[], &[], None)?;
    let free_percent: u64 = pressure
        .lines()
        .find(|l| l.contains("memory free percentage:"))?
        .split_whitespace()
        .last()?
        .trim_end_matches('%')
        .parse()
        .ok()?;
    let total_kb = total_bytes / 1024;
    Some(MemInfo {
        used_kb: total_kb * (100 - free_percent.min(100)) / 100,
        total_kb,
    })
}

#[cfg(windows)]
fn memory_impl(_probes: &RealProbes) -> Option<MemInfo> {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        dwMemoryLoad: 0,
        ullTotalPhys: 0,
        ullAvailPhys: 0,
        ullTotalPageFile: 0,
        ullAvailPageFile: 0,
        ullTotalVirtual: 0,
        ullAvailVirtual: 0,
        ullAvailExtendedVirtual: 0,
    };
    // SAFETY: `status` is a properly initialized MEMORYSTATUSEX with dwLength set.
    if unsafe { GlobalMemoryStatusEx(&mut status) } == 0 {
        return None;
    }
    let total_kb = status.ullTotalPhys / 1024;
    let avail_kb = status.ullAvailPhys / 1024;
    Some(MemInfo {
        used_kb: total_kb.saturating_sub(avail_kb),
        total_kb,
    })
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
fn memory_impl(_probes: &RealProbes) -> Option<MemInfo> {
    None
}
