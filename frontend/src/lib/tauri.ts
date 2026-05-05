import { invoke } from "@tauri-apps/api/core";
import { appDataDir } from "@tauri-apps/api/path";
import { Client, Stronghold } from "@tauri-apps/plugin-stronghold";

/** Summary length mode for the on-demand summary modal. "auto" lets the
 *  model pick a length appropriate to the conversation; the named tiers
 *  give the user explicit control. */
export type SummaryMode = "short" | "medium" | "auto";

export interface World {
  world_id: string;
  name: string;
  description: string;
  tone_tags: string[];
  invariants: string[];
  state: WorldState;
  created_at: string;
  updated_at: string;
}

export interface WorldState {
  time: { day_index: number; time_of_day: string };
  global_arcs: Array<{ arc_id: string; status: string; notes: string }>;
  facts: Array<{ fact_id: string; text: string; confidence: string }>;
  /** Optional weather key (see src/lib/weather.ts). Empty / absent =
   *  no weather set; the prompt injection skips the weather block. */
  weather?: string;
}

export interface Character {
  character_id: string;
  world_id: string;
  display_name: string;
  identity: string;
  voice_rules: string[];
  boundaries: string[];
  backstory_facts: string[];
  relationships: Record<string, unknown>;
  state: CharacterState;
  avatar_color: string;
  sex: string;
  is_archived: boolean;
  created_at: string;
  updated_at: string;
  /** Honest physical description generated from the active portrait by
   *  a vision call. Empty until generated. Shared into group-chat and
   *  narrative prompts so other characters can picture this person. */
  visual_description?: string;
  /** The portrait_id that produced the current visual_description.
   *  Cache key — if it matches the currently-active portrait, no
   *  re-generation is needed. */
  visual_description_portrait_id?: string | null;
  /** Up to 3 "still in their keeping" items. Refreshed on world-day
   *  rollover by a memory-tier LLM call; user-editable in settings. */
  inventory?: InventoryItem[];
  /** World-day index the inventory was last refreshed against. NULL =
   *  never seeded. */
  last_inventory_day?: number | null;
  /** Optional single-emoji signature. Empty string = no signature.
   *  Rendered into the prompt with usage guidance (use rarely, only on
   *  beats where the character feels especially themselves). */
  signature_emoji?: string;
  /** How often this character uses italicized stage directions
   *  (*leans back*, *looks out the window*). Overrides the global
   *  ~1-in-3-replies baseline per-character. Defaults to "normal". */
  action_beat_density?: "low" | "normal" | "high";
  /** When true, the full Empiricon document is injected into this character's LLM prompts. */
  has_read_empiricon?: boolean;
}

export interface InventoryItem {
  name: string;
  description: string;
  /** "physical" (thing they carry) or "interior" (the one non-physical
   *  thing they're carrying inside — memory, core truth, profound
   *  feeling of the day). Defaults to "physical" if omitted. */
  kind?: "physical" | "interior";
}

export interface InventoryRefreshResult {
  character_id: string;
  inventory: InventoryItem[];
  refreshed: boolean;
  /** "seed" | "refresh" | "noop" | "moment" */
  mode: string;
  /** Items newly added by a moment-anchored update (full item with description). */
  added?: InventoryItem[];
  /** Items whose description changed in a moment-anchored update. */
  updated?: InventoryItem[];
  /** Names of items that were removed. */
  removed?: string[];
}

export interface UpdateInventoryForMomentResponse {
  results: InventoryRefreshResult[];
  /** The "[Inventory updated:] ..." message row inserted into the chat
   *  transcript. Null when every target was pure-maintain (no changes
   *  were made — only possible on narrative-in-group fan-out). */
  new_message: Message | null;
}

export interface InventoryUpdateRecord {
  message_id: string;
  character_id: string;
  character_name: string;
  added: string[];
  updated: string[];
  removed: string[];
  created_at: string;
}

export interface CharacterState {
  mood: number;
  trust_user: number;
  goals: string[];
  open_loops: string[];
  last_seen: { day_index: number; time_of_day: string };
}

/// Compact summary of an illustration message — returned by
/// listThreadIllustrations for the sticky-thumbnail feature so the UI
/// can reference illustrations even when they're not paginated into
/// store.messages.
export interface IllustrationSummary {
  message_id: string;
  content: string;
  created_at: string;
  world_day: number | null;
  world_time: string | null;
  thread_id: string;
}

export interface Message {
  message_id: string;
  thread_id: string;
  role: "user" | "assistant" | "system" | "narrative" | "illustration" | "context" | "dream" | "inventory_update" | "imagined_chapter" | "settings_update" | "location_change";
  content: string;
  tokens_estimate: number;
  sender_character_id: string | null;
  created_at: string;
  world_day: number | null;
  world_time: string | null;
  /** True when this assistant message was emitted as a proactive ping
   *  (character reaching out first). Drives distinct styling + unread badge. */
  is_proactive?: boolean;
  /** Unicode-math chat-state signature derived from 𝓕 := (𝓡, 𝓒). Populated
   *  on assistant messages generated under reactions=off. Used by the chiptune
   *  soundtrack as the per-message momentstamp keying next-phrase generation. */
  formula_signature?: string | null;
}

export interface ProactivePingResult {
  message: Message | null;
  skipped_reason: string | null;
}

export interface GroupChat {
  group_chat_id: string;
  world_id: string;
  character_ids: string[];
  thread_id: string;
  display_name: string;
  created_at: string;
}

export interface SendGroupMessageResult {
  user_message: Message;
  character_responses: Message[];
}

export interface PromptGroupCharacterResult {
  assistant_message: Message;
  ai_reactions: Reaction[];
}

export interface PaginatedMessages {
  messages: Message[];
  total: number;
}


export interface ModelConfig {
  dialogue_model: string;
  tick_model: string;
  memory_model: string;
  embedding_model: string;
  image_model: string;
  vision_model: string;
  ai_provider: string;
  lmstudio_url: string;
  /** Declared local-model context window in tokens. Used for chunking long
   *  novelization prompts. UI shows this in 10k steps (e.g. "40k"). */
  lmstudio_context_tokens: number;
  /** Frontier (OpenAI) dialogue model used when a chat opts into the
   *  per-chat "Frontier" provider override. Stored separately from
   *  dialogue_model so the override works even when the primary dialogue
   *  model is configured for a local backend. */
  dialogue_model_frontier: string;
}

export interface LocalModelInfo {
  id: string;
  owned_by: string;
}

export interface SendMessageResult {
  user_message: Message;
  assistant_message: Message;
  ai_reactions: Reaction[];
}

