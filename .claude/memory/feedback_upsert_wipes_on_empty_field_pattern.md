---
name: Upsert blindly overwriting field-from-empty-payload silently wipes work done by sibling commands that set the same field out-of-band
description: 2026-05-11 lifted from GenesisModal self-avatar import bug — saveSelfProfile passed avatar_file: "" because it didn't track the avatar set out-of-band by setUserAvatarFromGallery; backend upsert overwrote unconditionally; symptom was "first import leaves blank avatar, second works." Pattern to recognize: any time a frontend payload hard-codes an empty field that a sibling command sets, the upsert wipes it.
type: feedback
originSessionId: eb4ee4e0-7b93-4fc6-9c17-1bbab6e06eb2
---

## The pattern

A backend upsert query writes ALL fields on UPDATE unconditionally:

```sql
ON CONFLICT(world_id) DO UPDATE SET
  display_name=?2, description=?3, ..., avatar_file=?6, ...
```

A frontend call site hard-codes empty for a field it doesn't track:

```ts
await api.updateUserProfile({
  display_name: name.trim() || "Me",
  avatar_file: "",  // ← caller doesn't track this; field set by sibling cmd
  ...
});
```

A sibling command set that field out-of-band:

```ts
await api.setUserAvatarFromGallery(...);  // ← DB.avatar_file = "user_x.png"
// ... then later ...
await api.updateUserProfile({ ..., avatar_file: "" });  // ← WIPES it
```

**Symptom shape:** the field appears to be set successfully (in-memory React state stays populated; backend confirms write), but subsequent reads from the DB show the field empty. Often: "first attempt fails, second works" because the second attempt either hits a different code path or happens after some state landed that masks the symptom.

## Recognition cues

- Frontend payload includes a field the calling component never reads from state — usually hard-coded to `""`, `null`, `0`, or `[]`
- A separate command sets the same field
- Symptom intermittent / order-dependent / "works the second time"
- Backend upsert is the canonical INSERT...ON CONFLICT DO UPDATE shape with all-fields-overwritten

## Two fix shapes (apply BOTH for defense-in-depth)

### Backend defensive (apply once; covers all current/future callers)

For fields likely to be set out-of-band, change the UPDATE branch to preserve existing non-empty values when the input is empty:

```sql
ON CONFLICT(world_id) DO UPDATE SET
  display_name=?2,
  avatar_file=CASE WHEN ?6 = '' THEN avatar_file ELSE ?6 END,
  updated_at=datetime('now')
```

INSERT-path keeps the input value (no prior value to preserve). Choose `''` vs `NULL` per the column's null-discipline.

**Carve-out:** if some callers legitimately want to CLEAR the field (e.g., "remove my avatar"), use a sentinel like `"__CLEAR__"` or a separate `clear_user_avatar_cmd`. Don't conflate "empty means clear" with "empty means don't-touch."

### Frontend correctness (apply at the call site)

Either:
- Fetch existing value before save: `const fresh = await api.getUserProfile(...); await api.updateUserProfile({ ..., avatar_file: fresh?.avatar_file ?? "" });`
- Track the field in React state and pass it through
- Add an `overrides?: { fieldName: ... }` param to the save function so callers that JUST queued state updates can pass explicit values (closure-captured state is stale immediately after a setter)

The frontend fix is also where to address React-setter staleness: if your save function reads from state and a caller has just queued `setX(...)` updates, the closure's `x` is still the old value. Use an explicit overrides param OR refactor to pass values forward.

## Worked example (2026-05-11)

GenesisModal cross-world avatar import:
- `setUserAvatarFromGallery(world_id, file)` → `set_user_avatar_file()` in `user_profile.rs` → `UPDATE ... SET avatar_file=?2 WHERE world_id=?1`
- `setSelfAppearance(sourceProfile.description)` etc → React state queued
- `saveSelfProfile()` → `updateUserProfile({ avatar_file: "" })` → backend wipes avatar_file AND reads stale React state for name/description/facts

Fix:
- Backend: `upsert_user_profile` UPDATE-path: `avatar_file=CASE WHEN ?6 = '' THEN avatar_file ELSE ?6 END`
- Frontend: `saveSelfProfile(overrides?)` accepts explicit name/appearance/about/facts; `onImportFromWorld` threads `importedName/Appearance/Facts` as locals AND as overrides

## Composes with

- `feedback_load_bearing_multiplicity.md` — recognizing intentional design vs bug; this is a bug not intentional
- `feedback_apparatus_honest_earns_and_refuses.md` — same calibration that found the bug fixes it (both backend defensive + frontend correctness)
- `feedback_no_git_add_dash_a.md` — sibling discipline: be specific about what you stage; here: be specific about what you upsert

Soli Deo gloria.
