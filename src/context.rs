//! The Claude Code statusline stdin JSON, parsed leniently.
//!
//! Compatibility contract: this tool must keep rendering across schema drift.
//! Every top-level field degrades to `None` on type mismatch instead of
//! failing the whole document, unknown fields are ignored, and the raw input
//! is preserved verbatim so custom modules receive exactly what Claude Code
//! sent.

use serde::{Deserialize, Deserializer, de::DeserializeOwned};

/// Deserializes a field to `None` on any shape/type mismatch instead of
/// erroring, isolating schema drift to the field where it happens.
fn lenient<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(T::deserialize(value).ok())
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Context {
    /// The stdin bytes exactly as received, for pass-through to custom modules.
    #[serde(skip)]
    pub raw: String,

    #[serde(deserialize_with = "lenient")]
    pub cwd: Option<String>,
    #[serde(deserialize_with = "lenient")]
    pub session_id: Option<String>,
    #[serde(deserialize_with = "lenient")]
    pub session_name: Option<String>,
    #[serde(deserialize_with = "lenient")]
    pub transcript_path: Option<String>,
    #[serde(deserialize_with = "lenient")]
    pub version: Option<String>,
    #[serde(deserialize_with = "lenient")]
    pub model: Option<Model>,
    #[serde(deserialize_with = "lenient")]
    pub workspace: Option<Workspace>,
    #[serde(deserialize_with = "lenient")]
    pub output_style: Option<OutputStyle>,
    #[serde(deserialize_with = "lenient")]
    pub cost: Option<Cost>,
    #[serde(deserialize_with = "lenient")]
    pub context_window: Option<ContextWindow>,
    #[serde(deserialize_with = "lenient")]
    pub exceeds_200k_tokens: Option<bool>,
    #[serde(deserialize_with = "lenient")]
    pub effort: Option<Effort>,
    #[serde(deserialize_with = "lenient")]
    pub thinking: Option<Thinking>,
    #[serde(deserialize_with = "lenient")]
    pub rate_limits: Option<RateLimits>,
    #[serde(deserialize_with = "lenient")]
    pub vim: Option<Vim>,
    #[serde(deserialize_with = "lenient")]
    pub agent: Option<Agent>,
    #[serde(deserialize_with = "lenient")]
    pub pr: Option<Pr>,
    #[serde(deserialize_with = "lenient")]
    pub worktree: Option<Worktree>,
}

impl Context {
    /// Parses the stdin JSON, falling back to an empty context on malformed
    /// input. Never fails: a statusline renders what it can.
    pub fn from_stdin_json(raw: String) -> Context {
        let mut ctx: Context = serde_json::from_str(&raw).unwrap_or_default();
        ctx.raw = raw;
        ctx
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Model {
    pub id: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Workspace {
    pub current_dir: Option<String>,
    pub project_dir: Option<String>,
    pub added_dirs: Vec<String>,
    pub git_worktree: Option<String>,
    pub repo: Option<Repo>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Repo {
    pub host: Option<String>,
    pub owner: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct OutputStyle {
    pub name: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Cost {
    pub total_cost_usd: Option<f64>,
    pub total_duration_ms: Option<u64>,
    pub total_api_duration_ms: Option<u64>,
    pub total_lines_added: Option<u64>,
    pub total_lines_removed: Option<u64>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct ContextWindow {
    pub total_input_tokens: Option<u64>,
    pub total_output_tokens: Option<u64>,
    pub context_window_size: Option<u64>,
    pub used_percentage: Option<f64>,
    pub remaining_percentage: Option<f64>,
    pub current_usage: Option<CurrentUsage>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct CurrentUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Effort {
    pub level: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Thinking {
    pub enabled: Option<bool>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct RateLimits {
    pub five_hour: Option<RateWindow>,
    pub seven_day: Option<RateWindow>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct RateWindow {
    pub used_percentage: Option<f64>,
    pub resets_at: Option<i64>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Vim {
    pub mode: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Agent {
    pub name: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Pr {
    pub number: Option<u64>,
    pub url: Option<String>,
    pub review_state: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Worktree {
    pub name: Option<String>,
    pub path: Option<String>,
    pub branch: Option<String>,
    pub original_cwd: Option<String>,
    pub original_branch: Option<String>,
}