export interface PromptCharacterResult {
  assistant_message: Message;
  ai_reactions: Reaction[];
}

export interface NarrativeResult {
  narrative_message: Message;
}

export interface DreamResult {
  dream_message: Message;
}

export interface IllustrationResult {
  illustration_message: Message;
}

export interface ConsultantChat {
  chat_id: string;
  thread_id: string;
  title: string;
  created_at: string;
  last_seen_message_id: string | null;
  /** "immersive" — in-the-story confidant (default). "backstage" —
   *  fourth-wall stage manager that reads the save file. */
  mode: "immersive" | "backstage";
}

export interface NovelEntry {
  novel_id: string;
  thread_id: string;
  world_day: number;
  content: string;
  created_at: string;
  updated_at: string;
}


export interface ResetToMessageResult {
  deleted_count: number;
  new_response: SendMessageResult | null;
}

export interface Reaction {
  reaction_id: string;
  message_id: string;
  emoji: string;
  reactor: "user" | "assistant";
  created_at: string;
  /** Which character authored this reaction. Null for user reactions and
   *  for legacy character reactions pre-dating the attribution column. */
  sender_character_id?: string | null;
}

export interface KeptRecord {
  kept_id: string;
  source_message_id: string | null;
  source_thread_id: string | null;
  source_world_day: number | null;
  source_created_at: string | null;
  // Live writes come from the auto-canonization classifier: character|user
  // subjects and one of the five live record types below. Older rows may
  // carry deprecated subject_type ("world"/"relationship") or record_type
  // ("relationship_note"/"world_fact") — still readable, kept in the union.
  subject_type: "character" | "user" | "world" | "relationship";
  subject_id: string;
  record_type:
    | "description_weave"
    | "voice_rule"
    | "boundary"
    | "known_fact"
    | "open_loop"
    | "relationship_note"
    | "world_fact";
  content: string;
  user_note: string;
  created_at: string;
}

/// Auto-canonization proposal kinds. All five map to an editable shape
/// on the CharacterEditor page (description = textarea; voice_rule /
/// boundary / known_fact / open_loop = single-line bullet strings).
export type CanonKind =
  | "description_weave"
  | "voice_rule"
  | "boundary"
  | "known_fact"
  | "open_loop";

/// What the classifier wants to do to the canonical record.
/// - add: append a new item (list kinds) — or rewrite identity (weave).
/// - update: replace an existing item with a nuanced version. Weave is
///   effectively always "update."
/// - remove: delete an existing item. Rare, destructive.
export type CanonAction = "add" | "update" | "remove";

export interface ProposedCanonUpdate {
  kind: CanonKind;
  action: CanonAction;
  subject_type: "character" | "user";
  subject_id: string;
  subject_label: string;
  /// For description_weave: full revised description to replace
  /// identity/description. For list kinds + add: the new bullet. For
  /// list kinds + update: the replacement text. For list kinds +
  /// remove: empty string (unused — content is the target being removed).
  new_content: string;
  /// Present when action ∈ {update, remove} on a list kind — the exact
  /// existing item being targeted. Commit-side matches this against the
  /// character's current state.
  target_existing_text: string | null;
  /// For description_weave: the old description. For list + update/remove:
  /// the targeted existing text (mirrored here for convenient UI render).
  /// For list + add: null.
  prior_content: string | null;
  justification: string;
}

export interface AppliedCanonUpdate extends ProposedCanonUpdate {
  /// Present for add/update commits (a kept_records row was written).
  /// Null for remove commits — the ledger records assertions kept, not
  /// assertions deleted.
  kept_id: string | null;
}

export interface UserProfile {
  world_id: string;
  display_name: string;
  description: string;
  facts: string[];
  boundaries: string[];
  avatar_file: string;
  updated_at: string;
  derived_formula?: string | null;
  derived_summary?: string | null;
}

export interface UserDerivationResult {
  derivation: string;
  summary: string;
}

export interface PortraitInfo {
  portrait_id: string;
  character_id: string;
  prompt: string;
  is_active: boolean;
  created_at: string;
  data_url: string;
}

export interface WorldImageInfo {
  image_id: string;
  world_id: string;
  prompt: string;
  source: string;
  is_active: boolean;
  created_at: string;
  data_url: string;
}

export interface GalleryItem {
  id: string;
  source_id: string;
  file_name: string;
  data_url: string;
  prompt: string;
  category: "world" | "character" | "user";
  label: string;
  is_archived: boolean;
  tags: string[];
  created_at: string;
}

export interface ChatBackground {
  character_id: string;
  bg_type: "color" | "world_image";
  bg_color: string;
  bg_image_id: string;
  bg_blur: number;
  updated_at: string;
}

export interface CharacterMood {
  character_id: string;
  valence: number;
  energy: number;
  tension: number;
  history: Array<{ v: number; e: number; t: number }>;
  updated_at: string;
}

export interface MoodSettings {
  enabled: boolean;
  drift_rate: number;
}

export interface DailyUsage {
  date: string;
  prompt_tokens: number;
  completion_tokens: number;
}

export interface MemoryArtifact {
  artifact_id: string;
  artifact_type: string;
  subject_id: string;
  world_id: string;
  content: string;
  sources: unknown[];
  created_at: string;
  updated_at: string;
}

const VAULT_PASSWORD = "world-threads-vault";
const CLIENT_NAME = "world-threads";
const API_KEY_RECORD = "openai_api_key";
const GOOGLE_API_KEY_RECORD = "google_ai_api_key";

let _stronghold: Stronghold | null = null;
let _client: Client | null = null;
let _initPromise: Promise<Client> | null = null;

function getVaultClient(): Promise<Client> {
  if (_client) return Promise.resolve(_client);
  if (_initPromise) return _initPromise;
  _initPromise = (async () => {
    try {
      const vaultPath = `${await appDataDir()}/vault.hold`;
      _stronghold = await Stronghold.load(vaultPath, VAULT_PASSWORD);
      try {
        _client = await _stronghold.loadClient(CLIENT_NAME);
      } catch {
        _client = await _stronghold.createClient(CLIENT_NAME);
      }
      return _client!;
    } catch (e) {
      _initPromise = null;
      throw e;
    }
  })();
  return _initPromise;
}

async function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T | null> {
  let timer: ReturnType<typeof setTimeout>;
  return Promise.race([
    promise,
    new Promise<null>((resolve) => { timer = setTimeout(() => resolve(null), ms); }),
  ]).finally(() => clearTimeout(timer!));
}

