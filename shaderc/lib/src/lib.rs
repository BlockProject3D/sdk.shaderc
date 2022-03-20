use std::error::Error;
use phf::phf_map;

mod targets;
mod config;

pub use config::*;

type TargetFunc = fn(Config) -> Result<(), Box<dyn Error>>;

static TARGETS: phf::Map<&'static str, TargetFunc> = phf_map! {
    "LIB" => targets::lib::build,
    "GL40" => targets::gl40::build,
    "GL42" => targets::gl42::build
};

pub struct Compiler
{
    func: TargetFunc
}

impl Compiler {
    /// Returns an instance of a compiler, None if no compiler exists for the required target.
    pub fn get(target_name: &str) -> Option<Compiler> {
        TARGETS.get(target_name).map(|v| Compiler { func: *v })
    }

    /// List available compiler targets.
    pub fn list_targets() -> impl Iterator<Item = &'static str> {
        TARGETS.keys().map(|v| *v)
    }

    /// Run the compiler with the given config.
    pub fn run(&self, config: Config) -> Result<(), Box<dyn Error>> {
        (self.func)(config)
    }
}
