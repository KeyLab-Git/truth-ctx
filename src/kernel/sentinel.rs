//! Semantic Vector Sentinel
//!
//! Converts text into embedding vectors (all-MiniLM-L6-v2, 384 dims) and uses
//! cosine similarity to detect two classes of pivot that keyword matching misses:
//!
//!   1. DRIFT — the new message is conceptually far from the established session
//!              context (e.g. switching from low-level systems talk to high-level
//!              web-framework talk without naming any specific tech).
//!
//!   2. IMPLICIT PIVOT — the new message is semantically close to an existing
//!              decision anchor but the keyword layer didn't flag it, meaning the
//!              user rephrased a preference change instead of naming it directly.
//!
//! The model is downloaded once (~30 MB) to the platform cache on first use.
//! Every subsequent run is fully offline.
//!
//! Activate with:  cargo build --features semantic

/// Cosine distance above this value triggers a DRIFT warning.
pub const DRIFT_THRESHOLD: f32 = 0.40;

/// Cosine similarity above this value means "same conceptual topic".
pub const ANCHOR_THRESHOLD: f32 = 0.72;

/// Cosine similarity below this value in the Post-Generation Audit triggers a correction.
pub const AUDIT_THRESHOLD: f32 = 0.85;

// ── Model singleton (semantic feature only) ───────────────────────────────────

#[cfg(feature = "semantic")]
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

#[cfg(feature = "semantic")]
use std::sync::OnceLock;

#[cfg(feature = "semantic")]
static MODEL: OnceLock<Option<TextEmbedding>> = OnceLock::new();

#[cfg(feature = "semantic")]
fn get_model() -> Option<&'static TextEmbedding> {
    MODEL.get_or_init(|| {
        TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                .with_show_download_progress(true),
        )
        .ok()
    })
    .as_ref()
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Embed a single text string.
/// Returns `None` when the semantic feature is disabled or the model failed to load.
pub fn try_embed(text: &str) -> Option<Vec<f32>> {
    #[cfg(feature = "semantic")]
    {
        let model = get_model()?;
        model
            .embed(vec![text.to_string()], None)
            .ok()?
            .into_iter()
            .next()
    }
    #[cfg(not(feature = "semantic"))]
    { let _ = text; None }
}

// ── Math ──────────────────────────────────────────────────────────────────────

/// Cosine similarity in [-1, 1].  1 = identical direction, 0 = orthogonal.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    (dot / (mag_a * mag_b)).clamp(-1.0, 1.0)
}

/// Cosine distance in [0, 2].  0 = identical, values > DRIFT_THRESHOLD signal drift.
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    1.0 - cosine_similarity(a, b)
}

/// Welford online algorithm — updates a running mean in-place without storing all samples.
/// `count` is the number of samples already averaged into `avg` before this call.
pub fn update_avg(avg: &mut Vec<f32>, count: u32, new_vec: &[f32]) {
    if avg.is_empty() {
        *avg = new_vec.to_vec();
        return;
    }
    if avg.len() != new_vec.len() {
        // dimension mismatch is a logic bug — skip rather than silently corrupt context
        return;
    }
    let n = (count + 1) as f32;
    for (a, b) in avg.iter_mut().zip(new_vec) {
        *a += (b - *a) / n;
    }
}