async function getApiKeyFromVault(): Promise<string> {
  try {
    const client = await getVaultClient();
    const store = client.getStore();
    const data = await withTimeout(store.get(API_KEY_RECORD), 3000);
    if (!data || data.length === 0) return "";
    return new TextDecoder().decode(new Uint8Array(data));
  } catch {
    return "";
  }
}

async function setApiKeyInVault(key: string): Promise<void> {
  const client = await getVaultClient();
  const store = client.getStore();
  const data = Array.from(new TextEncoder().encode(key));
  await store.insert(API_KEY_RECORD, data);
  await _stronghold!.save();
}

async function getGoogleApiKeyFromVault(): Promise<string> {
  try {
    const client = await getVaultClient();
    const store = client.getStore();
    const data = await withTimeout(store.get(GOOGLE_API_KEY_RECORD), 3000);
    if (!data || data.length === 0) return "";
    return new TextDecoder().decode(new Uint8Array(data));
  } catch {
    return "";
  }
}

async function setGoogleApiKeyInVault(key: string): Promise<void> {
  const client = await getVaultClient();
  const store = client.getStore();
  const data = Array.from(new TextEncoder().encode(key));
  await store.insert(GOOGLE_API_KEY_RECORD, data);
  await _stronghold!.save();
}

async function migrateApiKeyToVault(): Promise<string> {
  try {
    const legacyKey = await invoke<string | null>("get_setting_cmd", { key: "api_key" });
    if (legacyKey) {
      await setApiKeyInVault(legacyKey);
      await invoke<void>("set_setting_cmd", { key: "api_key", value: "" });
      return legacyKey;
    }
  } catch {
    // legacy table may not have the key
  }
  return await getApiKeyFromVault();
}

