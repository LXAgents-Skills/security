//! What the integration tests share: starting the real binaries, and talking to them.
//!
//! The tests drive the compiled `lxagents-security-server` and `lxagents-security` as child
//! processes, because that is what a client spawns; an in-process server could not be given a
//! second port, a signal, or a different environment.
#![allow(dead_code)]

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use rmcp::{
    RoleClient, ServiceExt,
    model::{CallToolRequestParams, CallToolResult, ContentBlock},
    service::RunningService,
    transport::{
        StreamableHttpClientTransport, TokioChildProcess,
        streamable_http_client::StreamableHttpClientTransportConfig,
    },
};

/// The server binary, as Cargo built it for this test run.
pub const SERVER: &str = env!("CARGO_BIN_EXE_lxagents-security-server");
/// The CLI binary.
pub const CLI: &str = env!("CARGO_BIN_EXE_lxagents-security");
/// The tool prefix.
pub const ID_PREFIX: &str = "lxagents_skills_security";

/// The token every server started here runs with unless a test says otherwise. Long enough to
/// pass the length check, and obviously not a real secret.
pub const TEST_TOKEN: &str = "test-token-0123456789abcdef0123456789abcdef";

/// How long to wait for a startup line: a deadline, not a sleep.
pub const READY_TIMEOUT: Duration = Duration::from_secs(30);

/// The twelve tool IDs the set derives, sorted.
pub const ALL_TOOLS: [&str; 12] = [
    "lxagents_skills_security_automation",
    "lxagents_skills_security_golang_general_backend_security",
    "lxagents_skills_security_javascript_express_web_server_security",
    "lxagents_skills_security_javascript_jquery_web_frontend_security",
    "lxagents_skills_security_js_general_web_frontend_security",
    "lxagents_skills_security_js_ts_nextjs_web_server_security",
    "lxagents_skills_security_js_ts_react_web_frontend_security",
    "lxagents_skills_security_js_ts_vue_web_frontend_security",
    "lxagents_skills_security_python_django_web_server_security",
    "lxagents_skills_security_python_fastapi_web_server_security",
    "lxagents_skills_security_python_flask_web_server_security",
    "lxagents_skills_security_skill",
];

/// The repository root.
pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every `.md` under `content/`, as `(set-relative path, absolute path)`.
pub fn markdown_files() -> Vec<(String, PathBuf)> {
    fn walk(dir: &Path, base: &Path, found: &mut Vec<(String, PathBuf)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, base, found);
            } else if path.extension().is_some_and(|ext| ext == "md") {
                let relative = path
                    .strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                found.push((relative, path));
            }
        }
    }
    let content = root().join("content");
    let mut found = Vec::new();
    walk(&content, &content, &mut found);
    found.sort();
    found
}

/// A port nothing is listening on, bound and released.
pub fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// A running server process and what it has written.
pub struct Server {
    pub child: Child,
    pub port: u16,
    pub url: String,
    stdout: Arc<Mutex<String>>,
    stderr: Arc<Mutex<String>>,
}

fn capture(mut stream: impl Read + Send + 'static, into: Arc<Mutex<String>>) {
    thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        while let Ok(n) = stream.read(&mut buffer) {
            if n == 0 {
                break;
            }
            into.lock()
                .unwrap()
                .push_str(&String::from_utf8_lossy(&buffer[..n]));
        }
    });
}

/// The environment every HTTP server here starts with: HTTP, loopback, a free port, the token.
fn http_command(port: u16, env: &[(&str, &str)]) -> Command {
    let mut command = Command::new(SERVER);
    command
        .env("MCP_TRANSPORT", "http")
        .env("PORT", port.to_string())
        // Loopback on purpose: the suite never opens a port on every interface.
        .env("HOST", "127.0.0.1")
        .env("MCP_AUTH_TOKEN", TEST_TOKEN)
        .env_remove("MCP_ALLOWED_HOSTS")
        .env_remove("MCP_CLUSTER_WORKERS")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in env {
        command.env(key, value);
    }
    command
}

impl Server {
    /// Start the server and wait for its startup line, or fail with its output.
    pub fn start(env: &[(&str, &str)]) -> Self {
        let port = free_port();
        let mut child = http_command(port, env).spawn().expect("spawn the server");
        let stdout = Arc::new(Mutex::new(String::new()));
        let stderr = Arc::new(Mutex::new(String::new()));
        capture(child.stdout.take().unwrap(), stdout.clone());
        capture(child.stderr.take().unwrap(), stderr.clone());
        let mut server = Self {
            child,
            port,
            url: format!("http://127.0.0.1:{port}"),
            stdout,
            stderr,
        };

        let deadline = Instant::now() + READY_TIMEOUT;
        while !server.output().contains("serving over http") {
            if let Some(status) = server.child.try_wait().unwrap() {
                panic!(
                    "the server never came up: it exited {status}\n--- output ---\n{}",
                    server.output()
                );
            }
            assert!(
                Instant::now() < deadline,
                "no startup line\n--- output ---\n{}",
                server.output()
            );
            thread::sleep(Duration::from_millis(20));
        }
        server
    }

    pub fn stdout(&self) -> String {
        self.stdout.lock().unwrap().clone()
    }

