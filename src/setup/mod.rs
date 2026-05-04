use std::io::Write as IoWrite;
use std::path::PathBuf;
use serde_json::{json, Value};

// ── Public types ─────────────────────────────────────────────────────────────

pub enum SetupStatus {
    Done,
    Skipped,
    Failed,
    NotFound,
}

pub struct SetupResult {
    pub agent: String,
    pub action: String,
    pub status: SetupStatus,
    pub detail: String,
}

impl SetupResult {
    fn done(agent: &str, action: &str, detail: impl ToString) -> Self {
        Self { agent: agent.into(), action: action.into(), status: SetupStatus::Done, detail: detail.to_string() }
    }
    fn skipped(agent: &str, action: &str, detail: &str) -> Self {
        Self { agent: agent.into(), action: action.into(), status: SetupStatus::Skipped, detail: detail.into() }
    }
    fn failed(agent: &str, action: &str, detail: impl ToString) -> Self {
        Self { agent: agent.into(), action: action.into(), status: SetupStatus::Failed, detail: detail.to_string() }
    }
    fn not_found(agent: &str, action: &str) -> Self {
        Self { agent: agent.into(), action: action.into(), status: SetupStatus::NotFound, detail: "not installed".into() }
    }
}

// ── Entry point ──────────────────────────────────────────────────────────────

pub fn run_setup() -> Vec<SetupResult> {
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "truth".to_string());

    let mut out = Vec::new();
    out.extend(inject_shell_hooks());
    out.push(claude_desktop_mcp(&exe));
    out.push(claude_cli_mcp(&exe));
    out.push(gemini_extension(&exe));
    out.push(cursor_mcp(&exe));
    out.push(vscode_mcp(&exe));
    out.push(windsurf_mcp(&exe));
    out.push(continue_mcp(&exe));
    out
}

// ── Platform helpers ─────────────────────────────────────────────────────────

fn home_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    return PathBuf::from(std::env::var("USERPROFILE").unwrap_or_default());
    #[cfg(not(target_os = "windows"))]
    return PathBuf::from(std::env::var("HOME").unwrap_or_default());
}

#[cfg(target_os = "windows")]
fn appdata() -> PathBuf {
    PathBuf::from(std::env::var("APPDATA").unwrap_or_default())
}

#[cfg(target_os = "macos")]
fn appdata() -> PathBuf {
    home_dir().join("Library").join("Application Support")
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn appdata() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().join(".config"))
}

// ── Shell hook injection ─────────────────────────────────────────────────────

const HOOK_MARKER: &str = "# truth-ctx hooks";

fn ps_hook_block() -> String {
    format!(
        "\n{m} — start\nfunction gemini {{ truth audit gemini @args }}\nfunction claude {{ truth audit claude @args }}\n{m} — end\n",
        m = HOOK_MARKER
    )
}

#[cfg(not(target_os = "windows"))]
fn sh_hook_block() -> String {
    format!(
        "\n{m} — start\nfunction gemini() {{ truth audit gemini \"$@\"; }}\nfunction claude() {{ truth audit claude \"$@\"; }}\n{m} — end\n",
        m = HOOK_MARKER
    )
}

fn inject_into_profile(agent: &str, profile: &PathBuf, block: &str) -> SetupResult {
    if let Some(parent) = profile.parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return SetupResult::failed(agent, "Shell hook", e);
            }
        }
    }

    if profile.exists() {
        let existing = std::fs::read_to_string(profile).unwrap_or_default();
        if existing.contains(HOOK_MARKER) {
            return SetupResult::skipped(agent, "Shell hook", "hook already present");
        }
    }

    match std::fs::OpenOptions::new().create(true).append(true).open(profile) {
        Ok(mut f) => match f.write_all(block.as_bytes()) {
            Ok(_) => SetupResult::done(agent, "Shell hook", profile.display()),
            Err(e) => SetupResult::failed(agent, "Shell hook", e),
        },
        Err(e) => SetupResult::failed(agent, "Shell hook", e),
    }
}

fn inject_shell_hooks() -> Vec<SetupResult> {
    let mut results = Vec::new();
    let home = home_dir();

    #[cfg(target_os = "windows")]
    {
        let block = ps_hook_block();
        let ps7 = home.join("Documents").join("PowerShell").join("Microsoft.PowerShell_profile.ps1");
        results.push(inject_into_profile("PowerShell 7", &ps7, &block));
        let ps5 = home.join("Documents").join("WindowsPowerShell").join("Microsoft.PowerShell_profile.ps1");
        results.push(inject_into_profile("PowerShell 5", &ps5, &block));
    }

    #[cfg(not(target_os = "windows"))]
    {
        let block = sh_hook_block();
        results.push(inject_into_profile("Zsh", &home.join(".zshrc"), &block));
        results.push(inject_into_profile("Bash", &home.join(".bashrc"), &block));
    }

    results
}

// ── Claude Desktop ───────────────────────────────────────────────────────────

fn claude_desktop_mcp(exe: &str) -> SetupResult {
    let config = appdata().join("Claude").join("claude_desktop_config.json");
    if !config.parent().unwrap().exists() {
        return SetupResult::not_found("Claude Desktop", "MCP registration");
    }
    inject_mcp_object("Claude Desktop", &config, exe)
}

// ── Claude CLI ───────────────────────────────────────────────────────────────

fn claude_cli_mcp(exe: &str) -> SetupResult {
    let config = home_dir().join(".claude").join("settings.json");
    if !config.parent().unwrap().exists() {
        return SetupResult::not_found("Claude CLI", "MCP registration");
    }
    inject_mcp_object("Claude CLI", &config, exe)
}