export const api = {
  createWorld: (name: string) => invoke<World>("create_world_cmd", { name }),
  getWorld: (worldId: string) => invoke<World>("get_world_cmd", { worldId }),
  getWorldDerivation: (worldId: string) => invoke<string | null>("get_world_derivation_cmd", { worldId }),
  listWorlds: () => invoke<World[]>("list_worlds_cmd"),
  updateWorld: (world: World) => invoke<void>("update_world_cmd", { world }),
  deleteWorld: (worldId: string) => invoke<void>("delete_world_cmd", { worldId }),
  updateWorldState: (worldId: string, state: WorldState) => invoke<void>("update_world_state_cmd", { worldId, state }),

  getTodayUsage: () => invoke<DailyUsage>("get_today_usage_cmd"),

  generateNextScorePhrase: (apiKey: string, args: {
    currentLastPhrase: unknown | null;
    momentstamp: string;
    moodHint?: string | null;
  }) => invoke<{ phrase: unknown; raw: string }>("generate_next_score_phrase_cmd", {
    apiKey,
    currentLastPhrase: args.currentLastPhrase ?? null,
    momentstamp: args.momentstamp,
    moodHint: args.moodHint ?? null,
  }),

  getUserProfile: (worldId: string) => invoke<UserProfile | null>("get_user_profile_cmd", { worldId }),
  updateUserProfile: (profile: UserProfile) => invoke<void>("update_user_profile_cmd", { profile }),
  regenerateCharacterDerivation: (apiKey: string, characterId: string) =>
    invoke<{ derivation: string; summary: string }>("regenerate_character_derivation_cmd", { apiKey, characterId }),
  regenerateUserDerivation: (apiKey: string, worldId: string, choices: {
    way_of_being?: string;
    place?: string;
    hands?: string;
    carrying?: string;
    seen_as?: string;
  }) => invoke<UserDerivationResult>("regenerate_user_derivation_cmd", {
    apiKey, worldId,
    wayOfBeing: choices.way_of_being ?? null,
    place: choices.place ?? null,
    hands: choices.hands ?? null,
    carrying: choices.carrying ?? null,
    seenAs: choices.seen_as ?? null,
  }),
  generateUserAvatar: (apiKey: string, worldId: string, formHint?: { display_name?: string; description?: string }) =>
    invoke<string>("generate_user_avatar_cmd", { apiKey, worldId, formHint: formHint ?? null }),
  uploadUserAvatar: (worldId: string, imageData: string) =>
    invoke<string>("upload_user_avatar_cmd", { worldId, imageData }),
  getUserAvatar: (worldId: string) =>
    invoke<string>("get_user_avatar_cmd", { worldId }),
  listAllUserAvatars: () =>
    invoke<Array<{ world_id: string; world_name: string; avatar_file: string; data_url: string }>>("list_all_user_avatars_cmd"),
  setUserAvatarFromGallery: (worldId: string, sourceFile: string) =>
    invoke<string>("set_user_avatar_from_gallery_cmd", { worldId, sourceFile }),

  listCharacters: (worldId: string) => invoke<Character[]>("list_characters_cmd", { worldId }),
  getCharacter: (characterId: string) => invoke<Character>("get_character_cmd", { characterId }),
  getCharacterDerivation: (characterId: string) => invoke<string | null>("get_character_derivation_cmd", { characterId }),
  updateCharacter: (character: Character) => invoke<void>("update_character_cmd", { character }),
  createCharacter: (worldId: string, displayName: string) => invoke<Character>("create_character_cmd", { worldId, displayName }),
  deleteCharacter: (characterId: string) => invoke<void>("delete_character_cmd", { characterId }),
  clearChatHistory: (characterId: string, keepMedia: boolean) => invoke<void>("clear_chat_history_cmd", { characterId, keepMedia }),
  archiveCharacter: (characterId: string) => invoke<void>("archive_character_cmd", { characterId }),
  unarchiveCharacter: (characterId: string) => invoke<void>("unarchive_character_cmd", { characterId }),
  listArchivedCharacters: (worldId: string) => invoke<Character[]>("list_archived_characters_cmd", { worldId }),

  saveUserMessage: (characterId: string, content: string) =>
    invoke<Message>("save_user_message_cmd", { characterId, content }),
  sendMessage: (apiKey: string, characterId: string, content: string) =>
    invoke<SendMessageResult>("send_message_cmd", { apiKey, characterId, content }),
  promptCharacter: (apiKey: string, characterId: string) =>
    invoke<PromptCharacterResult>("prompt_character_cmd", { apiKey, characterId }),
  tryProactivePing: (apiKey: string, characterId: string) =>
    invoke<ProactivePingResult>("try_proactive_ping_cmd", { apiKey, characterId }),
  getProactiveUnreadCounts: () =>
    invoke<Record<string, number>>("get_proactive_unread_counts_cmd"),
  adjustMessage: (apiKey: string, messageId: string, instructions: string, isGroup: boolean) =>
    invoke<Message>("adjust_message_cmd", { apiKey, messageId, instructions, isGroup }),
  editMessageContent: (messageId: string, content: string, isGroup: boolean) =>
    invoke<void>("edit_message_content_cmd", { messageId, content, isGroup }),
  deleteMessage: (messageId: string, isGroup: boolean) =>
    invoke<void>("delete_message_cmd", { messageId, isGroup }),

  // Novel entries
  generateNovelEntry: (apiKey: string, threadId: string, worldDay: number, isGroup: boolean) =>
    invoke<string>("generate_novel_entry_cmd", { apiKey, threadId, worldDay, isGroup }),
  saveNovelEntry: (threadId: string, worldDay: number, content: string) =>
    invoke<NovelEntry>("save_novel_entry_cmd", { threadId, worldDay, content }),
  getNovelEntry: (threadId: string, worldDay: number) =>
    invoke<NovelEntry | null>("get_novel_entry_cmd", { threadId, worldDay }),
  listNovelEntries: (threadId: string) =>
    invoke<NovelEntry[]>("list_novel_entries_cmd", { threadId }),
  deleteNovelEntry: (threadId: string, worldDay: number) =>
    invoke<void>("delete_novel_entry_cmd", { threadId, worldDay }),

  // Story consultant
  createConsultantChat: (threadId: string, title?: string, mode?: "immersive" | "backstage") =>
    invoke<ConsultantChat>("create_consultant_chat_cmd", { threadId, title: title ?? null, mode: mode ?? null }),
  listConsultantChats: (threadId: string) =>
    invoke<ConsultantChat[]>("list_consultant_chats_cmd", { threadId }),
  updateConsultantChatTitle: (chatId: string, title: string) =>
    invoke<void>("update_consultant_chat_title_cmd", { chatId, title }),
  deleteConsultantChat: (chatId: string) =>
    invoke<void>("delete_consultant_chat_cmd", { chatId }),
  generateConsultantTitle: (apiKey: string, userMessage: string) =>
    invoke<string>("generate_consultant_title_cmd", { apiKey, userMessage }),
  storyConsultant: (apiKey: string, chatId: string, characterId: string | null, groupChatId: string | null, userMessage: string) =>
    invoke<string>("story_consultant_cmd", { apiKey, chatId, characterId, groupChatId, userMessage }),
  loadConsultantChat: (chatId: string) =>
    invoke<Array<{ role: string; content: string }>>("load_consultant_chat_cmd", { chatId }),
  clearConsultantChat: (chatId: string) =>
    invoke<void>("clear_consultant_chat_cmd", { chatId }),
  truncateConsultantChat: (chatId: string, keepCount: number) =>
    invoke<void>("truncate_consultant_chat_cmd", { chatId, keepCount }),
  saveConsultantMessages: (chatId: string, messages: Array<{ role: string; content: string }>) =>
    invoke<void>("save_consultant_messages_cmd", { chatId, messages }),
  importChatMessages: (chatId: string, characterId: string | null, groupChatId: string | null) =>
    invoke<{ role: string; content: string }>("import_chat_messages_cmd", { chatId, characterId, groupChatId }),
  getLastSeenMessage: (chatId: string) =>
    invoke<{ message_id: string; role: string; content: string; speaker_name: string; character_id: string | null; avatar_color: string | null; created_at: string } | null>("get_last_seen_message_cmd", { chatId }),
  generateNarrative: (apiKey: string, characterId: string, customInstructions?: string) =>
    invoke<NarrativeResult>("generate_narrative_cmd", { apiKey, characterId, customInstructions: customInstructions ?? null }),
  generateDream: (apiKey: string, characterId: string) =>
    invoke<DreamResult>("generate_dream_cmd", { apiKey, characterId }),
  generateIllustration: (apiKey: string, characterId: string, qualityTier?: string, customInstructions?: string, previousIllustrationId?: string, includeSceneSummary?: boolean) =>
    invoke<IllustrationResult>("generate_illustration_cmd", { apiKey, characterId, qualityTier: qualityTier ?? null, customInstructions: customInstructions ?? null, previousIllustrationId: previousIllustrationId ?? null, includeSceneSummary: includeSceneSummary ?? true }),
  /// Backstage two-step illustration flow — preview returns the rendered
  /// image without inserting a chat message; attach commits it to the
  /// active chat (solo or group); discard cleans up the preview.
  previewBackstageIllustration: (apiKey: string, characterId: string, groupChatId: string | null, customInstructions?: string) =>
    invoke<{ image_id: string; data_url: string; aspect_ratio: number; caption: string }>("preview_backstage_illustration_cmd", { apiKey, characterId, groupChatId, customInstructions: customInstructions ?? null }),
  attachPreviewedIllustration: (imageId: string, targetThreadId: string, isGroupThread: boolean) =>
    invoke<Message>("attach_previewed_illustration_cmd", { imageId, targetThreadId, isGroupThread }),
  discardPreviewedIllustration: (imageId: string) =>
    invoke<void>("discard_previewed_illustration_cmd", { imageId }),
  deleteIllustration: (messageId: string) =>
    invoke<void>("delete_illustration_cmd", { messageId }),
  updateIllustrationCaption: (messageId: string, caption: string) =>
    invoke<void>("update_illustration_caption_cmd", { messageId, caption }),
  getIllustrationCaptions: (messageIds: string[]) =>
    invoke<Record<string, string>>("get_illustration_captions_cmd", { messageIds }),
  regenerateIllustration: (apiKey: string, characterId: string, messageId: string) =>
    invoke<IllustrationResult>("regenerate_illustration_cmd", { apiKey, characterId, messageId }),
  adjustIllustration: (apiKey: string, characterId: string, messageId: string, instructions: string) =>
    invoke<IllustrationResult>("adjust_illustration_cmd", { apiKey, characterId, messageId, instructions }),
  generateVideo: (apiKey: string, googleApiKey: string, characterId: string, illustrationMessageId: string, customPrompt?: string, durationSeconds?: number, style?: string, includeContext?: boolean) =>
    invoke<string>("generate_video_cmd", { apiKey, googleApiKey, characterId, illustrationMessageId, customPrompt: customPrompt ?? null, durationSeconds: durationSeconds ?? null, style: style ?? null, includeContext: includeContext ?? null }),
  getIllustrationAspectRatio: (illustrationMessageId: string) =>
    invoke<number>("get_illustration_aspect_ratio_cmd", { illustrationMessageId }),
  getVideoFile: (illustrationMessageId: string) =>
    invoke<string | null>("get_video_file_cmd", { illustrationMessageId }),
  removeVideo: (illustrationMessageId: string) =>
    invoke<void>("remove_video_cmd", { illustrationMessageId }),
  uploadVideo: (illustrationMessageId: string, videoData: string) =>
    invoke<string>("upload_video_cmd", { illustrationMessageId, videoData }),
  downloadIllustration: (illustrationMessageId: string) =>
    invoke<string>("download_illustration_cmd", { illustrationMessageId }),
  getVideoBytes: (videoFile: string) =>
    invoke<number[]>("get_video_bytes_cmd", { videoFile }),
  getMediaDir: () => invoke<string>("get_media_dir_cmd"),
  resetToMessage: (apiKey: string, characterId: string, messageId: string) =>
    invoke<ResetToMessageResult>("reset_to_message_cmd", { apiKey, characterId, messageId }),
  getIllustrationData: (messageId: string) =>
    invoke<string | null>("get_illustration_data_cmd", { messageId }),
  /// List every illustration message in a thread (across both solo
  /// `messages` and group `group_messages` tables), ordered ASC by
  /// created_at. Independent of pagination — returns the full
  /// timeline of illustrations so the UI can reference ones that
  /// aren't currently loaded in `store.messages`.
  listThreadIllustrations: (threadId: string) =>
    invoke<IllustrationSummary[]>("list_thread_illustrations_cmd", { threadId }),
  getLastMessageTime: (worldId: string) =>
    invoke<string | null>("get_last_message_time_cmd", { worldId }),
  getMessages: (characterId: string, limit?: number, offset?: number) =>
    invoke<PaginatedMessages>("get_messages_cmd", { characterId, limit, offset }),


  // TTS
  generateSpeech: (apiKey: string, messageId: string, text: string, characterId: string, tone?: string) =>
    invoke<number[]>("generate_speech_cmd", { apiKey, messageId, text, characterId, tone: tone ?? null }),
  generateVoiceSample: (apiKey: string, voice: string, tone?: string, model?: string) =>
    invoke<number[]>("generate_voice_sample_cmd", { apiKey, voice, tone: tone ?? null, model: model ?? null }),
  getSpeech: (messageId: string) =>
    invoke<number[] | null>("get_speech_cmd", { messageId }),
  listCachedAudio: () =>
    invoke<{ cached: Record<string, string[]>; last_tones: Record<string, string> }>("list_cached_audio_cmd"),
  deleteMessageAudio: (messageId: string) =>
    invoke<void>("delete_message_audio_cmd", { messageId }),
  clearVoiceSamples: () =>
    invoke<void>("clear_voice_samples_cmd"),

  getModelConfig: () => invoke<ModelConfig>("get_model_config_cmd"),
  setModelConfig: (config: ModelConfig) => invoke<void>("set_model_config_cmd", { config }),

  // Background novelization (local-model idle work).
  runBackgroundNovelization: (apiKey: string) =>
    invoke<void>("run_background_novelization_cmd", { apiKey }),
  cancelBackgroundNovelization: () =>
    invoke<void>("cancel_background_novelization_cmd"),
  getSetting: (key: string) => invoke<string | null>("get_setting_cmd", { key }),
  setSetting: (key: string, value: string) => invoke<void>("set_setting_cmd", { key, value }),
  isChildrenModePasswordSet: () =>
    invoke<boolean>("is_children_mode_password_set_cmd"),
  enableChildrenModeWithPassword: (password: string) =>
    invoke<void>("enable_children_mode_with_password_cmd", { password }),
  disableChildrenModeWithPassword: (password: string) =>
    invoke<void>("disable_children_mode_with_password_cmd", { password }),
  /// Insert a settings_update message row marking that the user just
  /// changed one or more chat settings. Surfaces in chat history both
  /// for the user (a small earthy-codeblock card) and for the LLM
  /// (so it knows previous replies were under different settings and
  /// shouldn't pattern-match against them).
  recordChatSettingsChange: (threadId: string, changes: Array<{ key: string; label: string; from: string; to: string }>, isGroup: boolean) =>
    invoke<Message>("record_chat_settings_change_cmd", { threadId, changes, isGroup }),
  getApiKey: () => getApiKeyFromVault(),
  setApiKey: (key: string) => setApiKeyInVault(key),
  migrateApiKey: () => migrateApiKeyToVault(),
  getGoogleApiKey: () => getGoogleApiKeyFromVault(),
  setGoogleApiKey: (key: string) => setGoogleApiKeyInVault(key),
  getBudgetMode: () => invoke<boolean>("get_budget_mode_cmd"),
  setBudgetMode: (enabled: boolean) => invoke<void>("set_budget_mode_cmd", { enabled }),
  listLocalModels: (url: string) => invoke<LocalModelInfo[]>("list_local_models_cmd", { url }),

  getMemoryArtifacts: (subjectId: string, artifactType: string) =>
    invoke<MemoryArtifact[]>("get_memory_artifacts_cmd", { subjectId, artifactType }),
  getThreadSummary: (characterId: string) =>
    invoke<string>("get_thread_summary_cmd", { characterId }),
  generateChatSummary: (apiKey: string, characterId: string, mode?: SummaryMode) =>
    invoke<string>("generate_chat_summary_cmd", { apiKey, characterId, mode: mode ?? null }),
  generateGroupChatSummary: (apiKey: string, groupChatId: string, mode?: SummaryMode) =>
    invoke<string>("generate_group_chat_summary_cmd", { apiKey, groupChatId, mode: mode ?? null }),

  generatePortrait: (apiKey: string, characterId: string, formHint?: { display_name?: string; identity?: string; backstory_facts?: unknown }) =>
    invoke<PortraitInfo>("generate_portrait_cmd", { apiKey, characterId, formHint: formHint ?? null }),
  generatePortraitVariation: (apiKey: string, characterId: string) =>
    invoke<PortraitInfo>("generate_portrait_variation_cmd", { apiKey, characterId }),
  generatePortraitWithPose: (apiKey: string, characterId: string, poseDescription: string) =>
    invoke<PortraitInfo>("generate_portrait_with_pose_cmd", { apiKey, characterId, poseDescription }),
  listPortraits: (characterId: string) =>
    invoke<PortraitInfo[]>("list_portraits_cmd", { characterId }),
  deletePortrait: (portraitId: string) =>
    invoke<void>("delete_portrait_cmd", { portraitId }),
  setActivePortrait: (characterId: string, portraitId: string) =>
    invoke<void>("set_active_portrait_cmd", { characterId, portraitId }),
  setPortraitFromGallery: (characterId: string, sourceFile: string) =>
    invoke<PortraitInfo>("set_portrait_from_gallery_cmd", { characterId, sourceFile }),
  getActivePortrait: (characterId: string) =>
    invoke<PortraitInfo | null>("get_active_portrait_cmd", { characterId }),
  backfillEmbeddings: (apiKey: string) =>
    invoke<{ embedded: number; skipped: number; errors: number }>("backfill_embeddings_cmd", { apiKey }),
  generateCharacterVisualDescription: (apiKey: string, characterId: string, force?: boolean) =>
    invoke<Character>("generate_character_visual_description_cmd", { apiKey, characterId, force: force ?? null }),
  listCharactersNeedingVisualDescription: (worldId: string) =>
    invoke<string[]>("list_characters_needing_visual_description_cmd", { worldId }),

  generateWorldImage: (apiKey: string, worldId: string, formHint?: { name?: string; description?: string; tone_tags?: unknown }) =>
    invoke<WorldImageInfo>("generate_world_image_cmd", { apiKey, worldId, formHint: formHint ?? null }),
  generateWorldImageWithPrompt: (apiKey: string, worldId: string, customPrompt: string) =>
    invoke<WorldImageInfo>("generate_world_image_with_prompt_cmd", { apiKey, worldId, customPrompt }),
  uploadWorldImage: (worldId: string, imageData: string, label: string) =>
    invoke<WorldImageInfo>("upload_world_image_cmd", { worldId, imageData, label }),
  listWorldImages: (worldId: string) =>
    invoke<WorldImageInfo[]>("list_world_images_cmd", { worldId }),
  listWorldGallery: (worldId: string) =>
    invoke<GalleryItem[]>("list_world_gallery_cmd", { worldId }),
  listWorldGalleryMeta: (worldId: string) =>
    invoke<GalleryItem[]>("list_world_gallery_meta_cmd", { worldId }),
  getGalleryImage: (fileName: string) =>
    invoke<string>("get_gallery_image_cmd", { fileName }),
  archiveGalleryItem: (itemId: string, category: string) =>
    invoke<void>("archive_gallery_item_cmd", { itemId, category }),
  unarchiveGalleryItem: (itemId: string, category: string) =>
    invoke<void>("unarchive_gallery_item_cmd", { itemId, category }),
  deleteGalleryItem: (itemId: string, category: string, fileName: string) =>
    invoke<void>("delete_gallery_item_cmd", { itemId, category, fileName }),
  saveCrop: (worldId: string, sourceCategory: string, sourceId: string, imageData: string) =>
    invoke<GalleryItem>("save_crop_cmd", { worldId, sourceCategory, sourceId, imageData }),
  getActiveWorldImage: (worldId: string) =>
    invoke<WorldImageInfo | null>("get_active_world_image_cmd", { worldId }),
  setActiveWorldImage: (worldId: string, imageId: string) =>
    invoke<void>("set_active_world_image_cmd", { worldId, imageId }),

  getChatBackground: (characterId: string) =>
    invoke<ChatBackground | null>("get_chat_background_cmd", { characterId }),
  updateChatBackground: (bg: ChatBackground) =>
    invoke<void>("update_chat_background_cmd", { bg }),

  getCharacterMood: (characterId: string) =>
    invoke<CharacterMood | null>("get_character_mood_cmd", { characterId }),
  getMoodSettings: () => invoke<MoodSettings>("get_mood_settings_cmd"),
  setMoodSettings: (settings: MoodSettings) => invoke<void>("set_mood_settings_cmd", { settings }),

  addReaction: (messageId: string, emoji: string, reactor: string) =>
    invoke<Reaction>("add_reaction_cmd", { messageId, emoji, reactor }),
  removeReaction: (messageId: string, emoji: string, reactor: string) =>
    invoke<void>("remove_reaction_cmd", { messageId, emoji, reactor }),
  getReactions: (messageIds: string[]) =>
    invoke<Reaction[]>("get_reactions_cmd", { messageIds }),
  getMoodReduction: (opts: { characterId?: string; groupChatId?: string }) =>
    invoke<string[]>("get_mood_reduction_cmd", { characterId: opts.characterId ?? null, groupChatId: opts.groupChatId ?? null }),

  // Canon (Promote to Canon flow)
  proposeKeptWeave: (apiKey: string, request: { sourceMessageId: string; subjectType: string; subjectId: string }) =>
    invoke<{ current_description: string; proposed_description: string }>("propose_kept_weave_cmd", { apiKey, request }),
  saveKeptRecord: (apiKey: string, request: {
    sourceMessageId?: string | null;
    subjectType: string;
    subjectId: string;
    recordType: string;
    content: string;
    userNote?: string;
  }) => invoke<KeptRecord>("save_kept_record_cmd", { apiKey, request }),
  /// Auto-canon propose: classify a moment into 1-2 proposed updates
  /// (description_weave / voice_rule / boundary / known_fact / open_loop)
  /// without applying anything. The returned proposals carry a kind,
  /// subject, content, optional prior content for diff, and a brief
  /// justification — the UI can let the user edit content and then
  /// commit.
  proposeAutoCanon: (apiKey: string, request: { sourceMessageId: string; act: "light" | "heavy"; userHint?: string }) =>
    invoke<ProposedCanonUpdate[]>("propose_auto_canon_cmd", { apiKey, request }),
  commitAutoCanon: (apiKey: string, request: { sourceMessageId: string; updates: ProposedCanonUpdate[]; userNote?: string }) =>
    invoke<AppliedCanonUpdate[]>("commit_auto_canon_cmd", { apiKey, request }),
  listKeptMessageIds: (threadId: string) =>
    invoke<string[]>("list_kept_message_ids_cmd", { threadId }),
  listKeptForMessage: (messageId: string) =>
    invoke<KeptRecord[]>("list_kept_for_message_cmd", { messageId }),
  deleteKeptRecord: (canonId: string) =>
    invoke<void>("delete_kept_record_cmd", { canonId }),

  // Backup
  getLatestBackup: () =>
    invoke<{ file_name: string; timestamp: string } | null>("get_latest_backup_cmd"),
  listBackups: () =>
    invoke<Array<{ file_name: string; timestamp: string }>>("list_backups_cmd"),
  backupNow: () =>
    invoke<{ file_name: string; timestamp: string }>("backup_now_cmd"),
  restoreBackup: (backupFileName: string) =>
    invoke<void>("restore_backup_cmd", { backupFileName }),

  // Group chats
  createGroupChat: (worldId: string, characterIds: string[]) =>
    invoke<GroupChat>("create_group_chat_cmd", { worldId, characterIds }),
  listGroupChats: (worldId: string) =>
    invoke<GroupChat[]>("list_group_chats_cmd", { worldId }),
  deleteGroupChat: (groupChatId: string) =>
    invoke<void>("delete_group_chat_cmd", { groupChatId }),
  clearGroupChatHistory: (groupChatId: string, keepMedia: boolean) =>
    invoke<void>("clear_group_chat_history_cmd", { groupChatId, keepMedia }),
  getGroupMessages: (groupChatId: string, limit?: number, offset?: number) =>
    invoke<PaginatedMessages>("get_group_messages_cmd", { groupChatId, limit, offset }),
  saveGroupUserMessage: (groupChatId: string, content: string) =>
    invoke<Message>("save_group_user_message_cmd", { groupChatId, content }),
  sendGroupMessage: (apiKey: string, groupChatId: string, content: string) =>
    invoke<SendGroupMessageResult>("send_group_message_cmd", { apiKey, groupChatId, content }),
  pickGroupResponders: (apiKey: string, groupChatId: string, content: string) =>
    invoke<string[]>("pick_group_responders_cmd", { apiKey, groupChatId, content }),
  promptGroupCharacter: (apiKey: string, groupChatId: string, characterId: string, addressTo?: string) =>
    invoke<PromptGroupCharacterResult>("prompt_group_character_cmd", { apiKey, groupChatId, characterId, addressTo: addressTo ?? null }),
  generateGroupNarrative: (apiKey: string, groupChatId: string, customInstructions?: string) =>
    invoke<NarrativeResult>("generate_group_narrative_cmd", { apiKey, groupChatId, customInstructions: customInstructions ?? null }),
  generateGroupIllustration: (apiKey: string, groupChatId: string, qualityTier?: string, customInstructions?: string, previousIllustrationId?: string, includeSceneSummary?: boolean) =>
    invoke<IllustrationResult>("generate_group_illustration_cmd", { apiKey, groupChatId, qualityTier: qualityTier ?? null, customInstructions: customInstructions ?? null, previousIllustrationId: previousIllustrationId ?? null, includeSceneSummary: includeSceneSummary ?? true }),
  refreshCharacterInventory: (apiKey: string, characterId: string) =>
    invoke<InventoryRefreshResult>("refresh_character_inventory_cmd", { apiKey, characterId }),
  refreshGroupInventories: (apiKey: string, groupChatId: string) =>
    invoke<InventoryRefreshResult[]>("refresh_group_inventories_cmd", { apiKey, groupChatId }),
  setCharacterInventory: (characterId: string, inventory: InventoryItem[]) =>
    invoke<InventoryItem[]>("set_character_inventory_cmd", { characterId, inventory }),
  updateInventoryForMoment: (apiKey: string, messageId: string) =>
    invoke<UpdateInventoryForMomentResponse>("update_inventory_for_moment_cmd", { apiKey, messageId }),
  getInventoryUpdatesForMessages: (messageIds: string[]) =>
    invoke<InventoryUpdateRecord[]>("get_inventory_updates_for_messages_cmd", { messageIds }),
  generateCharacterJournal: (apiKey: string, characterId: string) =>
    invoke<JournalEntry>("generate_character_journal_cmd", { apiKey, characterId }),
  maybeGenerateCharacterJournal: (apiKey: string, characterId: string) =>
    invoke<MaybeJournalResult>("maybe_generate_character_journal_cmd", { apiKey, characterId }),
  listCharacterJournals: (characterId: string, limit?: number) =>
    invoke<JournalEntry[]>("list_character_journals_cmd", { characterId, limit: limit ?? null }),
  generateUserJournal: (apiKey: string, worldId: string) =>
    invoke<UserJournalEntry>("generate_user_journal_cmd", { apiKey, worldId }),
  maybeGenerateUserJournal: (apiKey: string, worldId: string) =>
    invoke<MaybeUserJournalResult>("maybe_generate_user_journal_cmd", { apiKey, worldId }),
  listUserJournals: (worldId: string, limit?: number) =>
    invoke<UserJournalEntry[]>("list_user_journals_cmd", { worldId, limit: limit ?? null }),
  generateMeanwhileEvents: (apiKey: string, worldId: string) =>
    invoke<MeanwhileEvent[]>("generate_meanwhile_events_cmd", { apiKey, worldId }),
  maybeGenerateMeanwhileEvents: (apiKey: string, worldId: string) =>
    invoke<MeanwhileEvent[]>("maybe_generate_meanwhile_events_cmd", { apiKey, worldId }),
  listMeanwhileEvents: (worldId: string, limit?: number) =>
    invoke<MeanwhileEvent[]>("list_meanwhile_events_cmd", { worldId, limit: limit ?? null }),
  generateDailyReading: (apiKey: string, worldId: string) =>
    invoke<DailyReading>("generate_daily_reading_cmd", { apiKey, worldId }),
  maybeGenerateDailyReading: (apiKey: string, worldId: string) =>
    invoke<MaybeDailyReadingResult>("maybe_generate_daily_reading_cmd", { apiKey, worldId }),
  listDailyReadings: (worldId: string, limit?: number) =>
    invoke<DailyReading[]>("list_daily_readings_cmd", { worldId, limit: limit ?? null }),
  getLatestDailyReading: (worldId: string) =>
    invoke<DailyReading | null>("get_latest_daily_reading_cmd", { worldId }),
  generateImaginedChapter: (apiKey: string, request: GenerateImaginedChapterRequest) =>
    invoke<{ chapterId: string }>("generate_imagined_chapter_cmd", { apiKey, request }),
  listImaginedChaptersForThread: (threadId: string) =>
    invoke<ImaginedChapter[]>("list_imagined_chapters_for_thread_cmd", { threadId }),
  getImaginedChapter: (chapterId: string) =>
    invoke<ImaginedChapter>("get_imagined_chapter_cmd", { chapterId }),
  deleteImaginedChapter: (chapterId: string) =>
    invoke<void>("delete_imagined_chapter_cmd", { chapterId }),
  renameImaginedChapter: (chapterId: string, title: string) =>
    invoke<void>("rename_imagined_chapter_cmd", { chapterId, title }),
  updateImaginedChapterSceneLocation: (chapterId: string, sceneLocation: string | null) =>
    invoke<void>("update_imagined_chapter_scene_location_cmd", { chapterId, sceneLocation }),
  getImaginedChapterImageUrl: (chapterId: string) =>
    invoke<string>("get_imagined_chapter_image_url_cmd", { chapterId }),
  canonizeImaginedChapter: (chapterId: string) =>
    invoke<{ breadcrumbMessageId: string }>("canonize_imagined_chapter_cmd", { chapterId }),
  decanonizeImaginedChapter: (chapterId: string) =>
    invoke<void>("decanonize_imagined_chapter_cmd", { chapterId }),
  bulkDecanonizeImaginedChaptersForThread: (threadId: string) =>
    invoke<{ decanonizedCount: number }>("bulk_decanonize_imagined_chapters_for_thread_cmd", { threadId }),

  // Genesis — auto-generate a full starter world + 2 characters with
  // hi-def portraits, world image, inventories, and all data populated.
  // Streams `genesis-stage` events for progress.
  autoGenerateWorldWithCharacters: (apiKey: string, hints?: GenesisHints) =>
    invoke<GenesisResult>("auto_generate_world_with_characters_cmd", { apiKey, hints: hints ?? null }),
  reflectReachingAsNobleQuest: (apiKey: string, worldId: string, reachingText: string) =>
    invoke<string>("reflect_reaching_as_noble_quest_cmd", { apiKey, worldId, reachingText }),

  // Quests
  createQuest: (worldId: string, title: string, description: string, originKind?: "user_authored" | "message" | "meanwhile" | "backstage", originRef?: string) =>
    invoke<Quest>("create_quest_cmd", { worldId, title, description, originKind: originKind ?? null, originRef: originRef ?? null }),
  listQuests: (worldId: string) =>
    invoke<Quest[]>("list_quests_cmd", { worldId }),
  getQuest: (questId: string) =>
    invoke<Quest>("get_quest_cmd", { questId }),
  updateQuest: (questId: string, title: string, description: string) =>
    invoke<Quest>("update_quest_cmd", { questId, title, description }),
  updateQuestNotes: (questId: string, notes: string) =>
    invoke<Quest>("update_quest_notes_cmd", { questId, notes }),
  completeQuest: (questId: string, completionNote: string) =>
    invoke<Quest>("complete_quest_cmd", { questId, completionNote }),
  abandonQuest: (questId: string, abandonmentNote: string) =>
    invoke<Quest>("abandon_quest_cmd", { questId, abandonmentNote }),
  reopenQuest: (questId: string) =>
    invoke<Quest>("reopen_quest_cmd", { questId }),
  deleteQuest: (questId: string) =>
    invoke<void>("delete_quest_cmd", { questId }),
};

