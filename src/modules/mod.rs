//! Builtin modules: a closed enum with static dispatch. Every renderer
//! returns `Option<Segment>` — `None` means hidden, and any missing input
//! hides the module rather than erroring (fail-soft).

mod ctx;
pub mod custom;
mod directory;
mod git;
mod memory;
mod model;
mod project;
mod time;
mod usage;

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
    Project,
    Usage,
}

impl BuiltinModule {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "model" => Self::Model,
            "directory" => Self::Directory,
            "git" => Self::Git,
            "memory" => Self::Memory,
            "ctx" => Self::Ctx,
            "time" => Self::Time,
            "project" => Self::Project,
            "usage" => Self::Usage,
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
            Self::Project => project::render(context, probes),
            Self::Usage => usage::render(context, config, probes),
        }
    }
}