    pub fn stderr(&self) -> String {
        self.stderr.lock().unwrap().clone()
    }

    /// Everything the process has written, on either stream.
    pub fn output(&self) -> String {
        format!("{}{}", self.stdout(), self.stderr())
    }

    /// Send `signal` to the process (Unix).
    pub fn signal(&self, signal: &str) {
        let status = Command::new("kill")
            .arg(format!("-{signal}"))
            .arg(self.child.id().to_string())
            .status()
            .unwrap();
        assert!(status.success());
    }

    /// Wait up to `within` for the process to exit.
    pub fn wait_exit(&mut self, within: Duration) -> Option<ExitStatus> {
        let deadline = Instant::now() + within;
        while Instant::now() < deadline {
            if let Some(status) = self.child.try_wait().unwrap() {
                // Give the capture threads a moment to drain the pipes.
                thread::sleep(Duration::from_millis(50));
                return Some(status);
            }
            thread::sleep(Duration::from_millis(20));
        }
        None
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Start a server that is expected to stop by itself; return its exit code and output.
pub fn run_to_exit(env: &[(&str, &str)]) -> (Option<i32>, String) {
    let output = http_command(free_port(), env)
        .output()
        .expect("run the server");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.code(), text)
}

/// A raw HTTP/1.1 response.
#[derive(Debug)]
pub struct Raw {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Raw {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    pub fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body).unwrap_or_else(|_| panic!("not JSON: {}", self.body))
    }
}

/// One HTTP/1.1 request over a fresh socket, with exactly the headers given. A hand-written
/// client, because the tests need to send a `Host` header of their own choosing and many HTTP
/// clients silently replace it.
pub fn raw(port: u16, method: &str, path: &str, headers: &[(&str, &str)], body: &str) -> Raw {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    let mut request = format!("{method} {path} HTTP/1.1\r\n");
    if !headers
        .iter()
        .any(|(key, _)| key.eq_ignore_ascii_case("host"))
    {
        request.push_str(&format!("Host: 127.0.0.1:{port}\r\n"));
    }
    for (key, value) in headers {
        request.push_str(&format!("{key}: {value}\r\n"));
    }
    request.push_str(&format!(
        "Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    ));
    stream.write_all(request.as_bytes()).unwrap();
    stream.write_all(body.as_bytes()).unwrap();

    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    reader.read_line(&mut status_line).unwrap();
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    let mut headers = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.push((key.trim().to_owned(), value.trim().to_owned()));
        }
    }
    let mut body = Vec::new();
    let _ = reader.read_to_end(&mut body);
    Raw {
        status,
        headers,
        body: String::from_utf8_lossy(&body).into_owned(),
    }
}

/// The headers a JSON-RPC POST to `/mcp` carries, with the token.
pub fn rpc_headers(token: Option<&str>) -> Vec<(String, String)> {
    let mut headers = vec![
        ("Content-Type".to_owned(), "application/json".to_owned()),
        (
            "Accept".to_owned(),
            "application/json, text/event-stream".to_owned(),
        ),
    ];
    if let Some(token) = token {
        headers.push(("Authorization".to_owned(), format!("Bearer {token}")));
    }
    headers
}

/// `tools/list` as a bare JSON-RPC request.
pub const LIST_TOOLS: &str = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;

/// POST `body` to `/mcp` with `headers`.
pub fn post_mcp(port: u16, headers: &[(String, String)], body: &str) -> Raw {
    let borrowed: Vec<(&str, &str)> = headers
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    raw(port, "POST", "/mcp", &borrowed, body)
}

/// An MCP client over Streamable HTTP, carrying `token` if given.
pub async fn connect_http(url: &str, token: Option<&str>) -> RunningService<RoleClient, ()> {
    let mut config = StreamableHttpClientTransportConfig::with_uri(format!("{url}/mcp"));
    if let Some(token) = token {
        config = config.auth_header(token);
    }
    ().serve(StreamableHttpClientTransport::from_config(config))
        .await
        .expect("connect over HTTP")
}

/// An MCP client over stdio, spawning the server with `env`.
pub async fn connect_stdio(env: &[(&str, &str)]) -> RunningService<RoleClient, ()> {
    let mut command = tokio::process::Command::new(SERVER);
    command.env("MCP_TRANSPORT", "stdio").stderr(Stdio::null());
    for (key, value) in env {
        command.env(key, value);
    }
    ().serve(TokioChildProcess::new(command).expect("spawn"))
        .await
        .expect("connect over stdio")
}

/// Call `name` with no arguments.
pub async fn call(client: &RunningService<RoleClient, ()>, name: &str) -> CallToolResult {
    client
        .call_tool(CallToolRequestParams::new(name.to_owned()))
        .await
        .expect("call the tool")
}

/// The text of a single-content, successful tool result.
pub fn text_of(result: &CallToolResult) -> String {
    assert_ne!(result.is_error, Some(true), "expected a successful result");
    assert_eq!(
        result.content.len(),
        1,
        "expected exactly one content block"
    );
    match &result.content[0] {
        ContentBlock::Text(text) => text.text.clone(),
        other => panic!("expected text, got {other:?}"),
    }
}
