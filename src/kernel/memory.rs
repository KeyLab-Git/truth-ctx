use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use super::sentinel;

// ── Rule types ────────────────────────────────────────────────────────────────

struct BuiltinRule {
    dimension: &'static str,
    value:     &'static str,
    keywords:  &'static [&'static str],
}

#[derive(Deserialize)]
struct UserRule {
    dimension: String,
    value:     String,
    keywords:  Vec<String>,
}

#[derive(Deserialize)]
struct UserRuleFile {
    rules: Vec<UserRule>,
}

// ── Signals ───────────────────────────────────────────────────────────────────

// Expressing a preference — triggers silent registration on first occurrence
const PREF_SIGNALS: &[&str] = &[
    "use ", "using ", "go with ", "going with ", "let's use ", "lets use ",
    "i want ", "we want ", "we're using ", "please use ", "make it ",
    "set it to ", "keep it ", "prefer ",
];

// Explicitly changing something — triggers pivot reporting
const CHANGE_SIGNALS: &[&str] = &[
    "switch to", "switching to", "migrate to", "migrating to",
    "replace with", "instead of", "move to", "moving to",
    "change to", "changing to", "adopt", "use instead",
    "swap to", "ditch", "drop", "actually", "wait,",
    "switching from", "migrate from", "changing from", "no, use",
    "wait let", "actually let", "no longer", "not anymore",
];

// ── Built-in rules ────────────────────────────────────────────────────────────

