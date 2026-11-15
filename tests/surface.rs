//! The tool surface, through the real binaries over stdio and through the CLI.

mod common;

use std::{collections::HashSet, process::Command};

use common::*;
use lxagents_security::instructions::{naming::MAX_PUBLISHED_ID_LENGTH, surface};

fn cli(args: &[&str]) -> (Option<i32>, String) {
    let output = Command::new(CLI).args(args).output().unwrap();
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

#[tokio::test]
async fn the_cli_list_and_the_mcp_tool_list_agree() {
    let (code, stdout) = cli(&["tools"]);
    assert_eq!(code, Some(0));
    let from_cli: Vec<(String, String)> = stdout
        .lines()
        .map(|line| {
            let (name, description) = line.split_once("  ").unwrap();
            (name.to_owned(), description.trim_start().to_owned())
        })
        .collect();

    let client = connect_stdio(&[]).await;
    let tools = client.list_all_tools().await.unwrap();
    let from_mcp: Vec<(String, String)> = tools
        .iter()
        .map(|tool| {
            (
                tool.name.to_string(),
                tool.description.clone().unwrap_or_default().to_string(),
            )
        })
        .collect();
    assert_eq!(from_cli, from_mcp);
    client.cancel().await.unwrap();
}

#[test]
fn the_surface_is_the_twelve_tools_the_set_derives() {
    let mut names: Vec<_> = surface()
        .unwrap()
        .iter()
        .map(|tool| tool.name.as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(names, ALL_TOOLS);
}

#[test]
fn the_tool_list_and_the_files_on_disk_are_a_bijection() {
    let on_disk: HashSet<String> = markdown_files().into_iter().map(|(path, _)| path).collect();
    let served: HashSet<String> = surface()
        .unwrap()
        .iter()
        .map(|tool| tool.path.to_owned())
        .collect();
    assert_eq!(on_disk, served);
    assert_eq!(
        served.len(),
        surface().unwrap().len(),
        "a file is served twice"
    );
}

#[test]
fn every_tool_id_is_the_prefix_plus_its_own_filename_or_a_shortened_override() {
    for tool in surface().unwrap() {
        let base = tool
            .name
            .strip_prefix(&format!("{ID_PREFIX}_"))
            .expect("prefixed");
        let derived = tool
            .path
            .rsplit('/')
            .next()
            .unwrap()
            .trim_end_matches(".md")
            .to_lowercase()
            .replace('-', "_");
        let shortened = derived
            .replacen("javascript_typescript", "js_ts", 1)
            .replacen("javascript_general", "js_general", 1);
        assert!(
            base == derived || base == shortened,
            "{} does not derive from {}",
            tool.name,
            tool.path
        );
        assert!(
            tool.name.len() <= MAX_PUBLISHED_ID_LENGTH,
            "{} is too long",
            tool.name
        );
    }
}

#[test]
fn every_tool_has_a_distinct_name_and_a_description_to_route_on() {
    let tools = surface().unwrap();
    let names: HashSet<_> = tools.iter().map(|tool| &tool.name).collect();
    assert_eq!(names.len(), tools.len());
    for tool in tools {
        assert!(
            tool.description.len() > 20,
            "{} has no real description",
            tool.name
        );
    }
}

#[tokio::test]
async fn no_tool_takes_an_argument_a_write_verb_or_a_credential() {
    let client = connect_stdio(&[]).await;
    for tool in client.list_all_tools().await.unwrap() {
        let schema = &tool.input_schema;
        assert_eq!(schema.get("type").and_then(|v| v.as_str()), Some("object"));
        let properties = schema
            .get("properties")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        assert!(properties.is_empty(), "{} takes an argument", tool.name);
        assert!(
            schema
                .get("required")
                .and_then(|v| v.as_array())
                .is_none_or(Vec::is_empty)
        );
        for name in [
            "action",
            "verb",
            "operation",
            "command",
            "body",
            "content",
            "path",
            "apiKey",
            "api_key",
            "token",
            "secret",
            "password",
        ] {
            assert!(
                !properties.contains_key(name),
                "{} accepts {name}",
                tool.name
            );
        }
    }
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn every_tool_returns_its_own_file_whole_and_with_frontmatter() {
    let client = connect_stdio(&[]).await;
    for tool in surface().unwrap() {
        let on_disk = std::fs::read_to_string(root().join("content").join(tool.path)).unwrap();
        let served = text_of(&call(&client, &tool.name).await);
        assert_eq!(
            served, on_disk,
            "{} is not byte-identical to {}",
            tool.name, tool.path
        );
        assert!(served.starts_with("---\n") || served.starts_with("---\r\n"));
    }
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn an_unknown_tool_is_refused() {
    let client = connect_stdio(&[]).await;
    let refused = client
        .call_tool(rmcp::model::CallToolRequestParams::new(format!(
            "{ID_PREFIX}_license"
        )))
        .await;
    assert!(refused.is_err(), "an unknown tool must not answer");
    client.cancel().await.unwrap();
}

#[test]
fn the_licence_is_not_served_as_an_instruction() {
    for tool in surface().unwrap() {
        assert_ne!(tool.path, "LICENSE.txt");
        assert!(!tool.name.contains("license"));
    }
}

#[tokio::test]
async fn skill_is_the_entry_point_and_routes_to_the_ten_reference_guides() {
    let client = connect_stdio(&[]).await;
    let text = text_of(&call(&client, &format!("{ID_PREFIX}_skill")).await);
    for needle in [
        "references directory",
        "`<language>-<framework>-<stack>-security.md`",
        "`<language>-general-<stack>-security.md`",
        "agnostic to the framework",
    ] {
        assert!(text.contains(needle), "skill no longer says {needle}");
    }
    let references: Vec<_> = markdown_files()
        .into_iter()
        .filter(|(path, _)| path.starts_with("references/"))
        .collect();
    assert_eq!(references.len(), 10);
    for (path, _) in references {
        let base = path.rsplit('/').next().unwrap();
        assert!(base.ends_with("-security.md"), "{path}");
        assert!(
            base.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.'),
            "{path}"
        );
        assert!(
            ["python", "javascript", "golang"].contains(&base.split('-').next().unwrap()),
            "{path}"
        );
    }
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn the_tool_surface_reads_no_credential_and_needs_no_key_to_answer() {
    let client = connect_stdio(&[("MCP_AUTH_TOKEN", "")]).await;
    let text = text_of(
        &call(
            &client,
            &format!("{ID_PREFIX}_golang_general_backend_security"),
        )
        .await,
    );
    assert!(text.len() > 1000);
    client.cancel().await.unwrap();
}

#[test]
fn the_cli_help_names_every_environment_variable_the_server_reads() {
    let (code, help) = cli(&["--help"]);
    assert_eq!(code, Some(0));

    let mut sources = String::new();
    fn collect(dir: &std::path::Path, into: &mut String) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(&path, into);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                into.push_str(&std::fs::read_to_string(path).unwrap());
            }
        }
    }
    collect(&root().join("src"), &mut sources);

    // Every quoted SCREAMING_CASE name passed to an environment read: `get("X")`,
    // `env::var("X")`, and the CLI's own `env.insert("X", …)` overrides.
    let mut read = HashSet::new();
    for marker in ["get(\"", "env::var(\"", "insert(\""] {
        for piece in sources.split(marker).skip(1) {
            let name: String = piece
                .chars()
                .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
                .collect();
            if name.len() > 2 && piece[name.len()..].starts_with('"') {
                read.insert(name);
            }
        }
    }
    assert!(
        read.contains("MCP_AUTH_TOKEN"),
        "the scan must find MCP_AUTH_TOKEN: {read:?}"
    );
    assert!(
        read.contains("MCP_CLUSTER_WORKERS"),
        "the scan must find MCP_CLUSTER_WORKERS: {read:?}"
    );
    for name in read {
        assert!(help.contains(&name), "--help does not mention {name}");
    }
}

#[test]
fn the_cli_answers_version_help_and_unknown_commands_as_before() {
    let (code, version) = cli(&["-v"]);
    assert_eq!(code, Some(0));
    assert_eq!(version.trim(), env!("CARGO_PKG_VERSION"));
    assert_eq!(cli(&["--version"]).1, version);
    let (code, help) = cli(&[]);
    assert_eq!(code, Some(2), "no command prints help and exits 2");
    assert!(help.contains("Commands"));
    let output = Command::new(CLI).arg("frobnicate").output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Unknown command \"frobnicate\""));
}

#[test]
fn automation_lists_every_other_file_in_the_set_each_once_with_a_condition() {
    let hub = std::fs::read_to_string(root().join("content/automation.md")).unwrap();
    let listed: Vec<String> = hub
        .lines()
        .filter_map(|line| line.strip_prefix("- `"))
        .filter_map(|rest| rest.split_once("` - "))
        .filter(|(name, condition)| {
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                && condition.starts_with(|c: char| !c.is_whitespace())
        })
        .map(|(name, _)| name.to_owned())
        .collect();

    let mut expected: Vec<String> = surface()
        .unwrap()
        .iter()
        .filter(|tool| tool.path != "automation.md")
        .map(|tool| tool.name.clone())
        .collect();
    let mut sorted = listed.clone();
    sorted.sort();
    expected.sort();
    assert_eq!(sorted, expected, "automation.md and content/ disagree");
    assert_eq!(
        listed.iter().collect::<HashSet<_>>().len(),
        listed.len(),
        "a tool is listed twice"
    );
}
