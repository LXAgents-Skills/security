//! Who this server is: the facts every surface reports about itself, in one place.

/// The server's ID, as `initialize` and `/healthz` report it.
pub const SERVER_ID: &str = "lxagents-security";

/// The server's human-readable title.
pub const SERVER_TITLE: &str = "LXAgents Security";

/// The version, from `Cargo.toml`. There is no second place it is written.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The prefix every tool ID carries: `{org_name}_{repo_name}`, underscores only.
///
/// A client that connects several servers sees one flat tool list, so a bare name such as
/// `skill` or `automation` could collide with another server's. The prefix makes every ID
/// unique to this repository.
pub const ID_PREFIX: &str = "lxagents_skills_security";

/// What `initialize` tells a client about using this server.
pub const SERVER_INSTRUCTIONS: &str = "The global security set for python, javascript/typescript, and go, served read-only. Every file in the set is its own tool. Start at lxagents_skills_security_automation: it lists every other tool and the one condition that activates each, so you call only the guides the request needs. lxagents_skills_security_skill names the language and framework workflow. Read the general guide for the language before the framework-specific one - they are written to be read in that order. No tool takes an argument.";