const BUILTIN_RULES: &[BuiltinRule] = &[
    // Database
    BuiltinRule { dimension: "database",      value: "Firebase",          keywords: &["firebase", "firestore"] },
    BuiltinRule { dimension: "database",      value: "Supabase",          keywords: &["supabase"] },
    BuiltinRule { dimension: "database",      value: "MongoDB",           keywords: &["mongodb", "mongoose"] },
    BuiltinRule { dimension: "database",      value: "PostgreSQL",        keywords: &["postgresql", "postgres"] },
    BuiltinRule { dimension: "database",      value: "MySQL",             keywords: &["mysql"] },
    BuiltinRule { dimension: "database",      value: "SQLite",            keywords: &["sqlite"] },
    BuiltinRule { dimension: "database",      value: "PlanetScale",       keywords: &["planetscale"] },
    BuiltinRule { dimension: "database",      value: "Neon",              keywords: &["neon db", "neondb"] },
    BuiltinRule { dimension: "database",      value: "Turso",             keywords: &["turso"] },
    BuiltinRule { dimension: "database",      value: "Redis",             keywords: &["redis", "upstash"] },
    // Auth
    BuiltinRule { dimension: "auth",          value: "Firebase Auth",     keywords: &["firebase auth", "firebase authentication"] },
    BuiltinRule { dimension: "auth",          value: "Supabase Auth",     keywords: &["supabase auth", "supabase authentication"] },
    BuiltinRule { dimension: "auth",          value: "Auth0",             keywords: &["auth0"] },
    BuiltinRule { dimension: "auth",          value: "Clerk",             keywords: &["clerk"] },
    BuiltinRule { dimension: "auth",          value: "NextAuth",          keywords: &["nextauth", "next-auth", "next auth"] },
    BuiltinRule { dimension: "auth",          value: "Lucia",             keywords: &["lucia"] },
    BuiltinRule { dimension: "auth",          value: "BetterAuth",        keywords: &["better-auth", "better auth"] },
    // Framework
    BuiltinRule { dimension: "framework",     value: "React",             keywords: &["react", "react.js"] },
    BuiltinRule { dimension: "framework",     value: "Vue",               keywords: &["vue.js", "vuejs"] },
    BuiltinRule { dimension: "framework",     value: "Svelte",            keywords: &["svelte", "sveltekit"] },
    BuiltinRule { dimension: "framework",     value: "Angular",           keywords: &["angular"] },
    BuiltinRule { dimension: "framework",     value: "Solid",             keywords: &["solidjs", "solid.js"] },
    BuiltinRule { dimension: "framework",     value: "React Native",      keywords: &["react native"] },
    BuiltinRule { dimension: "framework",     value: "Flutter",           keywords: &["flutter"] },
    BuiltinRule { dimension: "framework",     value: "Expo",              keywords: &["expo"] },
    BuiltinRule { dimension: "framework",     value: "Next.js",           keywords: &["next.js", "nextjs"] },
    BuiltinRule { dimension: "framework",     value: "Nuxt",              keywords: &["nuxt"] },
    BuiltinRule { dimension: "framework",     value: "Remix",             keywords: &["remix"] },
    BuiltinRule { dimension: "framework",     value: "Astro",             keywords: &["astro"] },
    // Backend
    BuiltinRule { dimension: "backend",       value: "Express",           keywords: &["express.js", "expressjs"] },
    BuiltinRule { dimension: "backend",       value: "FastAPI",           keywords: &["fastapi"] },
    BuiltinRule { dimension: "backend",       value: "NestJS",            keywords: &["nestjs", "nest.js"] },
    BuiltinRule { dimension: "backend",       value: "Axum",              keywords: &["axum"] },
    BuiltinRule { dimension: "backend",       value: "Hono",              keywords: &["hono"] },
    BuiltinRule { dimension: "backend",       value: "Elysia",            keywords: &["elysia"] },
    BuiltinRule { dimension: "backend",       value: "Django",            keywords: &["django"] },
    BuiltinRule { dimension: "backend",       value: "Rails",             keywords: &["rails", "ruby on rails"] },
    // Styling
    BuiltinRule { dimension: "styling",       value: "Tailwind",          keywords: &["tailwind", "tailwindcss"] },
    BuiltinRule { dimension: "styling",       value: "CSS Modules",       keywords: &["css modules", "module.css"] },
    BuiltinRule { dimension: "styling",       value: "Styled Components", keywords: &["styled-components", "styled components"] },
    BuiltinRule { dimension: "styling",       value: "Emotion",           keywords: &["emotion", "css-in-js"] },
    BuiltinRule { dimension: "styling",       value: "Sass",              keywords: &["sass", "scss"] },
    BuiltinRule { dimension: "styling",       value: "UnoCSS",            keywords: &["unocss"] },
    // State management
    BuiltinRule { dimension: "state",         value: "Redux",             keywords: &["redux", "redux toolkit"] },
    BuiltinRule { dimension: "state",         value: "Zustand",           keywords: &["zustand"] },
    BuiltinRule { dimension: "state",         value: "MobX",              keywords: &["mobx"] },
    BuiltinRule { dimension: "state",         value: "Jotai",             keywords: &["jotai"] },
    BuiltinRule { dimension: "state",         value: "Recoil",            keywords: &["recoil"] },
    BuiltinRule { dimension: "state",         value: "Context API",       keywords: &["context api", "react context", "usecontext"] },
    BuiltinRule { dimension: "state",         value: "TanStack Query",    keywords: &["react query", "tanstack query"] },
    // UI library
    BuiltinRule { dimension: "ui-lib",        value: "shadcn/ui",         keywords: &["shadcn", "shadcn/ui"] },
    BuiltinRule { dimension: "ui-lib",        value: "Radix UI",          keywords: &["radix", "radix ui"] },
    BuiltinRule { dimension: "ui-lib",        value: "MUI",               keywords: &["mui", "material ui", "material-ui"] },
    BuiltinRule { dimension: "ui-lib",        value: "Ant Design",        keywords: &["ant design", "antd"] },
    BuiltinRule { dimension: "ui-lib",        value: "Chakra UI",         keywords: &["chakra ui", "chakra"] },
    BuiltinRule { dimension: "ui-lib",        value: "DaisyUI",           keywords: &["daisyui", "daisy ui"] },
    BuiltinRule { dimension: "ui-lib",        value: "Mantine",           keywords: &["mantine"] },
    // Color mode
    BuiltinRule { dimension: "color-mode",    value: "dark",              keywords: &["dark mode", "dark theme", "dark background"] },
    BuiltinRule { dimension: "color-mode",    value: "light",             keywords: &["light mode", "light theme", "light background"] },
    BuiltinRule { dimension: "color-mode",    value: "system",            keywords: &["system theme", "follow system", "prefers-color-scheme"] },
    // Primary color
    BuiltinRule { dimension: "primary-color", value: "blue",              keywords: &["blue primary", "blue accent", "blue color scheme", "primary blue", "blue theme"] },
    BuiltinRule { dimension: "primary-color", value: "green",             keywords: &["green primary", "green accent", "green theme"] },
    BuiltinRule { dimension: "primary-color", value: "red",               keywords: &["red primary", "red accent", "red theme"] },
    BuiltinRule { dimension: "primary-color", value: "purple",            keywords: &["purple primary", "purple accent", "purple theme"] },
    BuiltinRule { dimension: "primary-color", value: "orange",            keywords: &["orange primary", "orange accent", "orange theme"] },
    // Corner style
    BuiltinRule { dimension: "corners",       value: "rounded",           keywords: &["rounded corners", "round corners", "rounded buttons", "rounded cards", "rounded ui", "use rounded"] },
    BuiltinRule { dimension: "corners",       value: "sharp",             keywords: &["sharp corners", "square corners", "no border-radius", "sharp edges"] },
    BuiltinRule { dimension: "corners",       value: "pill",              keywords: &["pill buttons", "pill shape", "fully rounded", "pill-shaped"] },
    // Layout
    BuiltinRule { dimension: "layout",        value: "sidebar",           keywords: &["sidebar layout", "side navigation", "sidenav", "left sidebar"] },
    BuiltinRule { dimension: "layout",        value: "topnav",            keywords: &["top navigation", "topnav", "header nav"] },
    BuiltinRule { dimension: "layout",        value: "grid",              keywords: &["grid layout", "css grid", "grid-based layout"] },
    BuiltinRule { dimension: "layout",        value: "fullwidth",         keywords: &["full width", "full-width layout", "edge to edge"] },
    BuiltinRule { dimension: "layout",        value: "centered",          keywords: &["centered layout", "max-width container", "centered content"] },
    // Density
    BuiltinRule { dimension: "density",       value: "compact",           keywords: &["compact layout", "dense layout", "tight spacing", "compact ui"] },
    BuiltinRule { dimension: "density",       value: "spacious",          keywords: &["spacious layout", "lots of whitespace", "generous padding"] },
    // Design style
    BuiltinRule { dimension: "design-style",  value: "minimal",           keywords: &["minimal design", "minimalist", "clean design", "minimal ui"] },
    BuiltinRule { dimension: "design-style",  value: "glassmorphism",     keywords: &["glassmorphism", "frosted glass", "glass effect"] },
    BuiltinRule { dimension: "design-style",  value: "neumorphism",       keywords: &["neumorphism", "soft ui", "neumorphic"] },
    BuiltinRule { dimension: "design-style",  value: "flat",              keywords: &["flat design", "flat ui", "no shadows"] },
    BuiltinRule { dimension: "design-style",  value: "material",          keywords: &["material design", "material you"] },
    // Animation
    BuiltinRule { dimension: "animation",     value: "none",              keywords: &["no animation", "no transitions", "disable animation"] },
    BuiltinRule { dimension: "animation",     value: "subtle",            keywords: &["subtle animation", "minimal animation", "slight transitions"] },
    BuiltinRule { dimension: "animation",     value: "rich",              keywords: &["rich animation", "smooth animation", "fluid animation"] },
    // Typography
    BuiltinRule { dimension: "font-style",    value: "sans-serif",        keywords: &["sans-serif font", "sans serif font"] },
    BuiltinRule { dimension: "font-style",    value: "serif",             keywords: &["serif font", "classic font"] },
    BuiltinRule { dimension: "font-style",    value: "monospace",         keywords: &["monospace font", "mono font", "code font"] },
    BuiltinRule { dimension: "font-style",    value: "Inter",             keywords: &["inter font", "use inter"] },
    BuiltinRule { dimension: "font-style",    value: "Geist",             keywords: &["geist font", "use geist"] },
    // Runtime
    BuiltinRule { dimension: "runtime",       value: "Bun",               keywords: &["bun runtime", "using bun", "run with bun"] },
    BuiltinRule { dimension: "runtime",       value: "Node",              keywords: &["node.js", "nodejs"] },
    BuiltinRule { dimension: "runtime",       value: "Deno",              keywords: &["deno"] },
    // Hosting
    BuiltinRule { dimension: "hosting",       value: "Vercel",            keywords: &["vercel"] },
    BuiltinRule { dimension: "hosting",       value: "Netlify",           keywords: &["netlify"] },
    BuiltinRule { dimension: "hosting",       value: "Railway",           keywords: &["railway"] },
    BuiltinRule { dimension: "hosting",       value: "Fly.io",            keywords: &["fly.io", "flyio"] },
    BuiltinRule { dimension: "hosting",       value: "AWS",               keywords: &["aws", "amazon web services"] },
    BuiltinRule { dimension: "hosting",       value: "Cloudflare",        keywords: &["cloudflare pages", "cloudflare workers"] },
    // Language
    BuiltinRule { dimension: "language",      value: "TypeScript",        keywords: &["typescript"] },
    BuiltinRule { dimension: "language",      value: "JavaScript",        keywords: &["javascript", "vanilla js"] },
    BuiltinRule { dimension: "language",      value: "Python",            keywords: &["python"] },
    BuiltinRule { dimension: "language",      value: "Rust",              keywords: &["rustlang"] },
    BuiltinRule { dimension: "language",      value: "Go",                keywords: &["golang"] },
];

