//! The Streamable HTTP transport, over a real socket, against the real binary.
//!
//! The surface tests drive stdio. What they cannot reach is the part that only exists when
//! there is a socket: the route table, the framing, the host it binds, the shape of its
//! errors, the token, and what a signal does to the process.

mod common;

use std::{collections::HashSet, time::Duration};

use common::*;
use lxagents_security::http::{
    app::{self, AppOptions},
    auth::MIN_TOKEN_LENGTH,
    body::BODY_LIMIT_BYTES,
};
use tokio_util::sync::CancellationToken;

fn bearer(token: &str) -> (String, String) {
    ("Authorization".to_owned(), format!("Bearer {token}"))
}

fn get(server: &Server, path: &str, token: Option<&str>) -> Raw {
    let header = token.map(|token| format!("Bearer {token}"));
    let headers: Vec<(&str, &str)> = header
        .iter()
        .map(|value| ("Authorization", value.as_str()))
        .collect();
    raw(server.port, "GET", path, &headers, "")
}

fn with_host(server: &Server, host: &str, path: &str) -> Raw {
    raw(server.port, "GET", path, &[("Host", host)], "")
}

// ----------------------------------------------------------------------------------------
// Routes and the MCP endpoint.
// ----------------------------------------------------------------------------------------

#[test]
fn the_health_check_answers_without_a_session() {
    let server = Server::start(&[]);
    let response = get(&server, "/healthz", None);
    assert_eq!(response.status, 200);
    let body = response.json();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["server"], "lxagents-security");
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
}

#[test]
fn the_startup_line_names_the_interface_it_bound() {
    let server = Server::start(&[]);
    assert!(
        server.output().contains(&format!(
            "serving over http on :{}/mcp (127.0.0.1)",
            server.port
        )),
        "{}",
        server.output()
    );
}

#[tokio::test]
async fn a_server_process_writes_nothing_to_stdout() {
    let server = Server::start(&[]);
    let client = connect_http(&server.url, Some(TEST_TOKEN)).await;
    client.list_all_tools().await.unwrap();
    text_of(&call(&client, &format!("{ID_PREFIX}_skill")).await);
    client.cancel().await.unwrap();
    assert_eq!(server.stdout(), "", "stdout must stay empty");
    assert!(
        !server.stderr().is_empty(),
        "the startup line went to stderr"
    );
}

#[tokio::test]
async fn the_http_transport_serves_the_same_tools_as_stdio() {
    let server = Server::start(&[]);
    let http = connect_http(&server.url, Some(TEST_TOKEN)).await;
    let stdio = connect_stdio(&[]).await;
    let via_http = http.list_all_tools().await.unwrap();
    let via_stdio = stdio.list_all_tools().await.unwrap();
    assert_eq!(via_http.len(), 12, "one tool per file in the set");
    assert_eq!(via_http, via_stdio);
    for tool in &via_http {
        assert_eq!(
            text_of(&call(&http, &tool.name).await),
            text_of(&call(&stdio, &tool.name).await)
        );
    }
    http.cancel().await.unwrap();
    stdio.cancel().await.unwrap();
}

#[tokio::test]
async fn no_tool_advertises_an_argument_over_http_and_nothing_outside_the_set_is_registered() {
    let server = Server::start(&[]);
    let client = connect_http(&server.url, Some(TEST_TOKEN)).await;
    let tools = client.list_all_tools().await.unwrap();
    for tool in &tools {
        let properties = tool
            .input_schema
            .get("properties")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        assert!(
            properties.is_empty(),
            "{} takes an argument over HTTP",
            tool.name
        );
    }
    let mut names: Vec<_> = tools.iter().map(|tool| tool.name.to_string()).collect();
    names.sort();
    assert_eq!(names, ALL_TOOLS);
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn a_tool_call_over_http_returns_the_file_byte_identically() {
    let server = Server::start(&[]);
    let client = connect_http(&server.url, Some(TEST_TOKEN)).await;
    let text = text_of(
        &call(
            &client,
            &format!("{ID_PREFIX}_python_flask_web_server_security"),
        )
        .await,
    );
    let on_disk = std::fs::read_to_string(
        root().join("content/references/python-flask-web-server-security.md"),
    )
    .unwrap();
    assert_eq!(text, on_disk);
    client.cancel().await.unwrap();
}

#[test]
fn a_bare_tools_list_is_answered_without_a_handshake() {
    // Stateless: every request carries its own envelope, so a lone tools/list is served.
    let server = Server::start(&[]);
    let response = post_mcp(server.port, &rpc_headers(Some(TEST_TOKEN)), LIST_TOOLS);
    assert_eq!(response.status, 200, "{}", response.body);
    assert!(response.body.contains("python_flask_web_server_security"));
}

#[test]
fn an_unknown_route_says_what_this_server_does_not_serve() {
    let server = Server::start(&[]);
    let response = get(&server, "/nope", Some(TEST_TOKEN));
    assert_eq!(response.status, 404);
    let body = response.json();
    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["error"]["code"], -32601);
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Not found: /nope")
    );
}