export interface GenesisResult {
  world_id: string;
  character_ids: string[];
}

export interface GenesisHints {
  /** Freeform tone description. Empty / omitted → LLM picks from its mood seed. */
  tone?: string | null;
  /** One of "morning" | "midday" | "afternoon" | "evening" | "late night". */
  time_of_day?: string | null;
  /** Weather id from WEATHER_OPTIONS. */
  weather_key?: string | null;
}

export type GenesisReveal =
  | { kind: "world_named"; name: string; description: string }
  | { kind: "character_named"; character_id: string; name: string; identity: string; avatar_color: string }
  | { kind: "world_image_ready"; world_id: string }
  | { kind: "portrait_ready"; character_id: string };

export interface GenesisStageEvent {
  stage: string;
  detail: string;
  progress: number;
  reveal?: GenesisReveal;
}

export interface Quest {
  quest_id: string;
  world_id: string;
  title: string;
  description: string;
  notes: string;
  accepted_at: string;
  accepted_world_day: number | null;
  completed_at: string | null;
  completed_world_day: number | null;
  completion_note: string;
  abandoned_at: string | null;
  abandoned_world_day: number | null;
  abandonment_note: string;
  /** "user_authored" | "message" | "meanwhile" | "backstage" */
  origin_kind: string;
  origin_ref: string | null;
}