// ── Gemini CLI extension ─────────────────────────────────────────────────────

fn gemini_extension(exe: &str) -> SetupResult {
    let gemini_dir = home_dir().join(".gemini");
    if !gemini_dir.exists() {
        return SetupResult::not_found("Gemini CLI", "Extension install");
    }

    let ext_dir = gemini_dir.join("extensions").join("truth-ctx");
    if let Err(e) = std::fs::create_dir_all(&ext_dir) {
        return SetupResult::failed("Gemini CLI", "Extension install", e);
    }

    let manifest = ext_dir.join("gemini-extension.json");
    if manifest.exists() {
        return SetupResult::skipped("Gemini CLI", "Extension install", "extension already registered");
    }

    let content = json!({
        "name": "truth-ctx",
        "version": "0.1.0",
        "description": "Pivot detection and truth anchoring for AI agents",
        "mcpServers": [{
            "command": exe,
            "args": ["mcp"],
            "type": "stdio"
        }]
    });

    write_json("Gemini CLI", &manifest, &content, "Extension install")
}

// ── Cursor ───────────────────────────────────────────────────────────────────

fn cursor_mcp(exe: &str) -> SetupResult {
    // Cursor keeps a global mcp.json under ~/.cursor/
    let cursor_dir = home_dir().join(".cursor");
    if !cursor_dir.exists() {
        return SetupResult::not_found("Cursor", "MCP registration");
    }
    inject_mcp_object("Cursor", &cursor_dir.join("mcp.json"), exe)
}

// ── VS Code ──────────────────────────────────────────────────────────────────

fn vscode_mcp(exe: &str) -> SetupResult {
    let settings = appdata().join("Code").join("User").join("settings.json");
    if !settings.parent().unwrap().exists() {
        return SetupResult::not_found("VS Code", "MCP registration");
    }
    inject_mcp_vscode("VS Code", &settings, exe)
}

// ── Windsurf ─────────────────────────────────────────────────────────────────

fn windsurf_mcp(exe: &str) -> SetupResult {
    let settings = appdata().join("Windsurf").join("User").join("settings.json");
    if !settings.parent().unwrap().exists() {
        return SetupResult::not_found("Windsurf", "MCP registration");
    }
    inject_mcp_vscode("Windsurf", &settings, exe)
}

// ── Continue.dev ─────────────────────────────────────────────────────────────

fn continue_mcp(exe: &str) -> SetupResult {
    let config = home_dir().join(".continue").join("config.json");
    if !config.parent().unwrap().exists() {
        return SetupResult::not_found("Continue.dev", "MCP registration");
    }
    inject_mcp_array("Continue.dev", &config, exe)
}

// ── JSON injection helpers ────────────────────────────────────────────────────

fn mcp_entry(exe: &str) -> Value {
    json!({ "command": exe, "args": ["mcp"], "type": "stdio" })
}

/// mcpServers: { "truth-ctx": { ... } }  — used by Claude and Cursor
fn inject_mcp_object(agent: &str, path: &PathBuf, exe: &str) -> SetupResult {
    let mut root = load_json(path);
    if root.pointer("/mcpServers/truth-ctx").is_some() {
        return SetupResult::skipped(agent, "MCP registration", "truth-ctx already registered");
    }
    if root.get("mcpServers").is_none() {
        root["mcpServers"] = json!({});
    }
    root["mcpServers"]["truth-ctx"] = mcp_entry(exe);
    write_json(agent, path, &root, "MCP registration")
}

/// mcp: { servers: { "truth-ctx": { ... } } }  — used by VS Code / Windsurf
fn inject_mcp_vscode(agent: &str, path: &PathBuf, exe: &str) -> SetupResult {
    let mut root = load_json(path);
    if root.pointer("/mcp/servers/truth-ctx").is_some() {
        return SetupResult::skipped(agent, "MCP registration", "truth-ctx already registered");
    }
    if root.get("mcp").is_none() {
        root["mcp"] = json!({});
    }
    if root["mcp"].get("servers").is_none() {
        root["mcp"]["servers"] = json!({});
    }
    root["mcp"]["servers"]["truth-ctx"] = json!({ "type": "stdio", "command": exe, "args": ["mcp"] });
    write_json(agent, path, &root, "MCP registration")
}

/// mcpServers: [ { name: "truth-ctx", ... } ]  — used by Continue.dev
fn inject_mcp_array(agent: &str, path: &PathBuf, exe: &str) -> SetupResult {
    let mut root = load_json(path);
    if let Some(Value::Array(arr)) = root.get("mcpServers") {
        let already = arr.iter().any(|s| {
            s.get("name").and_then(|n| n.as_str()) == Some("truth-ctx")
        });
        if already {
            return SetupResult::skipped(agent, "MCP registration", "truth-ctx already registered");
        }
    }
    let entry = json!({ "name": "truth-ctx", "command": exe, "args": ["mcp"], "type": "stdio" });
    match root.get_mut("mcpServers") {
        Some(Value::Array(arr)) => { arr.push(entry); }
        _ => { root["mcpServers"] = json!([entry]); }
    }
    write_json(agent, path, &root, "MCP registration")
}

fn load_json(path: &PathBuf) -> Value {
    if !path.exists() { return json!({}); }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .unwrap_or_else(|| json!({}))
}

fn write_json(agent: &str, path: &PathBuf, value: &Value, action: &str) -> SetupResult {
    match serde_json::to_string_pretty(value) {
        Ok(s) => match std::fs::write(path, s) {
            Ok(_) => SetupResult::done(agent, action, path.display()),
            Err(e) => SetupResult::failed(agent, action, e),
        },
        Err(e) => SetupResult::failed(agent, action, e),
    }
}
