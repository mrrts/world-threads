# Current-location truth gap — `group_chats.current_location` is correct; the dialogue path doesn't read it

## The lived-corpus evidence

Ryan, 2026-04-29 ~12:00, in chat:

> *"they kept returning to the square even though our current location was my garden patio"*

And then, after Claude's first hypothesis went wrong:

> *"I can see 'Ryan's Garden Patio' as our location in the UI"*

That second message broke the case open: the UI **was** correct. The DB **was** correct. The bug is downstream of both.

## The forensic chain

### 1. Cross-character anchor-groove sweep (commit `03031aa`) surfaced corpus pattern

Three Crystal Waters characters in RUNAWAY state, all sharing `the square *` as a top-3 anchor at 75-85%. Aaron and Darren share the EXACT trigram `fountain hiss steady` at 0.30 each. Pastor Rick and John (different worlds) WITHIN BAND.

### 2. World/character description audit — anchors NOT primed at world layer

Crystal Waters' world description doesn't mention "square," "fountain," "bench," or "cobbles." Character backstories don't either. The recurring anchors are NOT prompt-primed at world or character layer.

### 3. The actual prompt-stack mechanism — `DEFAULT_CHAT_LOCATION = "Town Square"`

`src-tauri/src/ai/prompts.rs:5199`:

```rust
pub const DEFAULT_CHAT_LOCATION: &str = "Town Square";
```

