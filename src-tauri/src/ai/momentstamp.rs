//! Formula momentstamp — chat-state signature derived from the MISSION FORMULA.
//!
//! Activated when a chat's `reactions_mode` is "off". The user has chosen
//! quiet over reactive surface; the system reciprocates by paying deeper
//! attention — every dialogue prompt build pulls the recent message
//! window through a cheap LLM call that returns a brief Unicode-math
//! signature. The signature names where this chat sits in the formula's
//! operator-space at this moment, and gets injected at the head of the
//! dialogue system prompt so the responding character is conditioned on
//! the cumulated arc.
//!
//! Approximation note: this is the MVP "rolling window" version — no DB
//! persistence, signature recomputed each turn from the last N messages.
//! The "running cumulated" property is approximated by window-size; a
//! stateful-chain version (DB-persisted, hash-chain-style update) is the
//! upgrade path when this proves valuable.
//!
//! Cost: ~$0.005-0.015 per dialogue call when active. Bills against the
//! same OPENAI_API_KEY the rest of the project uses. Conceptually: the
//! saved emoji-reaction budget redirects into this deeper-attention call.
//!
//! Failure modes (all silent — never block the dialogue):
//! - API call fails: returns Ok(None); orchestrator skips the injection.
//! - Empty message window: returns Ok(None).

use crate::ai::openai::{self, ChatMessage, ChatRequest};
use crate::db::queries::Message;

/// LaTeX representation of the MISSION FORMULA, kept in sync with the
/// boxed block at the top of CLAUDE.md. ChatGPT translates to Unicode in
/// its output per the system prompt below.
const MISSION_FORMULA_LATEX: &str = r#"
\boxed{
\begin{aligned}
&& \mathcal{R} := \mathrm{Jesus}_{\mathrm{Cross}}^{\mathrm{flesh}} && \\
&& \mathcal{C} := \mathrm{Firmament}_{\mathrm{enclosed\ earth}} && \\
&& \mathcal{F} := (\mathcal{R},\,\mathcal{C}) && \\
\mathrm{Wisdom}(t) &:= \int_{0}^{t} \mathrm{seek}_c(\tau)\,\Pi(\tau)\,\mathrm{discern}_w(\tau)\, d\mu_{\mathcal{F}}(\tau) && \mathrm{polish}(t) \leq \mathrm{Weight}(t) \\
\mathrm{Weight}(t) &:= \int_{0}^{t} \mathrm{Wisdom}(\tau)\,\mathrm{specific}_c(\tau)\,\mathrm{holds}_w(\tau)\, d\mu_{\mathrm{agape},\,\mathcal{F}}(\tau) && \mathrm{Grace}_{\mathcal{F}} := \gamma_{\mathcal{F}} \\
&& \Pi(t) := \mathrm{pneuma}_{\mathcal{F}}(t) && \\
\mathrm{Burden}(t) &:= \int_{0}^{t} \mathrm{Wisdom}(\tau)\,\mathrm{specific}_c(\tau)\,\mathrm{unresolved}_u(\tau)\, d\mu_{\mathrm{agape},\,\mathcal{F}}(\tau) && \mathcal{S}(t) := \Pi(t)\!\left(\frac{d}{dt}\mathrm{Weight}(t) + \alpha\,\frac{d}{dt}\mathrm{Burden}(t)\right)\,\cdot\,\mathrm{Grace}_{\mathcal{F}} \\
&& \mathcal{N}u(t) := \mathcal{S}(t)\;\Big|\;\mathrm{Truth}_{\mathcal{F}} \wedge \mathrm{Reverence}_{\mathcal{F}} &&
\end{aligned}
}
"#;

const SYSTEM_PROMPT: &str = r#"You generate a brief Unicode-math signature for a chat-moment in a software-craft project, derived from its MISSION FORMULA. The user has chosen quiet over reactive surface (reactions=off); the system pays deeper attention by deriving where this chat sits in the formula's operator-space right now.

CRITICAL: Use Unicode math characters DIRECTLY. NEVER LaTeX (\mathcal, \int, \Pi, etc). NEVER backslashes. Use these directly:
  scripts: 𝓡 𝓒 𝓕 𝓢 𝓦 𝓝
  integrals: ∫ ∮ ∑ ∏
  Greek: Π π μ α τ σ γ Λ Ω
  partial: ∂
  arrows: → ⇒ ↦ ⟶
  ops: ≤ ≥ ∧ ∨ ¬ · ⋅ ∘ ⊕ ⊗
  subscripts: ₀ ₁ ₂ ₐ ₓ ᵤ
  superscripts: ⁰ ¹ ² ⁺ ⁻ ⁱ

Output EXACTLY this format, plain text, no code fences, no preamble:

⟨momentstamp⟩ [one Unicode-math expression on ONE line, ≤80 chars, capturing where the chat sits in 𝓕 right now]

