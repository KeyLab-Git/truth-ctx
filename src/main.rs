mod kernel;
mod monitor;
mod mcp;
mod setup;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::{self, Read, Write};
use tokio::sync::mpsc;

// ANSI color codes — no extra dependency needed
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

#[derive(Parser)]
#[command(name = "truth", about = "Truth-Ctx — Context OS for AI agents")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Intercept an AI call, inject truth anchor, then launch the agent
    Audit {
        /// The AI agent binary to wrap (e.g. gemini, claude)
        agent: String,
        /// All arguments to forward to the agent
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Start the background file-system sentinel
    Start,
    /// Print shell hook snippets (PowerShell + Zsh) to stdout
    Hooks,
    /// Run as an MCP server over stdio (register in Claude/Gemini settings)
    Mcp,
    /// Run pivot-detection benchmark against sample prompts
    Bench,
    /// Remove truth-ctx state and optionally the binary itself
    Uninstall {
        /// Also delete the truth-ctx binary from disk
        #[arg(long)]
        bin: bool,
    },
    /// Auto-detect AI agents and inject shell hooks + MCP registration
    Setup,
}

fn gemini_history_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let local = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| {
            std::env::var("USERPROFILE")
                .map(|p| format!("{}\\AppData\\Local", p))
                .unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Local".to_string())
        });
        PathBuf::from(local).join("Google").join("Gemini").join("tmp")
    }
    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".gemini").join("tmp")
    }
}

fn state_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    let base = std::env::var("USERPROFILE")
        .unwrap_or_else(|_| "C:\\Users\\Default".to_string());
    #[cfg(not(target_os = "windows"))]
    let base = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(base).join(".truth-ctx")
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Audit { agent, args } => {
            eprint!("{CYAN}{BOLD}Calling truth-ctx...{RESET} ");

            let mut os = kernel::memory::ContextOS::new();

            let flags: Vec<&str> = args.iter()
                .filter(|a| a.starts_with('-'))
                .map(|s| s.as_str())
                .collect();
            let prompt_parts: Vec<&str> = args.iter()
                .filter(|a| !a.starts_with('-'))
                .map(|s| s.as_str())
                .collect();
            let raw_prompt = prompt_parts.join(" ");

            let pivot_msg = os.detect_pivot(&raw_prompt);
            if let Some(ref msg) = pivot_msg {
                eprintln!("\n{YELLOW}⚠  {}{RESET}", msg);
            } else {
                eprintln!("context OK");
            }

            os.save();
            let mut final_prompt = os.inject_truth_anchor(&raw_prompt);

            let mut attempts = 0;
            const MAX_ATTEMPTS: u32 = 3;

            loop {
                attempts += 1;
                #[cfg(target_os = "windows")]
                let mut cmd = {
                    let mut c = Command::new("cmd");
                    c.args(["/c", agent.as_str()]);
                    c
                };
                #[cfg(not(target_os = "windows"))]
                let mut cmd = Command::new(&agent);

                cmd.args(&flags);
                if !final_prompt.is_empty() {
                    cmd.arg(&final_prompt);
                }

                // Intercept output for audit
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::inherit());

                match cmd.spawn() {
                    Ok(mut child) => {
                        let mut output = String::new();
                        if let Some(mut stdout) = child.stdout.take() {
                            let _ = stdout.read_to_string(&mut output);
                        }

                        let _ = child.wait();

                        // ── Post-Generation Audit ────────────────────────────
                        if os.state.latest_intent_vec.is_empty() {
                            print!("{}", output);
                            let _ = io::stdout().flush();
                            break;
                        }

                        if let Some(output_vec) = kernel::sentinel::try_embed(&output) {
                            let sim = kernel::sentinel::cosine_similarity(
                                &os.state.latest_intent_vec,
                                &output_vec
                            );

                            if sim >= kernel::sentinel::AUDIT_THRESHOLD {
                                print!("{}", output);
                                let _ = io::stdout().flush();
                                break;
                            } else if attempts < MAX_ATTEMPTS {
                                eprintln!(
                                    "\n{RED}✘ Instruction Drift detected (similarity: {:.2}){RESET}",
                                    sim
                                );
                                eprintln!("{DIM}  Triggering automatic correction... (Attempt {}/{}){RESET}", attempts, MAX_ATTEMPTS);
                                
                                final_prompt = format!(
                                    "{}\n\n[CRITICAL CORRECTION]\nYour previous output drifted from my intent: \"{}\".\nRefactor your response to strictly adhere to this instruction. Do not ignore the context anchor provided earlier.",
                                    raw_prompt,
                                    os.state.latest_intent_text
                                );
                                continue;
                            } else {
                                eprintln!("{YELLOW}⚠ Maximum correction attempts reached. Outputting best effort.{RESET}");
                                print!("{}", output);
                                let _ = io::stdout().flush();
                                break;
                            }
                        } else {
                            print!("{}", output);
                            let _ = io::stdout().flush();
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("{RED}[Truth-Ctx] Failed to launch '{}': {}{RESET}", agent, e);
                        std::process::exit(1);
                    }
                }
            }
        }

        Commands::Start => {
            let path_str = gemini_history_path().to_string_lossy().into_owned();
            let (tx, mut rx) = mpsc::channel(100);

            tokio::spawn(async move {
                monitor::PivotMonitor::watch_history(path_str, tx).await;
            });

            eprintln!("{CYAN}{BOLD}[Truth-Ctx] OS Kernel Online. Sentinel active.{RESET}");
            let mut os = kernel::memory::ContextOS::new();

            while let Some(path) = rx.recv().await {
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        let content = String::from_utf8_lossy(&bytes);
                        if let Some(clash) = os.detect_pivot(&content) {
                            eprintln!("{YELLOW}🔥 Background Pivot: {}{RESET}", clash);
                            os.save();
                        }
                    }
                    Err(e) => eprintln!("{RED}[Truth-Ctx] Read error on '{}': {}{RESET}", path, e),
                }
            }
        }

        Commands::Hooks => {
            print_hooks();
        }

        Commands::Mcp => {
            mcp::run();
        }

        Commands::Bench => {
            run_bench();
        }

        Commands::Uninstall { bin } => {
            uninstall(bin);
        }

        Commands::Setup => {
            print_setup_results(setup::run_setup());
        }
    }
}