#[test]
fn get_and_delete_on_mcp_are_refused_rather_than_served() {
    let server = Server::start(&[]);
    for method in ["GET", "DELETE"] {
        let response = raw(
            server.port,
            method,
            "/mcp",
            &[("Authorization", &format!("Bearer {TEST_TOKEN}"))],
            "",
        );
        assert_eq!(response.status, 405, "{method}");
        assert!(
            response.json()["error"]["message"]
                .as_str()
                .unwrap()
                .contains("stateless mode")
        );
    }
}

#[tokio::test]
async fn concurrent_requests_do_not_share_state() {
    let server = Server::start(&[("MCP_CLUSTER_WORKERS", "4")]);
    let tools = [
        "lxagents_skills_security_python_flask_web_server_security",
        "lxagents_skills_security_javascript_express_web_server_security",
        "lxagents_skills_security_golang_general_backend_security",
    ];
    let mut clients = Vec::new();
    for _ in tools {
        clients.push(connect_http(&server.url, Some(TEST_TOKEN)).await);
    }
    let (first, second, third) = tokio::join!(
        call(&clients[0], tools[0]),
        call(&clients[1], tools[1]),
        call(&clients[2], tools[2])
    );
    let results = [first, second, third];
    for (result, tool) in results.iter().zip(tools) {
        let text = text_of(result);
        let own_name = tool
            .strip_prefix(&format!("{ID_PREFIX}_"))
            .unwrap()
            .replace('_', "-");
        assert!(
            text.contains(&format!("name: {own_name}")),
            "{tool} was answered with another request's file"
        );
    }
    for client in clients {
        client.cancel().await.unwrap();
    }
}

// ----------------------------------------------------------------------------------------
// Shutdown.
// ----------------------------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn a_shutdown_drains_and_stops_accepting() {
    for workers in ["1", "3"] {
        let mut server = Server::start(&[("MCP_CLUSTER_WORKERS", workers)]);
        server.signal("INT");
        let status = server
            .wait_exit(Duration::from_secs(10))
            .expect("the process exited");
        assert_eq!(status.code(), Some(0), "the handler did not run: {status}");
        assert!(
            server.output().contains("SIGINT, draining"),
            "{}",
            server.output()
        );
        assert_eq!(
            server.output().matches("draining").count(),
            1,
            "one process, one drain line"
        );
        assert!(
            std::net::TcpStream::connect(("127.0.0.1", server.port)).is_err(),
            "the port is still served"
        );
    }
}

#[cfg(unix)]
#[test]
fn sigterm_drains_too() {
    let mut server = Server::start(&[]);
    server.signal("TERM");
    let status = server
        .wait_exit(Duration::from_secs(10))
        .expect("the process exited");
    assert_eq!(status.code(), Some(0));
    assert!(server.output().contains("SIGTERM, draining"));
}

#[cfg(unix)]
#[test]
fn a_second_signal_stops_the_wait_rather_than_queueing_behind_the_first() {
    let mut server = Server::start(&[]);
    server.signal("INT");
    server.signal("INT");
    let status = server
        .wait_exit(Duration::from_secs(10))
        .expect("the process exited");
    assert_eq!(status.code(), Some(0));
    assert_eq!(server.output().matches("SIGINT, draining").count(), 1);
}

// ----------------------------------------------------------------------------------------
// The Host allow-list.
// ----------------------------------------------------------------------------------------

