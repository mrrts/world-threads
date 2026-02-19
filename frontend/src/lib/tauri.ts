import { invoke } from "@tauri-apps/api/core";
import { appDataDir } from "@tauri-apps/api/path";
import { Client, Stronghold } from "@tauri-apps/plugin-stronghold";

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
  location: { current_scene: string };
  global_arcs: Array<{ arc_id: string; status: string; notes: string }>;
  facts: Array<{ fact_id: string; text: string; confidence: string }>;
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
  is_archived: boolean;
  created_at: string;
  updated_at: string;
}

export interface CharacterState {
  mood: number;
  trust_user: number;
  goals: string[];
  open_loops: string[];
  last_seen: { day_index: number; time_of_day: string };
}

export interface Message {
  message_id: string;
  thread_id: string;
  role: "user" | "assistant" | "system";
  content: string;
  tokens_estimate: number;
  created_at: string;
}

export interface PaginatedMessages {
  messages: Message[];
  total: number;
}

export interface WorldEvent {
  event_id: string;
  world_id: string;
  day_index: number;
  time_of_day: string;
  summary: string;
  involved_characters: string[];
  hooks: string[];
  trigger_type: string;
  created_at: string;
}

export interface ModelConfig {
  dialogue_model: string;
  tick_model: string;
  memory_model: string;
  embedding_model: string;
}

export interface SendMessageResult {
  user_message: Message;
  assistant_message: Message;
  tick_result: { events: string[]; state_patch: Record<string, unknown>; next_hooks: string[] } | null;
  new_events: WorldEvent[];
  ai_reactions: Reaction[];
}

export interface Reaction {
  reaction_id: string;
  message_id: string;
  emoji: string;
  reactor: "user" | "assistant";
  created_at: string;
}

export interface UserProfile {
  world_id: string;
  display_name: string;
  description: string;
  facts: string[];
  avatar_file: string;
  updated_at: string;
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
  listWorlds: () => invoke<World[]>("list_worlds_cmd"),
  updateWorld: (world: World) => invoke<void>("update_world_cmd", { world }),
  deleteWorld: (worldId: string) => invoke<void>("delete_world_cmd", { worldId }),
  updateWorldState: (worldId: string, state: WorldState) => invoke<void>("update_world_state_cmd", { worldId, state }),

  getTodayUsage: () => invoke<DailyUsage>("get_today_usage_cmd"),

  getUserProfile: (worldId: string) => invoke<UserProfile | null>("get_user_profile_cmd", { worldId }),
  updateUserProfile: (profile: UserProfile) => invoke<void>("update_user_profile_cmd", { profile }),
  generateUserAvatar: (apiKey: string, worldId: string, formHint?: { display_name?: string; description?: string }) =>
    invoke<string>("generate_user_avatar_cmd", { apiKey, worldId, formHint: formHint ?? null }),
  uploadUserAvatar: (worldId: string, imageData: string) =>
    invoke<string>("upload_user_avatar_cmd", { worldId, imageData }),
  getUserAvatar: (worldId: string) =>
    invoke<string>("get_user_avatar_cmd", { worldId }),
  setUserAvatarFromGallery: (worldId: string, sourceFile: string) =>
    invoke<string>("set_user_avatar_from_gallery_cmd", { worldId, sourceFile }),

  listCharacters: (worldId: string) => invoke<Character[]>("list_characters_cmd", { worldId }),
  getCharacter: (characterId: string) => invoke<Character>("get_character_cmd", { characterId }),
  updateCharacter: (character: Character) => invoke<void>("update_character_cmd", { character }),
  createCharacter: (worldId: string, displayName: string) => invoke<Character>("create_character_cmd", { worldId, displayName }),
  deleteCharacter: (characterId: string) => invoke<void>("delete_character_cmd", { characterId }),
  archiveCharacter: (characterId: string) => invoke<void>("archive_character_cmd", { characterId }),
  unarchiveCharacter: (characterId: string) => invoke<void>("unarchive_character_cmd", { characterId }),
  listArchivedCharacters: (worldId: string) => invoke<Character[]>("list_archived_characters_cmd", { worldId }),

  sendMessage: (apiKey: string, characterId: string, content: string) =>
    invoke<SendMessageResult>("send_message_cmd", { apiKey, characterId, content }),
  getMessages: (characterId: string, limit?: number, offset?: number) =>
    invoke<PaginatedMessages>("get_messages_cmd", { characterId, limit, offset }),

  listWorldEvents: (worldId: string, limit?: number) =>
    invoke<WorldEvent[]>("list_world_events_cmd", { worldId, limit }),
  retconLastTick: (worldId: string) => invoke<WorldEvent | null>("retcon_last_tick_cmd", { worldId }),

  getModelConfig: () => invoke<ModelConfig>("get_model_config_cmd"),
  setModelConfig: (config: ModelConfig) => invoke<void>("set_model_config_cmd", { config }),
  getSetting: (key: string) => invoke<string | null>("get_setting_cmd", { key }),
  setSetting: (key: string, value: string) => invoke<void>("set_setting_cmd", { key, value }),
  getApiKey: () => getApiKeyFromVault(),
  setApiKey: (key: string) => setApiKeyInVault(key),
  migrateApiKey: () => migrateApiKeyToVault(),
  getBudgetMode: () => invoke<boolean>("get_budget_mode_cmd"),
  setBudgetMode: (enabled: boolean) => invoke<void>("set_budget_mode_cmd", { enabled }),

  getMemoryArtifacts: (subjectId: string, artifactType: string) =>
    invoke<MemoryArtifact[]>("get_memory_artifacts_cmd", { subjectId, artifactType }),
  getThreadSummary: (characterId: string) =>
    invoke<string>("get_thread_summary_cmd", { characterId }),

  generatePortrait: (apiKey: string, characterId: string, formHint?: { display_name?: string; identity?: string; backstory_facts?: unknown }) =>
    invoke<PortraitInfo>("generate_portrait_cmd", { apiKey, characterId, formHint: formHint ?? null }),
  listPortraits: (characterId: string) =>
    invoke<PortraitInfo[]>("list_portraits_cmd", { characterId }),
  setActivePortrait: (characterId: string, portraitId: string) =>
    invoke<void>("set_active_portrait_cmd", { characterId, portraitId }),
  setPortraitFromGallery: (characterId: string, sourceFile: string) =>
    invoke<PortraitInfo>("set_portrait_from_gallery_cmd", { characterId, sourceFile }),
  getActivePortrait: (characterId: string) =>
    invoke<PortraitInfo | null>("get_active_portrait_cmd", { characterId }),

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
};