export interface ImaginedChapter {
  chapter_id: string;
  thread_id: string;
  world_day: number | null;
  title: string;
  seed_hint: string;
  scene_location: string | null;
  scene_description: string;
  image_id: string | null;
  content: string;
  created_at: string;
  /** message_id of the role='imagined_chapter' chat-history breadcrumb
   *  inserted when the chapter was canonized. Null until canonization. */
  breadcrumb_message_id: string | null;
  /** Whether this chapter has been blessed into canon. Pre-canon
   *  chapters live in the modal sidebar but don't appear in chat
   *  history and don't reach the dialogue prompt's history block. */
  canonized: boolean;
}

export interface GenerateImaginedChapterRequest {
  threadId: string;
  seedHint?: string;
  sceneLocation?: string;
  continueFromPrevious: boolean;
  imageTier?: "low" | "medium" | "high";
  /** Profundity dial. "Glimpse" = quiet daily moment, no excavation.
   *  "Opening" = one layer below default (default in UI). "Deep" =
   *  interior visible, real cost named. "Sacred" = confessional,
   *  threshold, rare. Omit to let the model pick. */
  depth?: "Glimpse" | "Opening" | "Deep" | "Sacred";
}

/** Streaming events emitted during chapter generation. */
export interface ImaginedChapterStageEvent {
  chapterId: string;
  phase: "inventing" | "rendering" | "writing";
  title?: string | null;
  toneHint?: string | null;
}
export interface ImaginedChapterImageEvent {
  chapterId: string;
  dataUrl: string;
  imageId: string;
}
export interface ImaginedChapterDoneEvent {
  chapterId: string;
  title: string;
  content: string;
}

export interface ReadingDomain {
  name: string;
  percent: number;
  phrase: string;
}

export interface DailyReading {
  reading_id: string;
  world_id: string;
  world_day: number;
  domains: ReadingDomain[];
  complication: string;
  created_at: string;
}

export interface MaybeDailyReadingResult {
  reading: DailyReading | null;
  refreshed: boolean;
}

export interface MaybeJournalResult {
  entry: JournalEntry | null;
  refreshed: boolean;
}

export interface JournalEntry {
  journal_id: string;
  character_id: string;
  world_day: number;
  content: string;
  created_at: string;
}

export interface UserJournalEntry {
  journal_id: string;
  world_id: string;
  world_day: number;
  content: string;
  created_at: string;
}

export interface MaybeUserJournalResult {
  entry: UserJournalEntry | null;
  refreshed: boolean;
}

export interface MeanwhileEvent {
  event_id: string;
  character_id: string;
  character_name: string;
  avatar_color: string;
  world_day: number;
  time_of_day: string;
  summary: string;
  created_at: string;
}