#[test]
fn with_no_allow_list_set_nothing_is_refused_and_the_startup_line_says_so() {
    let server = Server::start(&[]);
    for host in ["example.test", "evil.test", "127.0.0.1"] {
        assert_eq!(with_host(&server, host, "/healthz").status, 200, "{host}");
    }
    assert!(
        server
            .output()
            .contains("MCP_ALLOWED_HOSTS is unset, so no Host header allow-list is applied")
    );
}

#[test]
fn the_allow_list_is_applied_when_mcp_allowed_hosts_is_set() {
    let server = Server::start(&[(
        "MCP_ALLOWED_HOSTS",
        "security.example.test, other.example.test",
    )]);
    assert_eq!(
        with_host(&server, "security.example.test", "/healthz").status,
        200
    );
    assert_eq!(
        with_host(&server, "other.example.test", "/healthz").status,
        200
    );
    let refused = with_host(&server, "evil.test", "/healthz");
    assert_eq!(refused.status, 403);
    assert!(
        refused.json()["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Invalid Host: evil.test")
    );
    let with_port = format!("security.example.test:{}", server.port);
    assert_eq!(
        with_host(&server, &with_port, "/healthz").status,
        200,
        "the port is not part of the match"
    );
    assert!(
        !server.output().contains("MCP_ALLOWED_HOSTS is unset"),
        "setting it silences the warning"
    );
    assert!(
        !server
            .output()
            .contains("no Host header allow-list is applied")
    );
}

#[test]
fn the_allow_list_guards_the_health_check_and_runs_before_the_token_check() {
    let server = Server::start(&[("MCP_ALLOWED_HOSTS", "security.example.test")]);
    assert_eq!(with_host(&server, "evil.test", "/healthz").status, 403);
    assert_eq!(
        with_host(&server, "security.example.test", "/healthz").status,
        200
    );
    assert_eq!(
        with_host(&server, "evil.test", "/mcp").status,
        403,
        "a Host problem, not a 401"
    );
}

#[test]
fn an_allow_list_of_nothing_but_separators_still_counts_as_unset() {
    let server = Server::start(&[("MCP_ALLOWED_HOSTS", " , , ")]);
    assert_eq!(with_host(&server, "anything.test", "/healthz").status, 200);
    assert!(server.output().contains("MCP_ALLOWED_HOSTS is unset"));
}

// ----------------------------------------------------------------------------------------
// The body.
// ----------------------------------------------------------------------------------------

#[test]
fn the_body_limit_is_4_mb() {
    assert_eq!(BODY_LIMIT_BYTES, 4 * 1024 * 1024);
}

#[test]
fn a_body_over_the_limit_is_refused_in_the_json_rpc_envelope() {
    let server = Server::start(&[]);
    let oversized = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": format!("{ID_PREFIX}_skill"), "arguments": {}, "padding": "x".repeat(BODY_LIMIT_BYTES) },
    })
    .to_string();
    assert!(oversized.len() > BODY_LIMIT_BYTES);
    let response = post_mcp(server.port, &rpc_headers(Some(TEST_TOKEN)), &oversized);
    assert_eq!(response.status, 400);
    assert_eq!(response.json()["jsonrpc"], "2.0");
    assert_eq!(response.json()["error"]["code"], -32700);
}

#[test]
fn malformed_json_gets_exactly_the_same_answer_as_a_body_that_is_too_large() {
    let server = Server::start(&[]);
    let response = post_mcp(
        server.port,
        &rpc_headers(Some(TEST_TOKEN)),
        "{ this is not json",
    );
    assert_eq!(response.status, 400);
    assert_eq!(response.json()["error"]["code"], -32700);
    assert_eq!(
        response.json()["error"]["message"],
        "Parse error: request body is not valid JSON"
    );
}

#[test]
fn no_response_advertises_the_framework() {
    let server = Server::start(&[]);
    let responses = [
        get(&server, "/healthz", None),
        get(&server, "/nope", Some(TEST_TOKEN)),
        get(&server, "/mcp", Some(TEST_TOKEN)),
        get(&server, "/mcp", None),
    ];
    for response in responses {
        assert_eq!(response.header("x-powered-by"), None);
        assert_eq!(response.header("server"), None);
    }
}

// ----------------------------------------------------------------------------------------
// Worker threads.
// ----------------------------------------------------------------------------------------

