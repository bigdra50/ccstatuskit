//! Builtin modules: a closed enum with static dispatch. Every renderer
//! returns `Option<Segment>` — `None` means hidden, and any missing input
//! hides the module rather than erroring (fail-soft).

mod agent;
mod cost;
mod ctx;
pub mod custom;
mod directory;
mod dotnet;
mod git;
mod go;
mod java;
mod kotlin;
pub(crate) mod lang_support;
mod lines;
mod memory;
mod model;
mod node;
mod php;
mod pr;
mod python;
mod ruby;
mod rust_lang;
mod swift;
mod time;
mod unity;
mod usage;
mod worktree;

use crate::config::Config;
use crate::context::Context;
use crate::probes::Probes;
use crate::segment::Segment;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinModule {
    Model,
    Directory,
    Git,
    Memory,
    Ctx,
    Time,
    Usage,
    Cost,
    Lines,
    Agent,
    Pr,
    Worktree,
    Unity,
    Node,
    Rust,
    Go,
    Python,
    Dotnet,
    Ruby,
    Java,
    Kotlin,
    Php,
    Swift,
}

impl BuiltinModule {
    pub const ALL: [BuiltinModule; 23] = [
        Self::Model,
        Self::Directory,
        Self::Git,
        Self::Memory,
        Self::Ctx,
        Self::Time,
        Self::Usage,
        Self::Cost,
        Self::Lines,
        Self::Agent,
        Self::Pr,
        Self::Worktree,
        Self::Unity,
        Self::Node,
        Self::Rust,
        Self::Go,
        Self::Python,
        Self::Dotnet,
        Self::Ruby,
        Self::Java,
        Self::Kotlin,
        Self::Php,
        Self::Swift,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Directory => "directory",
            Self::Git => "git",
            Self::Memory => "memory",
            Self::Ctx => "ctx",
            Self::Time => "time",
            Self::Usage => "usage",
            Self::Cost => "cost",
            Self::Lines => "lines",
            Self::Agent => "agent",
            Self::Pr => "pr",
            Self::Worktree => "worktree",
            Self::Unity => "unity",
            Self::Node => "node",
            Self::Rust => "rust",
            Self::Go => "go",
            Self::Python => "python",
            Self::Dotnet => "dotnet",
            Self::Ruby => "ruby",
            Self::Java => "java",
            Self::Kotlin => "kotlin",
            Self::Php => "php",
            Self::Swift => "swift",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "model" => Self::Model,
            "directory" => Self::Directory,
            "git" => Self::Git,
            "memory" => Self::Memory,
            "ctx" => Self::Ctx,
            "time" => Self::Time,
            "usage" => Self::Usage,
            "cost" => Self::Cost,
            "lines" => Self::Lines,
            "agent" => Self::Agent,
            "pr" => Self::Pr,
            "worktree" => Self::Worktree,
            "unity" => Self::Unity,
            "node" => Self::Node,
            "rust" => Self::Rust,
            "go" => Self::Go,
            "python" => Self::Python,
            "dotnet" => Self::Dotnet,
            "ruby" => Self::Ruby,
            "java" => Self::Java,
            "kotlin" => Self::Kotlin,
            "php" => Self::Php,
            "swift" => Self::Swift,
            _ => return None,
        })
    }

    /// Renders with a default config. Only `usage` reads module-specific
    /// config; use [`BuiltinModule::render_with`] when it matters.
    pub fn render(self, context: &Context, probes: &dyn Probes) -> Option<Segment> {
        self.render_with(context, &Config::default(), probes)
    }

    pub fn render_with(
        self,
        context: &Context,
        config: &Config,
        probes: &dyn Probes,
    ) -> Option<Segment> {
        match self {
            Self::Model => model::render(context, probes),
            Self::Directory => directory::render(context, probes),
            Self::Git => git::render(context, probes),
            Self::Memory => memory::render(context, probes),
            Self::Ctx => ctx::render(context, probes),
            Self::Time => time::render(context, probes),
            Self::Usage => usage::render(context, config, probes),
            Self::Cost => cost::render(context, probes),
            Self::Lines => lines::render(context, probes),
            Self::Agent => agent::render(context, probes),
            Self::Pr => pr::render(context, probes),
            Self::Worktree => worktree::render(context, probes),
            Self::Unity => unity::render(context, probes),
            Self::Node => node::render(context, probes),
            Self::Rust => rust_lang::render(context, probes),
            Self::Go => go::render(context, probes),
            Self::Python => python::render(context, probes),
            Self::Dotnet => dotnet::render(context, probes),
            Self::Ruby => ruby::render(context, probes),
            Self::Java => java::render(context, probes),
            Self::Kotlin => kotlin::render(context, probes),
            Self::Php => php::render(context, probes),
            Self::Swift => swift::render(context, probes),
        }
    }
}
