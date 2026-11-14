//! How a served file is named as a tool. Pure: a path in, an ID out.

use crate::identity::ID_PREFIX;

/// The longest ID this repository publishes.
///
/// The protocol admits 128 characters. Several clients cap a tool name at 64, so every ID here
/// is kept to 64, and a base name that would push an ID past it is shortened in
/// [`NAME_OVERRIDES`]. Startup refuses an ID over it.
pub const MAX_PUBLISHED_ID_LENGTH: usize = 64;

/// IDs that do not survive mechanical derivation, keyed by set-relative path, as the base name
/// that follows the prefix.
///
/// Everything else is named after its own filename, so adding a file to `content/` adds the tool
/// that serves it without anyone deciding what to call it. Only the base name is ever shortened:
/// `javascript_typescript` becomes `js_ts` and `javascript` becomes `js`, so that four guides fit
/// in 64 characters. The owner chose that on 2026-11-11.
///
/// `AGENTS.md` would derive to `agents`, which says nothing about which document it is; no such
/// file is in the set today, and the override is kept for when one is.
pub const NAME_OVERRIDES: &[(&str, &str)] = &[
    ("AGENTS.md", "agents_entry_point"),
    (
        "references/javascript-general-web-frontend-security.md",
        "js_general_web_frontend_security",
    ),
    (
        "references/javascript-typescript-nextjs-web-server-security.md",
        "js_ts_nextjs_web_server_security",
    ),
    (
        "references/javascript-typescript-react-web-frontend-security.md",
        "js_ts_react_web_frontend_security",
    ),
    (
        "references/javascript-typescript-vue-web-frontend-security.md",
        "js_ts_vue_web_frontend_security",
    ),
];

/// The base name the filename alone gives: folder and `.md` dropped, lowercased, `-` to `_`.
pub fn derived_base(relative_path: &str) -> String {
    let file = relative_path.rsplit('/').next().unwrap_or(relative_path);
    let stem = file.strip_suffix(".md").unwrap_or(file);
    stem.to_lowercase().replace('-', "_")
}

/// The tool ID for a set-relative path: `{ID_PREFIX}_{base}`, with any override applied.
///
/// The folder is dropped, so two folders holding the same filename would collide; the surface
/// builder treats that as a startup error rather than letting one file shadow the other.
pub fn tool_id_for(relative_path: &str) -> String {
    let base = NAME_OVERRIDES
        .iter()
        .find(|(path, _)| *path == relative_path)
        .map_or_else(
            || derived_base(relative_path),
            |(_, base)| (*base).to_owned(),
        );
    format!("{ID_PREFIX}_{base}")
}

/// Whether `name` is a valid MCP tool name: `^[a-z][a-z0-9_]{0,127}$`.
pub fn is_valid_tool_name(name: &str) -> bool {
    let mut chars = name.chars();
    let starts_well = chars.next().is_some_and(|c| c.is_ascii_lowercase());
    starts_well
        && name.len() <= 128
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_filename_derives_its_own_id() {
        assert_eq!(
            tool_id_for("references/python-flask-web-server-security.md"),
            "lxagents_skills_security_python_flask_web_server_security"
        );
        assert_eq!(tool_id_for("SKILL.md"), "lxagents_skills_security_skill");
    }

    #[test]
    fn an_override_replaces_only_the_base() {
        assert_eq!(
            tool_id_for("references/javascript-typescript-vue-web-frontend-security.md"),
            "lxagents_skills_security_js_ts_vue_web_frontend_security"
        );
        for (path, base) in NAME_OVERRIDES {
            assert_eq!(tool_id_for(path), format!("{ID_PREFIX}_{base}"));
            assert!(tool_id_for(path).len() <= MAX_PUBLISHED_ID_LENGTH);
        }
    }

    #[test]
    fn tool_name_shape() {
        assert!(is_valid_tool_name("a"));
        assert!(is_valid_tool_name("lxagents_skills_security_skill_2"));
        assert!(!is_valid_tool_name(""));
        assert!(!is_valid_tool_name("1abc"));
        assert!(!is_valid_tool_name("_abc"));
        assert!(!is_valid_tool_name("abc-def"));
        assert!(!is_valid_tool_name("abc.def"));
        assert!(!is_valid_tool_name(&"a".repeat(129)));
        assert!(is_valid_tool_name(&"a".repeat(128)));
    }
}