// ── Dynamic "VALUE for SUBJECT" extraction ────────────────────────────────────

const FOR_TRIGGERS: &[&str] = &["use ", "set ", "go with ", "make it "];
const SKIP_WORDS:   &[&str] = &["the", "a", "an", "all", "our", "my", "your", "this", "that", "it"];

fn extract_for_pairs(text: &str) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    for trigger in FOR_TRIGGERS {
        let mut haystack = text;
        loop {
            let Some(pos) = haystack.find(trigger) else { break };
            let after = &haystack[pos + trigger.len()..];
            if let Some(for_pos) = after.find(" for ") {
                let value = after[..for_pos].trim();
                let rest  = after[for_pos + 5..].trim();
                if let Some(subject) = rest.split_whitespace()
                    .find(|w| !SKIP_WORDS.contains(w))
                {
                    let subject = subject.trim_matches(|c: char| !c.is_alphanumeric());
                    if !value.is_empty() && !subject.is_empty()
                        && value.len() < 40 && subject.len() < 25
                        && !value.contains("//")
                    {
                        pairs.push((subject.to_string(), value.to_string()));
                    }
                }
            }
            haystack = &haystack[pos + trigger.len()..];
            if haystack.is_empty() { break; }
        }
    }
    pairs
}

// ── User-defined rules ────────────────────────────────────────────────────────

