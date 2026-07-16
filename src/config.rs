//! User configuration: `$XDG_CONFIG_HOME/ccstatuskit/config.toml`, overridable
//! with the `CCSTATUSKIT_CONFIG` env var. A missing or broken config falls
//! back to defaults — configuration mistakes must never blank the statusline.

use crate::modules::BuiltinModule;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

pub const DEFAULT_FORMAT: &str = "$model $directory $memory $ctx $time $git\n$project $usage";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub format: Option<String>,
    /// Global module budget in milliseconds.
    pub command_timeout: Option<u64>,
    /// Named colors usable in any style string.
    pub palette: HashMap<String, String>,
    pub model: ModuleOverrides,
    pub directory: ModuleOverrides,
    pub git: ModuleOverrides,
    pub memory: ModuleOverrides,
    pub ctx: ModuleOverrides,
    pub time: ModuleOverrides,
    pub project: ModuleOverrides,
    pub usage: ModuleOverrides,
    pub custom: HashMap<String, CustomModuleConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ModuleOverrides {
    pub disabled: bool,
    pub style: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CustomModuleConfig {
    pub command: String,
    /// Shell to run the command with; defaults per OS.
    pub shell: Option<Vec<String>>,
    /// Only run when one of these paths exists under the project directory.
    pub when_dir: Vec<String>,
    pub style: Option<String>,
    pub disabled: bool,
}

impl Config {
    pub fn load() -> Config {
        let Some(path) = config_path() else {
            return Config::default();
        };
        let Ok(body) = std::fs::read_to_string(&path) else {
            return Config::default();
        };
        toml::from_str(&body).unwrap_or_else(|err| {
            eprintln!(
                "ccstatuskit: ignoring broken config {}: {err}",
                path.display()
            );
            Config::default()
        })
    }

    pub fn format(&self) -> &str {
        self.format.as_deref().unwrap_or(DEFAULT_FORMAT)
    }

    pub fn overrides(&self, module: BuiltinModule) -> &ModuleOverrides {
        match module {
            BuiltinModule::Model => &self.model,
            BuiltinModule::Directory => &self.directory,
            BuiltinModule::Git => &self.git,
            BuiltinModule::Memory => &self.memory,
            BuiltinModule::Ctx => &self.ctx,
            BuiltinModule::Time => &self.time,
            BuiltinModule::Project => &self.project,
            BuiltinModule::Usage => &self.usage,
        }
    }
}

fn config_path() -> Option<PathBuf> {
    if let Some(explicit) = std::env::var_os("CCSTATUSKIT_CONFIG") {
        return Some(PathBuf::from(explicit));
    }
    let config_home = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::home_dir().map(|home| home.join(".config")))?;
    Some(config_home.join("ccstatuskit/config.toml"))
}