#[tokio::test]
async fn mcp_cluster_workers_1_runs_a_single_thread_and_still_serves() {
    let server = Server::start(&[("MCP_CLUSTER_WORKERS", "1")]);
    assert!(
        server
            .output()
            .contains("MCP_CLUSTER_WORKERS is 1, so a single thread serves every request.")
    );
    assert_eq!(server.output().matches("serving over http").count(), 1);
    let client = connect_http(&server.url, Some(TEST_TOKEN)).await;
    assert_eq!(client.list_all_tools().await.unwrap().len(), 12);
    client.cancel().await.unwrap();
}

#[test]
fn mcp_cluster_workers_n_serves_from_n_threads_in_one_process() {
    let server = Server::start(&[("MCP_CLUSTER_WORKERS", "2")]);
    assert!(
        server
            .output()
            .contains("serving HTTP from 2 worker threads"),
        "{}",
        server.output()
    );
    assert_eq!(
        server.output().matches("serving over http").count(),
        1,
        "one listener"
    );
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let port = server.port;
            std::thread::spawn(move || raw(port, "GET", "/healthz", &[], ""))
        })
        .collect();
    for thread in threads {
        let response = thread.join().unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.json()["server"], "lxagents-security");
    }
}

#[tokio::test]
async fn stdio_runs_no_http_runtime() {
    let client = connect_stdio(&[]).await;
    assert_eq!(client.list_all_tools().await.unwrap().len(), 12);
    client.cancel().await.unwrap();
    let output = std::process::Command::new(SERVER)
        .env("MCP_TRANSPORT", "stdio")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("worker threads"), "{stderr}");
    assert!(!stderr.contains("serving over http"), "{stderr}");
    assert!(
        output.stdout.is_empty(),
        "stdio wrote something that is not JSON-RPC"
    );
}

// ----------------------------------------------------------------------------------------
// Authentication.
// ----------------------------------------------------------------------------------------

#[test]
fn a_request_with_no_token_is_refused_with_a_401_the_client_can_read() {
    let server = Server::start(&[]);
    let response = post_mcp(server.port, &rpc_headers(None), LIST_TOOLS);
    assert_eq!(response.status, 401);
    assert_eq!(
        response.header("www-authenticate"),
        Some("Bearer realm=\"mcp\"")
    );
    let body = response.json();
    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["id"], serde_json::Value::Null);
    assert_eq!(body["error"]["code"], -32001);
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("send the token as Authorization: Bearer")
    );
}

#[test]
fn a_wrong_token_is_refused_whatever_its_length_and_never_with_a_500() {
    let server = Server::start(&[]);
    let one_off = format!(
        "{}{}",
        &TEST_TOKEN[..TEST_TOKEN.len() - 1],
        if TEST_TOKEN.ends_with('f') { "e" } else { "f" }
    );
    let wrong = [
        one_off,
        TEST_TOKEN[..TEST_TOKEN.len() - 1].to_owned(),
        format!("{TEST_TOKEN}0"),
        TEST_TOKEN.to_uppercase(),
        "x".to_owned(),
        "x".repeat(10_000),
    ];
    for token in wrong {
        let mut headers = rpc_headers(None);
        headers.push(bearer(&token));
        let response = post_mcp(server.port, &headers, LIST_TOOLS);
        assert_eq!(
            response.status,
            401,
            "a {}-character token must be refused",
            token.len()
        );
        assert_eq!(
            response.header("www-authenticate"),
            Some("Bearer realm=\"mcp\", error=\"invalid_token\"")
        );
        assert!(
            response.json()["error"]["message"]
                .as_str()
                .unwrap()
                .contains("the token is not valid")
        );
    }
}

#[test]
fn only_a_bearer_credential_counts_and_the_scheme_is_case_insensitive() {
    let server = Server::start(&[]);
    let refused = [
        format!("Basic {TEST_TOKEN}"),
        "Bearer".to_owned(),
        "Bearer ".to_owned(),
        format!("Bearer {TEST_TOKEN} {TEST_TOKEN}"),
        TEST_TOKEN.to_owned(),
    ];
    for authorization in refused {
        let mut headers = rpc_headers(None);
        headers.push(("Authorization".into(), authorization.clone()));
        assert_eq!(
            post_mcp(server.port, &headers, LIST_TOOLS).status,
            401,
            "{authorization:.12}"
        );
    }
    let mut headers = rpc_headers(None);
    headers.push(("Authorization".into(), format!("bearer {TEST_TOKEN}")));
    assert_eq!(
        post_mcp(server.port, &headers, LIST_TOOLS).status,
        200,
        "RFC 7235: the scheme is case-insensitive"
    );
}