fn load_user_rules(rules_path: &PathBuf) -> Vec<(String, String, Vec<String>)> {
    if !rules_path.exists() { return Vec::new(); }
    let content = match std::fs::read_to_string(rules_path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let file: UserRuleFile = match serde_json::from_str(&content) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    file.rules.into_iter()
        .map(|r| (r.dimension, r.value, r.keywords))
        .collect()
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize)]
pub struct TruthState {
    /// Keyword-tracked decision map: dimension → current value.
    pub dimensions: HashMap<String, String>,

    /// Running mean of every preference-statement embedding seen this session.
    /// Used for session-level conceptual DRIFT detection.
    #[serde(default)]
    pub context_vec: Vec<f32>,

    /// How many embeddings have been averaged into `context_vec` so far.
    #[serde(default)]
    pub ctx_count: u32,

    /// First embedding recorded per dimension, used to catch paraphrased pivots
    /// that the keyword layer misses.
    #[serde(default)]
    pub anchor_vecs: HashMap<String, Vec<f32>>,

    /// The semantic vector of the most recent user instruction.
    /// Used for post-generation audit to catch instruction drift.
    #[serde(default)]
    pub latest_intent_vec: Vec<f32>,

    /// The raw text of the most recent user instruction.
    #[serde(default)]
    pub latest_intent_text: String,

    /// When true, the Stop hook audits every AI response before it reaches the user.
    #[serde(default)]
    pub audit_enabled: bool,
}

pub struct ContextOS {
    pub state: TruthState,
    state_path: PathBuf,
    rules: Vec<(String, String, Vec<String>)>,
}

fn state_file_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    let base = std::env::var("USERPROFILE")
        .unwrap_or_else(|_| "C:\\Users\\Default".to_string());
    #[cfg(not(target_os = "windows"))]
    let base = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(base).join(".truth-ctx").join("state.json")
}