fn print_hooks() {
    println!(
        r#"
# ── Truth-Ctx Shell Hooks ──────────────────────────────────────────────────
#
# These wrappers make every `gemini` / `claude` call pass through truth-ctx
# automatically. Add them to your shell profile and reload.
#
# ── PowerShell (add to $PROFILE) ───────────────────────────────────────────

function gemini {{
    truth audit gemini @args
}}

function claude {{
    truth audit claude @args
}}

# ── Zsh / Bash (add to ~/.zshrc or ~/.bashrc) ──────────────────────────────

function gemini() {{
    truth audit gemini "$@"
}}

function claude() {{
    truth audit claude "$@"
}}

# ── Usage ───────────────────────────────────────────────────────────────────
# After adding the hooks, run:
#   . $PROFILE          # PowerShell
#   source ~/.zshrc     # Zsh
#
# Then just use `gemini` or `claude` normally. Truth-Ctx intercepts every call.
"#
    );
}

fn run_bench() {
    println!("Benchmarking not yet implemented.");
}

fn uninstall(remove_bin: bool) {
    println!("{CYAN}{BOLD}Truth-Ctx Uninstall{RESET}");
    println!();

    let dir = state_dir();
    if dir.exists() {
        match std::fs::remove_dir_all(&dir) {
            Ok(_) => println!("{GREEN}✓ Removed state directory: {}{RESET}", dir.display()),
            Err(e) => println!("{RED}✗ Failed to remove {}: {}{RESET}", dir.display(), e),
        }
    } else {
        println!("{DIM}  State directory not found — already clean{RESET}");
    }

    match std::env::current_exe() {
        Ok(exe) => {
            if remove_bin {
                match std::fs::remove_file(&exe) {
                    Ok(_) => println!("{GREEN}✓ Removed binary: {}{RESET}", exe.display()),
                    Err(e) => {
                        println!("{YELLOW}⚠  Binary could not be auto-removed (normal on Windows while running){RESET}");
                        println!("{DIM}  Delete manually:{RESET}");
                        println!("      {}", exe.display());
                        println!("{DIM}  Error: {}{RESET}", e);
                    }
                }
            } else {
                println!("{DIM}  Binary left at: {}{RESET}", exe.display());
                println!("{DIM}  Run `truth uninstall --bin` to also remove it{RESET}");
            }
        }
        Err(e) => println!("{RED}✗ Could not locate binary: {}{RESET}", e),
    }

    println!();
    println!("{CYAN}{BOLD}Shell hook cleanup{RESET}");
    println!("{DIM}  Remove these lines from your shell profile ($PROFILE / ~/.zshrc / ~/.bashrc):{RESET}");
    println!();
    println!("  # PowerShell");
    println!("  function gemini {{ truth audit gemini @args }}");
    println!("  function claude  {{ truth audit claude  @args }}");
    println!();
    println!("  # Zsh / Bash");
    println!("  function gemini() {{ truth audit gemini \"$@\" }}");
    println!("  function claude()  {{ truth audit claude  \"$@\" }}");
}

fn print_setup_results(results: Vec<setup::SetupResult>) {
    println!("{CYAN}{BOLD}Truth-Ctx Setup{RESET}");
    println!();

    let col_a = 16usize;
    let col_b = 20usize;
    let line = "─".repeat(col_a + col_b + 28);

    println!("{DIM}{}{RESET}", line);
    println!(
        "{DIM}{:<col_a$}  {:<col_b$}  {:<8}  {}{RESET}",
        "Agent", "Action", "Status", "Detail"
    );
    println!("{DIM}{}{RESET}", line);

    let mut done = 0u32;
    let mut skipped = 0u32;
    let mut failed = 0u32;
    let mut not_found = 0u32;

    for r in &results {
        let (icon, color) = match r.status {
            setup::SetupStatus::Done     => { done += 1;      ("✓ done",    GREEN)  }
            setup::SetupStatus::Skipped  => { skipped += 1;   ("○ skip",    DIM)    }
            setup::SetupStatus::Failed   => { failed += 1;    ("✗ fail",    RED)    }
            setup::SetupStatus::NotFound => { not_found += 1; ("– n/a",     DIM)    }
        };

        // Truncate detail so it fits in the terminal
        let detail = if r.detail.len() > 52 {
            format!("…{}", &r.detail[r.detail.len() - 51..])
        } else {
            r.detail.clone()
        };

        println!(
            "{:<col_a$}  {:<col_b$}  {color}{:<8}{RESET}  {DIM}{}{RESET}",
            r.agent, r.action, icon, detail
        );
    }

    println!("{DIM}{}{RESET}", line);
    println!(
        "  {GREEN}{done} done{RESET}  {DIM}{skipped} skipped  {not_found} not found{RESET}  {RED}{failed} failed{RESET}"
    );
    println!();

    if done > 0 {
        println!("{CYAN}Reload your shell profile for hook changes to take effect:{RESET}");
        println!("{DIM}  PowerShell:  . $PROFILE{RESET}");
        println!("{DIM}  Zsh/Bash:    source ~/.zshrc{RESET}");
    }
}