#[test]
fn an_unauthenticated_caller_learns_nothing_about_which_routes_exist() {
    let server = Server::start(&[]);
    for (method, path) in [
        ("GET", "/mcp"),
        ("DELETE", "/mcp"),
        ("POST", "/nope"),
        ("GET", "/sse"),
        ("POST", "/message"),
    ] {
        let response = raw(server.port, method, path, &[], "");
        assert_eq!(
            response.status, 401,
            "{method} {path} must be refused, not routed"
        );
        assert_eq!(
            response.header("allow"),
            None,
            "{method} {path} leaked its methods"
        );
    }
}

#[test]
fn the_token_is_checked_before_the_body_is_read() {
    let server = Server::start(&[]);
    let response = raw(
        server.port,
        "POST",
        "/mcp",
        &[("Content-Type", "application/json")],
        "{ this is not json",
    );
    assert_eq!(response.status, 401);
}

#[test]
fn only_get_healthz_is_open_and_the_exemption_is_exact() {
    let server = Server::start(&[]);
    assert_eq!(
        get(&server, "/healthz", None).status,
        200,
        "the probe needs no token"
    );
    assert_eq!(
        get(&server, "/healthz", Some("wrong")).status,
        200,
        "a wrong token on it is not an error"
    );
    assert_eq!(raw(server.port, "POST", "/healthz", &[], "").status, 401);
    assert_eq!(raw(server.port, "HEAD", "/healthz", &[], "").status, 401);
    assert_eq!(get(&server, "/healthz/x", None).status, 401);
    assert_eq!(get(&server, "/healthzz", None).status, 401);
}

#[test]
fn the_startup_line_says_a_token_is_required_and_no_log_line_ever_carries_one() {
    let server = Server::start(&[]);
    assert!(
        server
            .output()
            .contains("bearer token required on every route except GET /healthz")
    );
    assert!(!server.output().contains("no bearer token is required"));
    let attempt = "wrong-token-that-must-not-be-logged-0123456789";
    let mut headers = rpc_headers(None);
    headers.push(bearer(attempt));
    post_mcp(server.port, &headers, LIST_TOOLS);
    post_mcp(server.port, &rpc_headers(Some(TEST_TOKEN)), LIST_TOOLS);
    std::thread::sleep(Duration::from_millis(100));
    assert!(
        !server.output().contains(TEST_TOKEN),
        "the real token must not be logged"
    );
    assert!(
        !server.output().contains(attempt),
        "a rejected token must not be logged"
    );
}

#[tokio::test]
async fn a_token_with_a_trailing_newline_as_an_env_file_leaves_it_still_matches() {
    let padded = format!("  {TEST_TOKEN}\r\n");
    let server = Server::start(&[("MCP_AUTH_TOKEN", &padded)]);
    let client = connect_http(&server.url, Some(TEST_TOKEN)).await;
    assert_eq!(client.list_all_tools().await.unwrap().len(), 12);
    client.cancel().await.unwrap();
}

#[test]
fn a_token_of_exactly_the_minimum_length_is_accepted() {
    let token = "k".repeat(MIN_TOKEN_LENGTH);
    let server = Server::start(&[("MCP_AUTH_TOKEN", &token)]);
    assert_eq!(
        post_mcp(server.port, &rpc_headers(Some(&token)), LIST_TOOLS).status,
        200
    );
}

