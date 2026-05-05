use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use crate::kernel::memory::ContextOS;

pub fn run() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut os = ContextOS::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) if !l.trim().is_empty() => l,
            Ok(_) => continue,
            Err(_) => break, // stdin closed — shut down cleanly
        };

        let msg: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Notifications have no "id" and need no response
        let is_notification = msg.get("id").is_none();
        let id = msg.get("id").cloned().unwrap_or(Value::Null);
        let method = msg["method"].as_str().unwrap_or("");

        if is_notification {
            continue; // notifications/initialized, notifications/cancelled, etc.
        }

        let response = match method {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": {
                        "name": "truth-ctx",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }
            }),

            // Keep-alive ping — must respond with empty result
            "ping" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {}
            }),

            "tools/list" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": [
                        {
                            "name": "truth_check",
                            "description": "Audit the user prompt for tech stack pivots and return a Truth Anchor. Call this at the start of every response.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "prompt": {
                                        "type": "string",
                                        "description": "The user's raw prompt text"
                                    }
                                },
                                "required": ["prompt"],
                                "additionalProperties": false
                            }
                        },
                        {
                            "name": "truth_status",
                            "description": "Return the currently tracked tech stack state.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "additionalProperties": false
                            }
                        },
                        {
                            "name": "truth_reset",
                            "description": "Clear all tracked state. Use when starting a new project.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "additionalProperties": false
                            }
                        },
                        {
                            "name": "truth_audit",
                            "description": "Post-generation hallucination check. Pass the AI response text; returns a warning if it diverges from the user's original intent (cosine similarity < 0.85). Requires the semantic feature to be enabled.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "response": {
                                        "type": "string",
                                        "description": "The AI-generated response text to audit"
                                    }
                                },
                                "required": ["response"],
                                "additionalProperties": false
                            }
                        }
                    ]
                }
            }),

            "tools/call" => {
                let tool = msg["params"]["name"].as_str().unwrap_or("");
                let args = &msg["params"]["arguments"];

                match tool {
                    "truth_check" => {
                        let prompt = args["prompt"].as_str().unwrap_or("");
                        let pivot = os.detect_pivot(prompt);
                        os.save();
                        let anchored = os.inject_truth_anchor(prompt);

                        let text = match pivot {
                            Some(p) => format!("⚠ PIVOT: {}\n\n{}", p, anchored),
                            None => anchored,
                        };

                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": { "content": [{ "type": "text", "text": text }] }
                        })
                    }

                    "truth_status" => {
                        let block = os.state.to_anchor_block();
                        let text = if block.is_empty() {
                            "No tech stack tracked yet.".to_string()
                        } else {
                            format!("[TRUTH STATE]\n{}", block)
                        };
                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": { "content": [{ "type": "text", "text": text }] }
                        })
                    }

                    "truth_reset" => {
                        os.state.dimensions.clear();
                        os.save();
                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": { "content": [{ "type": "text", "text": "Truth state cleared." }] }
                        })
                    }

                    "truth_audit" => {
                        let response = args["response"].as_str().unwrap_or("");
                        let text = match os.audit_response(response) {
                            Some(warning) => warning,
                            None => "✓ Response aligns with original intent.".to_string(),
                        };
                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": { "content": [{ "type": "text", "text": text }] }
                        })
                    }

                    _ => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32601, "message": "Unknown tool" }
                    })
                }
            }

            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": "Method not found" }
            })
        };

        let _ = writeln!(out, "{}", serde_json::to_string(&response).unwrap());
        let _ = out.flush();
    }
}
