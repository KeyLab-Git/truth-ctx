

```text
  ████████╗██████╗ ██╗   ██╗████████╗██╗  ██╗          ██████╗████████╗██╗  ██╗
  ╚══██╔══╝██╔══██╗██║   ██║╚══██╔══╝██║  ██║         ██╔════╝╚══██╔══╝╚██╗██╔╝
     ██║   ██████╔╝██║   ██║   ██║   ███████║         ██║        ██║    ╚███╔╝ 
     ██║   ██╔══██╗██║   ██║   ██║   ██╔══██║         ██║        ██║    ██╔██╗ 
     ██║   ██║  ██║╚██████╔╝   ██║   ██║  ██║    ██╗  ╚██████╗   ██║   ██╔╝ ██╗
     ╚═╝   ╚═╝  ╚═╝ ╚═════╝    ╚═╝   ╚═╝  ╚═╝    ╚═╝   ╚═════╝   ╚═╝   ╚═╝  ╚═╝
                    The Sentinel Kernel for AI Context

```

# 🛡️ Truth-Ctx: The Context OS & Sentinel
> **"Guard the Truth. Block the Noise. Own the Context."**

[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows-blue.svg)](#-cross-platform)
[![License](https://img.shields.io/badge/License-Zero--Telemetry-green.svg)](#-zero-telemetry-promise)

---

## 🌩️ The Problem: "Context Decay"
When conversations with AI agents like Gemini or Claude get long, they inevitably drift.

*   **Attention Decay:** Instructions buried in the middle of a chat are ignored.
*   **Instruction Drift:** You pivot (e.g., "Change buttons from blue to red"), but 4 messages later, the AI hallucinates the old "blue" state.
*   **Architectural Leaks:** The AI suggests Firebase logic in your Supabase project because it forgot the "Truth".

---

## ⚡ The Solution: The Sentinel Architecture
**Truth-Ctx** isn't just a helper; it’s a **Runtime Kernel** for your AI sessions. It stands between your intent and the AI's execution to enforce absolute integrity.

### 🧠 **Tech Stack & Intelligence**
| Feature | Technology | Function |
| :--- | :--- | :--- |
| **Kernel** | **Pure Rust** | Near-zero latency context management. |
| **Audit Engine** | **Cosine Similarity** | Mathematically detects intent pivots[cite: 1]. |
| **Logic Guard** | **Local Embeddings** | Runs 100% offline; no API calls required[cite: 1]. |
| **Interceptors** | **Shell Hooks** | Hijacks `gemini` and `claude` commands to audit prompts[cite: 1]. |

---

## 🛡️ Key Pillars

### 🛰️ **The Sentinel (Active Auditor)**
The Sentinel watches every byte the AI generates. If the AI output mathematically drifts away from your latest decision (e.g., it uses blue instead of red), the Sentinel **intercepts** and **auto-corrects** the code before it ever hits your terminal[cite: 1].

### 🔒 **Zero-Telemetry Promise**
In alignment with the **KeyLabs** vision, Truth-Ctx is a closed loop.
*   **No Phoning Home:** No usage statistics or private code snippets are ever transmitted[cite: 1].
*   **Local-First State:** Your project memory stays on your disk in Valencia City, not in a cloud training set[cite: 1].

### 🧵 **Cross-Session Memory**
Standard chats have amnesia. Truth-Ctx provides a "bridge" so your AI remembers the rules of **Stride**, **Ledgr**, or **Kalaya** across different chat windows and platforms[cite: 1].

---

## 📥 Getting Started

### **1. Installation**
```bash
# Install via the Rust toolchain
cargo install truth-ctx
