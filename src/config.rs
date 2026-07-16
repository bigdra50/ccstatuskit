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
    pub usage: UsageConfig,
    pub custom: HashMap<String, CustomModuleConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ModuleOverrides {
    pub disabled: bool,
    pub style: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct UsageConfig {
    pub disabled: bool,
    pub style: Option<String>,
    /// Also fetch per-model ("scoped") quotas from the unofficial OAuth
    /// usage endpoint. Experimental; may break without notice.
    pub scoped: bool,
    /// Window filter: "five_hour"/"5h", "seven_day"/"wk", or a model-name
    /// substring like "fable". Empty shows everything available.
    pub only: Vec<String>,
    /// Cache freshness for scoped data, in seconds.
    pub cache_ttl: Option<u64>,
    /// Spawn a background refresh when the scoped cache is stale.
    pub auto_refresh: bool,
    /// Cache directory override (defaults to the XDG cache dir).
    pub cache_dir: Option<PathBuf>,
}

impl Default for UsageConfig {
    fn default() -> Self {
        UsageConfig {
            disabled: false,
            style: None,
            scoped: false,
            only: Vec::new(),
            cache_ttl: None,
            auto_refresh: true,
            cache_dir: None,
        }
    }
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

    /// The (disabled, style) pair every module shares, regardless of the
    /// module's own config shape.
    pub fn module_basics(&self, module: BuiltinModule) -> (bool, Option<&str>) {
        fn generic(o: &ModuleOverrides) -> (bool, Option<&str>) {
            (o.disabled, o.style.as_deref())
        }
        match module {
            BuiltinModule::Model => generic(&self.model),
            BuiltinModule::Directory => generic(&self.directory),
            BuiltinModule::Git => generic(&self.git),
            BuiltinModule::Memory => generic(&self.memory),
            BuiltinModule::Ctx => generic(&self.ctx),
            BuiltinModule::Time => generic(&self.time),
            BuiltinModule::Project => generic(&self.project),
            BuiltinModule::Usage => (self.usage.disabled, self.usage.style.as_deref()),
        }
    }
}

/// Default cache directory: `$XDG_CACHE_HOME/ccstatuskit` or `~/.cache/ccstatuskit`.
pub fn default_cache_dir() -> Option<PathBuf> {
    let cache_home = std::env::var_os("XDG_CACHE_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::home_dir().map(|home| home.join(".cache")))?;
    Some(cache_home.join("ccstatuskit"))
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