#[test]
fn http_refuses_to_start_without_a_usable_token_and_says_so_once() {
    let one_short = "k".repeat(MIN_TOKEN_LENGTH - 1);
    let cases = [
        ("unset", "", "MCP_AUTH_TOKEN is not set".to_owned()),
        (
            "only whitespace",
            "  \n ",
            "MCP_AUTH_TOKEN is not set".to_owned(),
        ),
        (
            "too short",
            "short-secret",
            "MCP_AUTH_TOKEN is 12 characters, and it must be at least 32".to_owned(),
        ),
        (
            "one character short",
            one_short.as_str(),
            format!("is {} characters", MIN_TOKEN_LENGTH - 1),
        ),
    ];
    for (name, token, message) in cases {
        for workers in ["1", "3"] {
            // Every interface, so this is a server. It exits before binding.
            let (code, output) = run_to_exit(&[
                ("MCP_AUTH_TOKEN", token),
                ("HOST", "0.0.0.0"),
                ("MCP_CLUSTER_WORKERS", workers),
            ]);
            assert_eq!(code, Some(1), "{name}: {output}");
            assert!(output.contains(&message), "{name}: {output}");
            assert!(
                output.contains("openssl rand -hex 32"),
                "{name}: the message must say how to fix it"
            );
            assert!(
                !output.contains("serving over http"),
                "{name}: it must not have bound the port"
            );
            assert!(
                !output.contains("worker threads"),
                "{name}: no runtime was built"
            );
            assert_eq!(
                output.matches("will not start the HTTP transport").count(),
                1,
                "{name}"
            );
            if !token.trim().is_empty() {
                assert!(
                    !output.contains(token),
                    "{name}: the value must not be echoed"
                );
            }
        }
    }
}

#[test]
fn the_app_cannot_be_built_without_a_usable_token() {
    let surface = lxagents_security::instructions::surface().unwrap();
    let build = |token: &str, local: bool| {
        let options = AppOptions {
            token: token.into(),
            allowed_hosts: vec![],
            local,
        };
        app::build(surface, &options, CancellationToken::new())
    };
    assert!(
        build("", false)
            .unwrap_err()
            .contains("Refusing to build the HTTP app: MCP_AUTH_TOKEN is not set")
    );
    assert!(
        build("short", false)
            .unwrap_err()
            .contains("must be at least 32")
    );
    assert!(build("", true).is_ok(), "local needs no token");
    assert!(
        build("short", true)
            .unwrap_err()
            .contains("must be at least 32")
    );
    assert!(build(&"z".repeat(40), false).is_ok());
}

#[tokio::test]
async fn stdio_never_reads_the_token_so_even_an_unusable_one_does_not_stop_it() {
    let client = connect_stdio(&[("MCP_AUTH_TOKEN", "x")]).await;
    assert_eq!(client.list_all_tools().await.unwrap().len(), 12);
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn http_bound_to_loopback_starts_and_serves_without_a_token() {
    for workers in ["1", "2"] {
        let server = Server::start(&[("MCP_AUTH_TOKEN", ""), ("MCP_CLUSTER_WORKERS", workers)]);
        let output = server.output();
        assert!(
            output.contains(
                "bound to loopback, so no bearer token is required; Host is limited to localhost"
            ),
            "{output}"
        );
        assert!(!output.contains("bearer token required on every route"));
        assert!(!output.contains("MCP_ALLOWED_HOSTS is unset"));
        let client = connect_http(&server.url, None).await;
        assert_eq!(client.list_all_tools().await.unwrap().len(), 12);
        assert!(!text_of(&call(&client, &format!("{ID_PREFIX}_automation")).await).is_empty());
        client.cancel().await.unwrap();
    }
}

#[test]
fn a_local_server_without_a_token_only_answers_to_a_localhost_host_header() {
    let server = Server::start(&[("MCP_AUTH_TOKEN", "")]);
    for host in ["localhost", "127.0.0.1"] {
        assert_eq!(
            with_host(&server, &format!("{host}:{}", server.port), "/healthz").status,
            200,
            "{host}"
        );
    }
    for host in ["attacker.example", "attacker.example:80"] {
        assert_eq!(with_host(&server, host, "/healthz").status, 403, "{host}");
    }
}

#[test]
fn a_local_server_refuses_a_token_that_is_set_but_too_short() {
    let (code, output) = run_to_exit(&[("MCP_AUTH_TOKEN", "short-secret")]);
    assert_eq!(code, Some(1), "{output}");
    assert!(output.contains("MCP_AUTH_TOKEN is 12 characters"));
}

#[test]
fn every_tool_name_seen_over_http_is_distinct() {
    let server = Server::start(&[]);
    let response = post_mcp(server.port, &rpc_headers(Some(TEST_TOKEN)), LIST_TOOLS);
    let tools = response.json()["result"]["tools"]
        .as_array()
        .unwrap()
        .clone();
    let names: HashSet<_> = tools
        .iter()
        .map(|tool| tool["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(names.len(), 12);
}