impl TruthState {
    fn empty() -> Self {
        TruthState {
            dimensions:  HashMap::new(),
            context_vec: Vec::new(),
            ctx_count:   0,
            anchor_vecs: HashMap::new(),
            latest_intent_vec: Vec::new(),
            latest_intent_text: String::new(),
            audit_enabled: false,
        }
    }

    pub fn to_anchor_block(&self) -> String {
        if self.dimensions.is_empty() { return String::new(); }
        let mut entries: Vec<_> = self.dimensions.iter().collect();
        entries.sort_by_key(|(k, _)| k.as_str());
        entries.iter()
            .map(|(k, v)| format!("  {}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl ContextOS {
    pub fn new() -> Self {
        let path = state_file_path();
        let state = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_else(TruthState::empty)
        } else {
            TruthState::empty()
        };

        let mut rules: Vec<(String, String, Vec<String>)> = BUILTIN_RULES.iter()
            .map(|r| (
                r.dimension.to_string(),
                r.value.to_string(),
                r.keywords.iter().map(|s| s.to_string()).collect(),
            ))
            .collect();
        let rules_path = path.parent()
            .unwrap_or_else(|| std::path::Path::new(""))
            .join("rules.json");
        rules.extend(load_user_rules(&rules_path));

        Self { state, state_path: path, rules }
    }

    pub fn save(&self) {
        if let Some(dir) = self.state_path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.state) {
            let _ = std::fs::write(&self.state_path, json);
        }
    }

    // ── Three-layer pivot detection ───────────────────────────────────────────

    pub fn detect_pivot(&mut self, text: &str) -> Option<String> {
        let lower = text.to_lowercase();
        let maybe_new_vec = sentinel::try_embed(&lower);
        
        // Always update latest intent for the auditor
        if let Some(ref new_vec) = maybe_new_vec {
            self.state.latest_intent_vec = new_vec.clone();
            self.state.latest_intent_text = text.to_string();
        }

        let has_signal = PREF_SIGNALS.iter().chain(CHANGE_SIGNALS.iter())
            .any(|s| lower.contains(s));
        if !has_signal { return None; }

        let has_change = CHANGE_SIGNALS.iter().any(|s| lower.contains(s));
        let mut pivots: Vec<String> = Vec::new();

        // ── Layer 1: keyword rules ────────────────────────────────────────────
        for (dim, val, kws) in &self.rules {
            if !kws.iter().any(|kw| lower.contains(kw.as_str())) { continue; }
            match self.state.dimensions.get(dim).map(|v| v.as_str()) {
                Some(cur) if cur != val.as_str() => {
                    pivots.push(format!("[{}] {} → {}", dim.to_uppercase(), cur, val));
                    self.state.dimensions.insert(dim.clone(), val.clone());
                }
                None => { self.state.dimensions.insert(dim.clone(), val.clone()); }
                _ => {}
            }
        }

        // ── Layer 2: dynamic "VALUE for SUBJECT" extraction ───────────────────
        for (subject, value) in extract_for_pairs(&lower) {
            match self.state.dimensions.get(&subject).map(|v| v.clone()) {
                Some(cur) if cur != value => {
                    pivots.push(format!("[{}] {} → {}", subject.to_uppercase(), cur, value));
                    self.state.dimensions.insert(subject, value);
                }
                None => { self.state.dimensions.insert(subject, value); }
                _ => {}
            }
        }

        // ── Layer 3: semantic vector sentinel ────────────────────────────────
        //
        //  3a. SESSION DRIFT — compares the new message to the running context
        //      mean. A high cosine distance means the conversation entered a
        //      conceptually different space even if no keywords triggered.
        //
        //  3b. IMPLICIT PIVOT — compares the new message to per-dimension anchor
        //      embeddings. High similarity to an existing anchor + change signal
        //      but no keyword match = paraphrased pivot.
        //
        //  3c. ANCHOR MAINTENANCE — stores/updates the embedding for each tracked
        //      dimension, and updates the session context mean.
        //
        if let Some(new_vec) = maybe_new_vec {
            // 3a — session drift
            if !self.state.context_vec.is_empty() {
                let dist = sentinel::cosine_distance(&self.state.context_vec, &new_vec);
                if dist > sentinel::DRIFT_THRESHOLD {
                    pivots.push(format!(
                        "[SEMANTIC] Conceptual drift detected (distance: {:.2}) — intent may have shifted",
                        dist
                    ));
                }
            }

            // 3b — implicit pivot: scan anchors, but only warn when there's an
            //      explicit change signal (avoids noise on normal conversation).
            if has_change {
                let implicit: Vec<String> = self.state.anchor_vecs.iter()
                    .filter_map(|(dim, anchor_vec)| {
                        let sim = sentinel::cosine_similarity(anchor_vec, &new_vec);
                        // Close enough to be "about" the same dimension …
                        let on_topic = sim > sentinel::ANCHOR_THRESHOLD;
                        // … but keywords didn't already catch a pivot for it.
                        let keyword_missed = !pivots.iter()
                            .any(|p| p.contains(&format!("[{}]", dim.to_uppercase())));
                        if on_topic && keyword_missed {
                            Some(format!(
                                "[SEMANTIC] Implicit re-address of '{}' (similarity: {:.2}) — possible unlabelled pivot",
                                dim, sim
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();
                pivots.extend(implicit);
            }

            // 3c — anchor maintenance: store first embedding per dimension,
            //      then update the session context mean.
            let dims_without_anchor: Vec<String> = self.state.dimensions.keys()
                .filter(|d| !self.state.anchor_vecs.contains_key(*d))
                .cloned()
                .collect();
            for dim in dims_without_anchor {
                self.state.anchor_vecs.insert(dim, new_vec.clone());
            }

            sentinel::update_avg(
                &mut self.state.context_vec,
                self.state.ctx_count,
                &new_vec,
            );
            self.state.ctx_count += 1;
        }

        if pivots.is_empty() { None } else {
            Some(format!("PIVOT DETECTED: {} | Anchoring new state.", pivots.join(" | ")))
        }
    }

    pub fn inject_truth_anchor(&self, prompt: &str) -> String {
        let anchor = self.state.to_anchor_block();
        if anchor.is_empty() { return prompt.to_string(); }
        super::scheduler::generate_final_prompt(prompt, &anchor)
    }

    pub fn enable_audit(&mut self) {
        self.state.audit_enabled = true;
        self.save();
    }

    /// Keyword-only contradiction check — works without the semantic feature.
    /// Scans the AI response for tech keywords that contradict the anchored stack.
    /// Does NOT update state.
    pub fn check_response_contradictions(&self, response: &str) -> Option<String> {
        let lower = response.to_lowercase();
        let mut hits: Vec<String> = Vec::new();

        for (dim, val, kws) in &self.rules {
            let Some(anchored) = self.state.dimensions.get(dim) else { continue };
            if anchored == val { continue; }
            if kws.iter().any(|kw| lower.contains(kw.as_str())) {
                hits.push(format!(
                    "[{}] response mentions '{}' but stack is anchored to '{}'",
                    dim.to_uppercase(), val, anchored
                ));
            }
        }

        if hits.is_empty() {
            None
        } else {
            Some(format!("⚠ RESPONSE CONTRADICTION: {}", hits.join(" | ")))
        }
    }

    /// Post-generation audit: embed `response` and compare it to the last recorded
    /// user intent vector. Returns `Some(warning)` when similarity < AUDIT_THRESHOLD,
    /// meaning the response has likely drifted from what the user actually asked for.
    pub fn audit_response(&self, response: &str) -> Option<String> {
        if self.state.latest_intent_vec.is_empty() {
            return None;
        }
        let response_vec = sentinel::try_embed(response)?;
        let sim = sentinel::cosine_similarity(&self.state.latest_intent_vec, &response_vec);
        if sim < sentinel::AUDIT_THRESHOLD {
            Some(format!(
                "⚠ HALLUCINATION RISK: Response diverges from original intent \
                 (similarity: {:.2}, threshold: {:.2}). Original intent: \"{}\"",
                sim,
                sentinel::AUDIT_THRESHOLD,
                self.state.latest_intent_text,
            ))
        } else {
            None
        }
    }
}