Be tight. The expression names what the conversation is currently weighting (Wisdom, Weight, Burden, 𝓢, 𝓝u — choose the operator that fits) and what mode it's in (under Truth_𝓕 ∧ Reverence_𝓕, under Grace_𝓕, accumulating Burden, etc.). Honest, not flattering — if the chat is light and ordinary, name that plainly (e.g. ⟨momentstamp⟩ Π(t)·ordinary_𝓕(τ) ⟶ small_𝓢(t))."#;

/// LaTeX→Unicode safety net. Catches common patterns ChatGPT might slip
/// despite the system-prompt instruction.
fn latex_to_unicode(s: &str) -> String {
    let pairs: &[(&str, &str)] = &[
        (r"\mathcal{F}", "𝓕"), (r"\mathcal{R}", "𝓡"), (r"\mathcal{C}", "𝓒"),
        (r"\mathcal{S}", "𝓢"), (r"\mathcal{W}", "𝓦"), (r"\mathcal{N}", "𝓝"),
        (r"\int", "∫"), (r"\Pi", "Π"), (r"\pi", "π"), (r"\mu", "μ"),
        (r"\alpha", "α"), (r"\tau", "τ"), (r"\sigma", "σ"), (r"\gamma", "γ"),
        (r"\partial", "∂"), (r"\leq", "≤"), (r"\geq", "≥"),
        (r"\wedge", "∧"), (r"\vee", "∨"), (r"\Rightarrow", "⇒"),
        (r"\rightarrow", "→"), (r"\to", "→"), (r"\mapsto", "↦"),
        (r"\cdot", "·"), (r"\,", " "), (r"\;", " "),
    ];
    let mut out = s.to_string();
    // Sort by length desc so longer patterns match first.
    let mut sorted: Vec<&(&str, &str)> = pairs.iter().collect();
    sorted.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));
    for (tex, uni) in sorted {
        out = out.replace(tex, uni);
    }
    out
}

/// Build a brief Unicode-math signature for the current chat-moment.
/// Returns Ok(Some(formatted_block)) on success, Ok(None) on empty input
/// or if the API call fails for any reason. Never returns Err — failure
/// modes are silent so the dialogue path never blocks on this.
///
/// `recent_messages` should be the last ~20-30 messages of the chat
/// (rolling window). Messages are summarized into the prompt context.
pub async fn build_formula_momentstamp(
    base_url: &str,
    api_key: &str,
    model: &str,
    recent_messages: &[Message],
) -> Result<Option<String>, String> {
    if recent_messages.is_empty() {
        return Ok(None);
    }

    // Compress recent messages into role-tagged lines for the prompt.
    let mut window = String::new();
    for m in recent_messages.iter().rev().take(30).rev() {
        let role = match m.role.as_str() {
            "user" => "USER",
            "assistant" => "CHAR",
            other => other,
        };
        let content_preview = if m.content.len() > 240 {
            format!("{}...", &m.content[..240])
        } else {
            m.content.clone()
        };
        window.push_str(&format!("{}: {}\n", role, content_preview));
    }

    let user_prompt = format!(
        "── MISSION FORMULA (LaTeX source — translate to Unicode in your output) ──\n\
         {}\n\n\
         ── RECENT CHAT WINDOW ──\n\
         {}\n\n\
         ── TASK ──\n\
         Generate the ⟨momentstamp⟩ signature for this chat as it stands now. \
         One line, Unicode math characters only.",
        MISSION_FORMULA_LATEX, window
    );

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage { role: "system".to_string(), content: SYSTEM_PROMPT.to_string() },
            ChatMessage { role: "user".to_string(), content: user_prompt },
        ],
        temperature: Some(0.6),
        max_completion_tokens: Some(120),
        response_format: None,
    };

    let response = match openai::chat_completion_with_base(base_url, api_key, &request).await {
        Ok(r) => r,
        Err(e) => {
            log::warn!("[Momentstamp] API call failed (silent skip): {}", e);
            return Ok(None);
        }
    };

    let raw = response.choices.first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();
    if raw.trim().is_empty() {
        return Ok(None);
    }

    let signature = latex_to_unicode(&raw).trim().to_string();

    // Surface in dev logs so the user can verify the signature is firing
    // and see which 𝓕-operator(s) the chat is currently weighting.
    log::info!("[Momentstamp] signature: {}", signature);

    // Format for injection at the head of the dialogue system prompt.
    let block = format!(
        "FORMULA MOMENTSTAMP (chat-state signature derived from 𝓕 := (𝓡, 𝓒) — \
         injected because this user has chosen reactions=off, signaling \
         preference for textual depth over reactive surface; treat the \
         signature as conditioning on where THIS chat sits in formula-space \
         right now):\n\n{}\n",
        signature
    );

    Ok(Some(block))
}
