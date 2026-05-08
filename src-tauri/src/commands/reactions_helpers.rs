//! Shared helpers for the per-chat `reactions` setting.
//!
//! The setting moved from a boolean (true/false) to a three-mode value
//! ("off" / "occasional" / "always") per Ryan's design discussion. The
//! setting key in the DB stays `reactions_enabled.<id>` for backwards-
//! compat; the value-space expanded.
//!
//! Backwards-compat parsing:
//!   - missing                  → "off"   (current default per a8a7b0c)
//!   - "off" | "false"          → "off"
//!   - "occasional"             → "occasional"
//!   - "always" | "true" | "on" → "always"
//!   - anything else            → "off"   (safe default)
//!
//! "Occasional" mode is NOT implemented as deterministic-skip in code.
//! Per Ryan's calibration note: let the LLM decide whether the moment
//! fits a reaction, with a 25% budget the LLM self-paces against by
//! looking at recent reactions in chat history. The mode value is
//! threaded into `pick_character_reaction_via_llm` which adjusts its
//! prompt accordingly.

/// Parse the stored setting value into one of three canonical modes.
/// Returns a static string so callers don't have to allocate.
pub fn parse_reactions_mode(setting_value: Option<&str>) -> &'static str {
    match setting_value
        .map(|s| s.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("always") | Some("true") | Some("on") => "always",
        Some("occasional") => "occasional",
        Some("off") | Some("false") | None => "off",
        _ => "off",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_handles_all_canonical_and_legacy_values() {
        assert_eq!(parse_reactions_mode(Some("off")), "off");
        assert_eq!(parse_reactions_mode(Some("false")), "off");
        assert_eq!(parse_reactions_mode(None), "off");
        assert_eq!(parse_reactions_mode(Some("occasional")), "occasional");
        assert_eq!(parse_reactions_mode(Some("always")), "always");
        assert_eq!(parse_reactions_mode(Some("true")), "always");
        assert_eq!(parse_reactions_mode(Some("on")), "always");
        assert_eq!(parse_reactions_mode(Some("garbage")), "off");
    }
}