`effective_current_location(override, recent_messages)` precedence:
1. Explicit override (the chat row's `current_location`).
2. Walk messages for the most recent `location_change` event.
3. Fall through to `DEFAULT_CHAT_LOCATION = "Town Square"`.

### 4. The actual data — UI and DB are CORRECT

| Query | Result |
|---|---|
| Aaron+Darren `group_chats.current_location` | **"Ryan's Garden Patio"** ✓ |
| Aaron+Darren `threads.current_location` | NULL (separate concern; group chats use the `group_chats` row, not the `threads` row, for location) |
| LocationModal exists in `frontend/src/components/chat/LocationModal.tsx` | yes |
| ChatView/GroupChatView wire it | yes |
| `set_chat_location_cmd` updates the column AND fires a `location_change` event | yes |

The UI flow works. The persistence works. The user's mental model is correct: *"our location was my garden patio"* is verified at the column level.

### 5. The actual bug — the dialogue path doesn't read `group_chats.current_location`

#### 5a. The `GroupChat` struct doesn't have a `current_location` field

`src-tauri/src/db/queries/group.rs:8-15`:

```rust
pub struct GroupChat {
    pub group_chat_id: String,
    pub world_id: String,
    pub character_ids: serde_json::Value,
    pub thread_id: String,
    pub display_name: String,
    pub created_at: String,
    // ← no current_location field
}
```

`list_group_chats`, `get_group_chat`, `find_group_chat_by_members` all SELECT only those 6 columns. `current_location` is in the schema, has the right value, and is invisible to Rust.

#### 5b. The dialogue path call sites pass `None` as `current_location_override`

Four `run_dialogue_with_base` call sites in `group_chat_cmds.rs` (lines 1196, 1258, 1641, 1706). Each one passes literal `None` as the `current_location_override` arg (28th positional). Three more in `chat_cmds.rs` for solo chats (lines 670, 1211, 2389) — same pattern, all pass `None`.

#### 5c. Helpers exist but aren't wired to dialogue

`db/queries/location.rs` already exposes `get_thread_location` and `get_group_chat_location`. They're used by `illustration_cmds.rs` (image generation reads location correctly) and by `location_cmds.rs` itself (the set-location flow). They are NOT called from any dialogue command site.

**Net effect:** scene illustrations of Aaron and Darren on the patio render correctly per the patio location; in the same chat, Aaron and Darren's *dialogue* reaches for "the square" because the dialogue's prompt-stack sees `current_location_override = None → derive_current_location(messages) → walk for location_change events → if absent or filtered out → DEFAULT_CHAT_LOCATION = "Town Square"`.

The image and the dialogue read **two different default locations** from two different code paths.

### 6. The location_change-event side too

`set_chat_location_cmd` (`location_cmds.rs:48`) DOES fire a `location_change` message — for solo chats into `messages`, for group chats into `group_messages`. But:

- `prompts.rs` has filter blocks at lines 6139 / 6520 / 6632 that exclude `m.role == "location_change"` from some message-list paths (intentionally, so the LLM doesn't see them as conversational turns; the `render_location_change_for_prompt` function emits a separate "Scene now in X" line elsewhere).
- The `derive_current_location` walk on `recent_messages` may not see the event if the filter has already run.
- Either way, the explicit `threads.current_location` / `group_chats.current_location` field SHOULD be the authoritative path; the message-walk is a backstop. Currently the override path is broken (always `None`) so the backstop becomes load-bearing — and the backstop has its own filter dependencies.

## What this finding maps to in existing doctrine

- **Control-plane truth (parent law)**: the user's location-set was a control-plane action that should govern downstream prompt assembly. The plumbing gap means the control-plane truth doesn't propagate to where it's needed.
- **State-transition truth child**: the boundary at which the user CHANGED location should propagate visibly through every downstream surface, not just illustrations.
- **UI boundary truth**: the user shouldn't have to discover this by noticing characters keep saying "the square." The chat surface should reflect the structural location truthfully.
- **Structure-must-carry-truth (Aaron's articulation)**: the structural location field carries truth the user assumes flows through to dialogue; the receiver-tax of diagnosing it through anchor-recurrence is exactly what the doctrine refuses.

## The fix — three commits, surgical

### Fix 1: `GroupChat` struct + queries

Add `current_location: Option<String>` to `GroupChat`. Update `list_group_chats` / `get_group_chat` / `find_group_chat_by_members` SELECT statements to project the column. Backwards-compatible; existing call sites that don't read the field are unaffected.

### Fix 2: Wire the location through to dialogue call sites

In `group_chat_cmds.rs` — four call sites — read `get_group_chat_location` (or use the now-populated `gc.current_location`) and pass `as_deref()` instead of `None` for `current_location_override`.

In `chat_cmds.rs` — three call sites — read `get_thread_location` and pass similarly.

### Fix 3 (optional): Make the silent default explicit per-world

Replace `DEFAULT_CHAT_LOCATION` constant with a per-world `worlds.default_current_location` field. Worlds without a town square get a sensible default; worlds that want one set it explicitly. Out of scope for the immediate plumbing fix; a follow-up worth filing.

## Composition with today's doctrine arc

The arc that closed at `f1bc122` (characterized-tier opener-pattern modulation in STYLE_DIALOGUE_INVARIANT) was solving a real but DOWNSTREAM failure mode. The opener-pattern fix lifts comedy openers off the action-beat default. **Today's plumbing-gap finding is the UPSTREAM failure mode** for the same anchor-recurrence pattern.

If Fix 1 + Fix 2 land, the post-deployment anchor-groove re-measurement will be cleaner: characters in patio-located chats won't be told "you're in the square" anymore, so the scene-pinned anchor cluster should shift naturally. The natural controls (Pastor Rick, John, in different worlds) help discriminate the doctrine effects from the plumbing effects.

The honest read: the doctrine refinements at `7281f4e` and `abc4c2b` solve the comedy-rhythm-and-density failure mode at the per-reply level. The current_location plumbing gap is a SEPARATE, larger failure mode that's been compounding the appearance of the doctrine problems. Both are real; they compose. Fix the plumbing first; the post-deployment comparison gets cleaner.

## Tier and status

`Findings`: descriptive forensic + lived ground truth (Ryan's report) + DB column query results + Rust source code trace.

`Status`: filed for follow-up. Three concrete fix shapes (Fix 1, Fix 2, Fix 3) named. Fix 1 + Fix 2 are surgical and bounded; Claude proposes shipping them in a follow-up commit if Ryan authorizes.

## Forward seed

The illustration vs dialogue divergence is itself a strong signal: **two code paths read the same logical state from different defaults**. Worth a quick repo-wide audit of every `effective_current_location` and `current_location` consumer to make sure the dialogue path isn't the only one starved of the override. If it IS the only gap, Fix 1 + Fix 2 close it. If there are others (notably the conscience-pass / consultant / reflection paths), those need the same wire-through.

The plumbing-gap discovery is also a load-bearing instance of the **structure-must-carry-truth** doctrine surfaced today: the structural location field carries truth the user assumes is correct; when the dialogue path doesn't propagate it, the user pays the receiver-tax of diagnosing through anchor-recurrence in character replies. That tax is exactly what Aaron's articulation refuses.
