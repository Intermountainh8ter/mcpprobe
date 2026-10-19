//! Minimal Model Context Protocol client over stdio: newline-delimited
//! JSON-RPC 2.0, `initialize` handshake, `tools/list`, `tools/call`.
//! The server's stderr is discarded; only what it writes to stdout is read.

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};

pub const PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "inputSchema", default)]
    pub input_schema: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

pub struct McpClient {
    child: Child,
    stdin: ChildStdin,
    stdout: tokio::io::Lines<BufReader<ChildStdout>>,
    next_id: u64,
    pub timeout: Duration,
    pub server_info: serde_json::Value,
}

impl McpClient {
    pub async fn spawn(argv: &[String], cwd: Option<&std::path::Path>, timeout: Duration) -> anyhow::Result<Self> {
        anyhow::ensure!(!argv.is_empty(), "server command is empty");
        let mut cmd = tokio::process::Command::new(&argv[0]);
        cmd.args(&argv[1..]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).kill_on_drop(true);
        if let Some(d) = cwd { cmd.current_dir(d); }
        let mut child = cmd.spawn().map_err(|e| anyhow::anyhow!("spawn {}: {e}", argv[0]))?;
        let stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("no stdin"))?;
        let stdout = BufReader::new(child.stdout.take().ok_or_else(|| anyhow::anyhow!("no stdout"))?).lines();
        Ok(Self { child, stdin, stdout, next_id: 0, timeout, server_info: serde_json::Value::Null })
    }

    async fn send(&mut self, msg: &serde_json::Value) -> anyhow::Result<()> {
        let mut line = serde_json::to_string(msg)?;
        line.push('\n');
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(())
    }

    /// Send a request and wait for the response with the matching id, skipping notifications.
    pub async fn request(&mut self, method: &str, params: serde_json::Value) -> anyhow::Result<Result<serde_json::Value, RpcError>> {
        self.next_id += 1;
        let id = self.next_id;
        self.send(&serde_json::json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})).await?;
        let deadline = tokio::time::Instant::now() + self.timeout;
        loop {
            let line = tokio::time::timeout_at(deadline, self.stdout.next_line()).await.map_err(|_| anyhow::anyhow!("timeout waiting for {method}"))??;
            let Some(line) = line else { anyhow::bail!("server closed stdout during {method}") };
            let v: serde_json::Value = match serde_json::from_str(&line) { Ok(v) => v, Err(_) => continue };
            if v["id"].as_u64() != Some(id) { continue; }
            if let Some(err) = v.get("error") {
                return Ok(Err(RpcError { code: err["code"].as_i64().unwrap_or(0), message: err["message"].as_str().unwrap_or("").to_string() }));
            }
            return Ok(Ok(v["result"].clone()));
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        let r = self.request("initialize", serde_json::json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {"name": "mcpprobe", "version": env!("CARGO_PKG_VERSION")}
        })).await?;
        let r = r.map_err(|e| anyhow::anyhow!("initialize: {} ({})", e.message, e.code))?;
        self.server_info = r["serverInfo"].clone();
        self.send(&serde_json::json!({"jsonrpc": "2.0", "method": "notifications/initialized"})).await?;
        Ok(())
    }

    pub async fn tools_list(&mut self) -> anyhow::Result<Vec<Tool>> {
        let r = self.request("tools/list", serde_json::json!({})).await?.map_err(|e| anyhow::anyhow!("tools/list: {}", e.message))?;
        Ok(serde_json::from_value(r["tools"].clone())?)
    }

    pub async fn tools_call(&mut self, name: &str, args: serde_json::Value) -> anyhow::Result<Result<serde_json::Value, RpcError>> {
        self.request("tools/call", serde_json::json!({"name": name, "arguments": args})).await
    }

    pub async fn shutdown(mut self) {
        let _ = self.child.kill().await;
    }
}
