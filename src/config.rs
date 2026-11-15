//! Every environment variable the server reads, read in one place.
//!
//! The one exception is `MCP_AUTH_TOKEN`, which `http::auth` reads at the moment the HTTP
//! transport starts: it is a secret, and the stdio path must never touch it.

use std::collections::HashMap;

/// The default HTTP port.
pub const DEFAULT_PORT: u16 = 3000;

/// The default HTTP interface: every IPv4 interface, named rather than implied.
pub const DEFAULT_HOST: &str = "0.0.0.0";

/// Which transport serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    Stdio,
    Http,
}

/// The server's configuration. Built once at the entry point and passed down; never mutated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// `MCP_TRANSPORT`: `http` or `streamable-http` (any case) is HTTP, anything else stdio.
    pub transport: Transport,
    /// `PORT`, 3000 when unset.
    pub port: u16,
    /// `HOST`, `0.0.0.0` when unset or empty.
    pub host: String,
    /// Whether `host` is loopback, so HTTP may run without a token.
    pub local: bool,
    /// `MCP_CLUSTER_WORKERS`: HTTP worker threads. Unset or unusable is one per CPU.
    pub workers: usize,
    /// `MCP_ALLOWED_HOSTS`, split on commas, trimmed, empties dropped. Empty applies no check.
    pub allowed_hosts: Vec<String>,
}

/// Why a configuration cannot be used. Names the variable and never guesses a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError(pub String);

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ConfigError {}

/// The leading integer of `raw`, the way `parseInt` reads it: optional whitespace and sign,
/// then digits, ignoring whatever follows. `None` when there are no digits.
fn leading_integer(raw: &str) -> Option<i64> {
    let trimmed = raw.trim_start();
    let (sign, digits) = match trimmed.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };
    let end = digits
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(digits.len());
    digits[..end].parse::<i64>().ok().map(|n| sign * n)
}

/// A comma-separated list, trimmed, with empty entries dropped.
pub fn split_list(raw: Option<&str>) -> Vec<String> {
    raw.unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Whether `host` is a loopback address, which only this machine can reach.
///
/// `localhost`, any `127.x.x.x`, and `::1` with or without brackets. `0.0.0.0` and `::` mean
/// every interface, so they are not loopback; neither is an empty value.
pub fn is_loopback(host: &str) -> bool {
    let value = host.trim().to_lowercase();
    if value == "localhost" || value == "::1" || value == "[::1]" {
        return true;
    }
    let octets: Vec<&str> = value.split('.').collect();
    octets.len() == 4
        && octets[0] == "127"
        && octets[1..].iter().all(|octet| {
            (1..=3).contains(&octet.len()) && octet.chars().all(|c| c.is_ascii_digit())
        })
}

/// How many HTTP worker threads to run: `MCP_CLUSTER_WORKERS` when it is a whole number of zero
/// or more, otherwise the number of CPUs this process was actually given.
fn worker_count(raw: Option<&str>, cpus: usize) -> usize {
    raw.and_then(leading_integer)
        .and_then(|n| usize::try_from(n).ok())
        .unwrap_or(cpus)
}

/// The number of CPUs available to this process, or 1 when it cannot be told.
pub fn available_parallelism() -> usize {
    std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
}

/// The configuration, from `env`.
pub fn load(env: &HashMap<String, String>) -> Result<Config, ConfigError> {
    let get = |key: &str| env.get(key).map(String::as_str);

    let transport = match get("MCP_TRANSPORT")
        .unwrap_or("stdio")
        .to_lowercase()
        .as_str()
    {
        "http" | "streamable-http" => Transport::Http,
        _ => Transport::Stdio,
    };

    let port = match get("PORT") {
        None => DEFAULT_PORT,
        Some(raw) => leading_integer(raw)
            .and_then(|n| u16::try_from(n).ok())
            .ok_or_else(|| {
                ConfigError(format!(
                    "PORT must be a port number from 0 to 65535, got \"{raw}\""
                ))
            })?,
    };

    let host = get("HOST")
        .filter(|h| !h.is_empty())
        .unwrap_or(DEFAULT_HOST)
        .to_owned();

    Ok(Config {
        transport,
        port,
        local: is_loopback(&host),
        host,
        workers: worker_count(get("MCP_CLUSTER_WORKERS"), available_parallelism()),
        allowed_hosts: split_list(get("MCP_ALLOWED_HOSTS")),
    })
}

/// The configuration from this process's environment.
pub fn from_process_env() -> Result<Config, ConfigError> {
    load(&std::env::vars().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    #[test]
    fn an_empty_environment_gives_the_documented_defaults() {
        assert_eq!(
            load(&env(&[])).unwrap(),
            Config {
                transport: Transport::Stdio,
                port: 3000,
                host: "0.0.0.0".into(),
                local: false,
                workers: available_parallelism(),
                allowed_hosts: vec![],
            }
        );
    }

    #[test]
    fn every_variable_is_read_and_the_transport_is_case_insensitive() {
        let config = load(&env(&[
            ("MCP_TRANSPORT", "HTTP"),
            ("PORT", "8080"),
            ("HOST", "127.0.0.1"),
            ("MCP_CLUSTER_WORKERS", "1"),
            ("MCP_ALLOWED_HOSTS", " a.example , ,b.example,"),
        ]))
        .unwrap();
        assert_eq!(config.transport, Transport::Http);
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.workers, 1);
        assert_eq!(config.allowed_hosts, ["a.example", "b.example"]);
        assert_eq!(
            load(&env(&[("MCP_TRANSPORT", "Streamable-HTTP")]))
                .unwrap()
                .transport,
            Transport::Http
        );
    }

    #[test]
    fn an_unusable_worker_count_falls_back_to_one_per_cpu() {
        for value in ["", "-1", "many"] {
            assert_eq!(
                load(&env(&[("MCP_CLUSTER_WORKERS", value)]))
                    .unwrap()
                    .workers,
                available_parallelism()
            );
        }
        assert_eq!(
            load(&env(&[("MCP_CLUSTER_WORKERS", "0")])).unwrap().workers,
            0
        );
    }

    #[test]
    fn an_unusable_port_is_refused_with_its_name() {
        for value in ["", "http", "70000", "-1"] {
            let error = load(&env(&[("PORT", value)])).unwrap_err();
            assert!(error.0.starts_with("PORT must be a port number"), "{value}");
        }
    }

    #[test]
    fn only_a_loopback_bind_counts_as_local() {
        for host in [
            "127.0.0.1",
            "127.1.2.3",
            "localhost",
            "LOCALHOST",
            "::1",
            "[::1]",
        ] {
            assert!(is_loopback(host), "{host}");
            assert!(load(&env(&[("HOST", host)])).unwrap().local, "{host}");
        }
        for host in [
            "0.0.0.0",
            "::",
            "",
            "10.0.0.5",
            "192.168.1.2",
            "example.com",
            "127.0.0.1.example.com",
            "127.0.0.1234",
        ] {
            assert!(!is_loopback(host), "{host}");
        }
        assert!(
            !load(&env(&[])).unwrap().local,
            "the default bind, every interface, is a server"
        );
    }
}
