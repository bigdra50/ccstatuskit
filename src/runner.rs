//! Binds the engine's `ModuleRunner` to real builtin modules, applying
//! per-module config overrides (disabled, style).

use crate::config::Config;
use crate::context::Context;
use crate::engine::ModuleRunner;
use crate::format::ModuleRef;
use crate::modules::BuiltinModule;
use crate::probes::Probes;
use crate::segment::Segment;
use crate::style::parse_style;

pub struct BuiltinRunner {
    pub context: Context,
    pub config: Config,
    pub probes: Box<dyn Probes>,
}

impl ModuleRunner for BuiltinRunner {
    fn run(&self, module: &ModuleRef) -> Option<Segment> {
        match module {
            ModuleRef::Builtin(name) => {
                // Unknown module names hide instead of erroring, so a config
                // written for a newer ccstatuskit degrades gracefully.
                let builtin = BuiltinModule::from_name(name)?;
                let (disabled, style_spec) = self.config.module_basics(builtin);
                if disabled {
                    return None;
                }
                let mut segment =
                    builtin.render_with(&self.context, &self.config, self.probes.as_ref())?;
                if let Some(spec) = style_spec {
                    match parse_style(spec, &self.config.palette) {
                        Ok(style) => segment.style = style,
                        Err(err) => eprintln!("ccstatuskit: [{name}] {err}"),
                    }
                }
                Some(segment)
            }
            ModuleRef::Custom(name) => {
                let cfg = self.config.custom.get(name)?;
                let timeout =
                    std::time::Duration::from_millis(self.config.command_timeout.unwrap_or(250));
                crate::modules::custom::render(cfg, &self.context, &self.config.palette, timeout)
            }
        }
    }
}
