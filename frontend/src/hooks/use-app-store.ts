import { useCallback, useEffect, useState } from "react";
import { playChime } from "@/lib/chime";
import { api, type World, type Character, type Message, type ModelConfig, type Reaction, type PortraitInfo, type UserProfile, type WorldImageInfo, type GroupChat, type InventoryItem } from "@/lib/tauri";

export interface AppState {
  worlds: World[];
  activeWorld: World | null;
  characters: Character[];
  archivedCharacters: Character[];
  activeCharacter: Character | null;
  groupChats: GroupChat[];
  activeGroupChat: GroupChat | null;
  messages: Message[];
  reactions: Record<string, Reaction[]>;
  activePortraits: Record<string, PortraitInfo>;
  activeWorldImage: WorldImageInfo | null;
  userProfile: UserProfile | null;
  modelConfig: ModelConfig;
  apiKey: string;
  budgetMode: boolean;
  editingUserProfile: boolean;
  loading: boolean;
  autoRespond: boolean;
  notifyOnMessage: boolean;
  /** 0-5 index into a fixed font-size ladder for chat message bubbles.
   *  2 = default. Persisted as the "chat_font_size" setting. */
  chatFontSize: number;
  /** Character ID currently awaiting a response, or null */
  sending: string | null;
  /** Character ID currently generating a narrative, or null */
  generatingNarrative: string | null;
  /** Character ID currently generating an illustration, or null */
  generatingIllustration: string | null;
  generatingVideo: string | null;
  /** Message ID currently being adjusted, or null */
  adjustingMessageId: string | null;
  /** Which character is currently generating a response in a group chat */
  sendingCharacterId: string | null;
  /** Map of illustration message_id → video filename */
  videoFiles: Record<string, string>;
  aspectRatios: Record<string, number>;
  totalMessages: number;
  loadingOlder: boolean;
  loadingChat: boolean;
  chatError: string | null;
  lastFailedContent: string | null;
  error: string | null;
  /** Per-thread count of proactive pings the user hasn't replied to yet.
   *  Populated by `get_proactive_unread_counts_cmd`. Empty map when
   *  nothing is outstanding. Keyed by thread_id. */
  proactiveUnreadCounts: Record<string, number>;
}

const PAGE_SIZE = 20;
/** Initial + per-page size for chat message loading. Long histories
 *  were starting to cause noticeable render lag. */
const CHAT_PAGE_SIZE = 100;

const defaultModelConfig: ModelConfig = {
  dialogue_model: "gpt-4o",
  tick_model: "gpt-4o-mini",
  memory_model: "gpt-4o-mini",
  embedding_model: "text-embedding-3-small",
  image_model: "gpt-image-1.5",
  vision_model: "gpt-4.1",
  ai_provider: "openai",
  lmstudio_url: "http://127.0.0.1:1234",
};

export function useAppStore() {
  const [state, setState] = useState<AppState>({
    worlds: [],
    activeWorld: null,
    characters: [],
    archivedCharacters: [],
    activeCharacter: null,
    groupChats: [],
    activeGroupChat: null,
    messages: [],
    reactions: {},
    activePortraits: {},
    activeWorldImage: null,
    userProfile: null,
    modelConfig: defaultModelConfig,
    apiKey: "",
    budgetMode: false,
    editingUserProfile: false,
    totalMessages: 0,
    loadingOlder: false,
    loadingChat: false,
    autoRespond: true,
    notifyOnMessage: true,
    chatFontSize: 2,
    loading: true,
    sending: null,
    generatingNarrative: null,
    generatingIllustration: null,
    generatingVideo: null,
    adjustingMessageId: null,
    sendingCharacterId: null,
    videoFiles: {},
    aspectRatios: {},
    chatError: null,
    lastFailedContent: null,
    error: null,
    proactiveUnreadCounts: {},
  });

  const setError = useCallback((error: string | null) => {
    setState((s) => ({ ...s, error }));
    if (error) setTimeout(() => setState((s) => ({ ...s, error: null })), 5000);
  }, []);

  const loadActivePortraits = useCallback(async (characters: Character[]) => {
    const result: Record<string, PortraitInfo> = {};
    for (const ch of characters) {
      try {
        const p = await api.getActivePortrait(ch.character_id);
        if (p) result[ch.character_id] = p;
      } catch {
        // skip
      }
    }
    return result;
  }, []);

  const loadReactions = useCallback(async (messages: Message[]) => {
    if (messages.length === 0) return {};
    try {
      const ids = messages.map((m) => m.message_id).filter((id) => !id.startsWith("pending-"));
      if (ids.length === 0) return {};
      const reactions = await api.getReactions(ids);
      const grouped: Record<string, Reaction[]> = {};
      for (const r of reactions) {
        if (!grouped[r.message_id]) grouped[r.message_id] = [];
        grouped[r.message_id].push(r);
      }
      return grouped;
    } catch {
      return {};
    }
  }, []);

  const loadWorlds = useCallback(async () => {
    try {
      const worlds = await api.listWorlds();
      setState((s) => ({ ...s, worlds }));
      return worlds;
    } catch (e) {
      setError(String(e));
      return [];
    }
  }, [setError]);

  const loadInitial = useCallback(async () => {
    setState((s) => ({ ...s, loading: true }));
    try {
      const [worlds, modelConfig, apiKey, budgetMode, autoRespondSetting, notifySetting, fontSizeSetting] = await Promise.all([
        api.listWorlds(),
        api.getModelConfig(),
        api.migrateApiKey(),
        api.getBudgetMode(),
        api.getSetting("auto_respond").catch(() => null),
        api.getSetting("notify_on_message").catch(() => null),
        api.getSetting("chat_font_size").catch(() => null),
      ]);

      let activeWorld: World | null = null;
      let characters: Character[] = [];
      let archivedCharacters: Character[] = [];
      let activeCharacter: Character | null = null;
      let activeGroupChat: GroupChat | null = null;
      let groupChats: GroupChat[] = [];
      let messages: Message[] = [];
      let totalMessages = 0;
      let reactions: Record<string, Reaction[]> = {};
      let aspectRatios: Record<string, number> = {};
      let activePortraits: Record<string, PortraitInfo> = {};
      let activeWorldImage: WorldImageInfo | null = null;
      let userProfile: UserProfile | null = null;

      if (worlds.length > 0) {
        activeWorld = worlds[0];
        const wid = activeWorld.world_id;
        const [chars, archived, wImage, uProfile, gChats] = await Promise.all([
          api.listCharacters(wid),
          api.listArchivedCharacters(wid),
          api.getActiveWorldImage(wid),
          api.getUserProfile(wid),
          api.listGroupChats(wid),
        ]);
        characters = chars;
        archivedCharacters = archived;
        activeWorldImage = wImage;
        userProfile = uProfile;
        groupChats = gChats;
        activePortraits = await loadActivePortraits([...characters, ...archivedCharacters]);

        // Restore last active chat for this world
        const lastChat = await api.getSetting(`last_chat.${wid}`).catch(() => null);
        if (lastChat?.startsWith("group:")) {
          const gcId = lastChat.slice(6);
          const gc = groupChats.find((g) => g.group_chat_id === gcId);
          if (gc) {
            activeGroupChat = gc;
            const page = await api.getGroupMessages(gc.group_chat_id, CHAT_PAGE_SIZE, 0);
            messages = page.messages;
            totalMessages = page.total;
            reactions = await loadReactions(messages);
          }
        } else if (lastChat?.startsWith("char:")) {
          const charId = lastChat.slice(5);
          const ch = characters.find((c) => c.character_id === charId);
          if (ch) {
            activeCharacter = ch;
            const page = await api.getMessages(ch.character_id, CHAT_PAGE_SIZE, 0);
            messages = page.messages;
            totalMessages = page.total;
            [reactions, aspectRatios] = await Promise.all([
              loadReactions(messages),
              loadAspectRatios(messages),
            ]);
          }
        }

        // Fallback to first character if no last chat found
        if (!activeCharacter && !activeGroupChat && characters.length > 0) {
          activeCharacter = characters[0];
          const page = await api.getMessages(activeCharacter.character_id, CHAT_PAGE_SIZE, 0);
          messages = page.messages;
          totalMessages = page.total;
          [reactions, aspectRatios] = await Promise.all([
            loadReactions(messages),
            loadAspectRatios(messages),
          ]);
        }
      }

      setState({
        worlds,
        activeWorld,
        characters,
        archivedCharacters,
        activeCharacter,
        groupChats,
        activeGroupChat: activeGroupChat ?? null,
        messages,
        totalMessages,
        reactions,
        aspectRatios,
        activePortraits,
        activeWorldImage,
        userProfile,
        modelConfig,
        apiKey: apiKey ?? "",
        budgetMode,
        autoRespond: autoRespondSetting !== "false",
        notifyOnMessage: notifySetting !== "false",
        chatFontSize: (() => {
          const n = fontSizeSetting ? parseInt(fontSizeSetting, 10) : 2;
          return Number.isFinite(n) ? Math.max(0, Math.min(5, n)) : 2;
        })(),
        loadingOlder: false,
    loadingChat: false,
        loading: false,
        sending: null,
        generatingNarrative: null,
        generatingIllustration: null,
        generatingVideo: null,
    sendingCharacterId: null,
        videoFiles: {},
        error: null,
        editingUserProfile: false,
        chatError: null,
        lastFailedContent: null,
        adjustingMessageId: null,
        proactiveUnreadCounts: {},
      });
    } catch (e) {
      setError(String(e));
      setState((s) => ({ ...s, loading: false }));
    }
  }, [setError, loadReactions, loadActivePortraits]);

  useEffect(() => { loadInitial(); }, [loadInitial]);

  const selectWorld = useCallback(async (world: World) => {
    setState((s) => ({
      ...s,
      activeWorld: world,
      characters: [],
      archivedCharacters: [],
      activeCharacter: null,
      groupChats: [],
      activeGroupChat: null,
      messages: [],
      totalMessages: 0,
      reactions: {},
      activePortraits: {},
      activeWorldImage: null,
      userProfile: null,
    }));
    try {
      const [characters, archivedCharacters, activeWorldImage, userProfile, groupChats] = await Promise.all([
        api.listCharacters(world.world_id),
        api.listArchivedCharacters(world.world_id),
        api.getActiveWorldImage(world.world_id),
        api.getUserProfile(world.world_id),
        api.listGroupChats(world.world_id),
      ]);
      const activePortraits = await loadActivePortraits([...characters, ...archivedCharacters]);

      // Restore last active chat for this world
      let activeCharacter: Character | null = null;
      let activeGroupChat: GroupChat | null = null;
      let messages: Message[] = [];
      let totalMessages = 0;
      let reactions: Record<string, Reaction[]> = {};

      const lastChat = await api.getSetting(`last_chat.${world.world_id}`).catch(() => null);
      if (lastChat?.startsWith("group:")) {
        const gcId = lastChat.slice(6);
        const gc = groupChats.find((g) => g.group_chat_id === gcId);
        if (gc) {
          activeGroupChat = gc;
          const page = await api.getGroupMessages(gc.group_chat_id, CHAT_PAGE_SIZE, 0);
          messages = page.messages;
          totalMessages = page.total;
          reactions = await loadReactions(messages);
        }
      } else if (lastChat?.startsWith("char:")) {
        const charId = lastChat.slice(5);
        const ch = characters.find((c) => c.character_id === charId);
        if (ch) {
          activeCharacter = ch;
          const page = await api.getMessages(ch.character_id, CHAT_PAGE_SIZE, 0);
          messages = page.messages;
          totalMessages = page.total;
          reactions = await loadReactions(messages);
        }
      }

      // Fallback to first character if no last chat found
      if (!activeCharacter && !activeGroupChat && characters.length > 0) {
        activeCharacter = characters[0];
        const page = await api.getMessages(activeCharacter.character_id, CHAT_PAGE_SIZE, 0);
        messages = page.messages;
        totalMessages = page.total;
        reactions = await loadReactions(messages);
      }

      setState((s) => {
        if (s.activeWorld?.world_id !== world.world_id) return s;
        return { ...s, characters, archivedCharacters, activeCharacter, groupChats, activeGroupChat, messages, totalMessages, reactions, activePortraits, activeWorldImage, userProfile };
      });
    } catch (e) {
      setError(String(e));
    }
  }, [setError, loadReactions, loadActivePortraits]);

  const loadAspectRatios = useCallback(async (messages: Message[]) => {
    const illustrationIds = messages.filter((m) => m.role === "illustration").map((m) => m.message_id);
    if (illustrationIds.length === 0) return {};
    const result: Record<string, number> = {};
    for (const id of illustrationIds) {
      try {
        const ar = await api.getIllustrationAspectRatio(id);
        if (ar > 0) result[id] = ar;
      } catch { /* ignore */ }
    }
    return result;
  }, []);

  const selectCharacter = useCallback(async (character: Character) => {
    setState((s) => ({ ...s, activeCharacter: character, activeGroupChat: null, messages: [], totalMessages: 0, reactions: {}, editingUserProfile: false, chatError: null, lastFailedContent: null, loadingChat: true }));
    if (state.activeWorld) {
      api.setSetting(`last_chat.${state.activeWorld.world_id}`, `char:${character.character_id}`).catch(() => {});
    }
    try {
      const page = await api.getMessages(character.character_id, CHAT_PAGE_SIZE, 0);
      const [reactions, aspectRatios] = await Promise.all([
        loadReactions(page.messages),
        loadAspectRatios(page.messages),
      ]);
      setState((s) => {
        if (s.activeCharacter?.character_id !== character.character_id) return s;
        return { ...s, messages: page.messages, totalMessages: page.total, reactions, aspectRatios, loadingChat: false };
      });
    } catch (e) {
      setError(String(e));
      setState((s) => ({ ...s, loadingChat: false }));
    }
  }, [setError, loadReactions]);

  const selectGroupChat = useCallback(async (groupChat: GroupChat) => {
    setState((s) => ({ ...s, activeGroupChat: groupChat, activeCharacter: null, messages: [], totalMessages: 0, reactions: {}, editingUserProfile: false, chatError: null, lastFailedContent: null, loadingChat: true }));
    if (state.activeWorld) {
      api.setSetting(`last_chat.${state.activeWorld.world_id}`, `group:${groupChat.group_chat_id}`).catch(() => {});
    }
    try {
      const page = await api.getGroupMessages(groupChat.group_chat_id, CHAT_PAGE_SIZE, 0);
      const reactions = await loadReactions(page.messages);
      setState((s) => {
        if (s.activeGroupChat?.group_chat_id !== groupChat.group_chat_id) return s;
        return { ...s, messages: page.messages, totalMessages: page.total, reactions, loadingChat: false };
      });
    } catch (e) {
      setError(String(e));
      setState((s) => ({ ...s, loadingChat: false }));
    }
  }, [setError, loadReactions]);

  const createGroupChat = useCallback(async (characterIds: string[]) => {
    if (!state.activeWorld) return;
    try {
      const gc = await api.createGroupChat(state.activeWorld.world_id, characterIds);
      const groupChats = await api.listGroupChats(state.activeWorld.world_id);
      setState((s) => ({ ...s, groupChats }));
      await selectGroupChat(gc);
    } catch (e) {
      setError(String(e));
    }
  }, [state.activeWorld, setError, selectGroupChat]);

  const deleteGroupChat = useCallback(async (groupChatId: string) => {
    try {
      await api.deleteGroupChat(groupChatId);
      if (state.activeWorld) {
        const groupChats = await api.listGroupChats(state.activeWorld.world_id);
        setState((s) => ({
          ...s,
          groupChats,
          activeGroupChat: s.activeGroupChat?.group_chat_id === groupChatId ? null : s.activeGroupChat,
          messages: s.activeGroupChat?.group_chat_id === groupChatId ? [] : s.messages,
        }));
      }
    } catch (e) {
      setError(String(e));
    }
  }, [state.activeWorld, setError]);

  // Helper: extract world time fields for optimistic messages
  const worldTimeFields = () => ({
    world_day: state.activeWorld?.state?.time?.day_index ?? null,
    world_time: state.activeWorld?.state?.time?.time_of_day ?? null,
  });

  const sendGroupMessage = useCallback(async (content: string) => {
    if (!state.activeGroupChat || !state.apiKey) return;
    if (state.activeWorld) api.setSetting(`last_chat.${state.activeWorld.world_id}`, `group:${state.activeGroupChat.group_chat_id}`).catch(() => {});

    if (!state.autoRespond) {
      // Just save user message without triggering responses
      const optimisticMsg: Message = {
        message_id: `pending-${Date.now()}`,
        thread_id: "",
        role: "user",
        content,
        tokens_estimate: 0,
        sender_character_id: null,
        created_at: new Date().toISOString(),
        ...worldTimeFields(),
      };
      setState((s) => ({ ...s, chatError: null, messages: [...s.messages, optimisticMsg] }));
      try {
        const saved = await api.saveGroupUserMessage(state.activeGroupChat.group_chat_id, content);
        setState((s) => ({
          ...s,
          messages: [...s.messages.filter((m) => m.message_id !== optimisticMsg.message_id), saved],
          totalMessages: s.totalMessages + 1,
        }));
      } catch (e) {
        setState((s) => ({ ...s, chatError: String(e), messages: s.messages.filter((m) => m.message_id !== optimisticMsg.message_id) }));
      }
      return;
    }

    const optimisticMsg: Message = {
      message_id: `pending-${Date.now()}`,
      thread_id: "",
      role: "user",
      content,
      tokens_estimate: 0,
      sender_character_id: null,
      created_at: new Date().toISOString(),
      ...worldTimeFields(),
    };

    // Save user message first
    setState((s) => ({
      ...s,
      chatError: null,
      lastFailedContent: null,
      messages: [...s.messages, optimisticMsg],
    }));

    try {
      // Save user message to DB
      const saved = await api.saveGroupUserMessage(state.activeGroupChat.group_chat_id, content);
      setState((s) => ({
        ...s,
        messages: [...s.messages.filter((m) => m.message_id !== optimisticMsg.message_id), saved],
        totalMessages: s.totalMessages + 1,
      }));

      // Ask the backend which characters should respond and in what
      // order. Hybrid policy inside: name-mention → one specific
      // character; otherwise an LLM pick (with first_speaker promotion);
      // falls back to all-respond in character_ids order. The memberIds
      // list is the unconditional safety net if the call fails.
      const memberIds: string[] = Array.isArray(state.activeGroupChat.character_ids) ? state.activeGroupChat.character_ids : [];
      let charIds: string[];
      try {
        const picked = await api.pickGroupResponders(state.apiKey, state.activeGroupChat.group_chat_id, content);
        charIds = picked.filter((id) => memberIds.includes(id));
        if (charIds.length === 0) charIds = memberIds;
      } catch {
        charIds = memberIds;
      }
      for (const cid of charIds) {
        setState((s) => ({ ...s, sending: state.activeGroupChat!.group_chat_id, sendingCharacterId: cid }));
        const res = await api.promptGroupCharacter(state.apiKey, state.activeGroupChat!.group_chat_id, cid);
        setState((s) => {
          const merged = { ...s.reactions };
          for (const r of res.ai_reactions) {
            if (!merged[r.message_id]) merged[r.message_id] = [];
            merged[r.message_id].push(r);
          }
          return {
            ...s,
            messages: [...s.messages, res.assistant_message],
            totalMessages: s.totalMessages + 1,
            reactions: merged,
            sending: null,
            sendingCharacterId: null,
          };
        });
        if (state.notifyOnMessage) playChime();
      }
    } catch (e) {
      setState((s) => ({
        ...s,
        sending: null,
        chatError: String(e),
        lastFailedContent: content,
        messages: s.messages.filter((m) => m.message_id !== optimisticMsg.message_id),
      }));
    }
  }, [state.activeGroupChat, state.apiKey, state.autoRespond, state.notifyOnMessage]);

  const promptGroupCharacter = useCallback(async (characterId: string, addressTo?: string) => {
    if (!state.activeGroupChat || !state.apiKey) return;

    const charIds: string[] = Array.isArray(state.activeGroupChat.character_ids) ? state.activeGroupChat.character_ids : [];
    const selectedIdx = charIds.indexOf(characterId);

    // Characters that should respond: the selected one, plus all after it if auto-respond is on
    const respondingIds = state.autoRespond
      ? charIds.slice(selectedIdx)
      : [characterId];

    setState((s) => ({ ...s, sending: state.activeGroupChat!.group_chat_id, chatError: null }));

    try {
      for (const cid of respondingIds) {
        setState((s) => ({ ...s, sendingCharacterId: cid }));
        const res = await api.promptGroupCharacter(state.apiKey, state.activeGroupChat!.group_chat_id, cid, addressTo);
        setState((s) => {
          const merged = { ...s.reactions };
          for (const r of res.ai_reactions) {
            if (!merged[r.message_id]) merged[r.message_id] = [];
            merged[r.message_id].push(r);
          }
          return {
            ...s,
            messages: [...s.messages, res.assistant_message],
            totalMessages: s.totalMessages + 1,
            reactions: merged,
            sendingCharacterId: null,
          };
        });
        if (state.notifyOnMessage) playChime();
      }
      setState((s) => ({ ...s, sending: null }));
    } catch (e) {
      setState((s) => ({
        ...s,
        sending: null,
        chatError: String(e),
      }));
    }
  }, [state.activeGroupChat, state.apiKey, state.autoRespond, state.notifyOnMessage]);

  const selectUserProfile = useCallback(() => {
    setState((s) => ({ ...s, editingUserProfile: true }));
  }, []);

  const createWorld = useCallback(async (name: string) => {
    try {
      const world = await api.createWorld(name);
      const worlds = await api.listWorlds();
      const characters = await api.listCharacters(world.world_id);
      setState((s) => ({
        ...s,
        worlds,
        activeWorld: world,
        characters,
        activeCharacter: characters[0] ?? null,
        messages: [],
      }));
    } catch (e) {
      setError(String(e));
    }
  }, [setError]);

  const updateWorld = useCallback(async (world: World) => {
    try {
      await api.updateWorld(world);
      const worlds = await api.listWorlds();
      setState((s) => ({ ...s, worlds, activeWorld: world }));
    } catch (e) {
      setError(String(e));
    }
  }, [setError]);

  const deleteWorld = useCallback(async (worldId: string) => {
    try {
      await api.deleteWorld(worldId);
      const worlds = await api.listWorlds();
      setState((s) => ({ ...s, worlds, activeWorld: worlds[0] ?? null }));
      if (worlds.length > 0) selectWorld(worlds[0]);
    } catch (e) {
      setError(String(e));
    }
  }, [setError, selectWorld]);

  const updateCharacter = useCallback(async (character: Character) => {
    try {
      await api.updateCharacter(character);
      if (state.activeWorld) {
        const characters = await api.listCharacters(state.activeWorld.world_id);
        setState((s) => ({ ...s, characters, activeCharacter: character }));
      }
    } catch (e) {
      setError(String(e));
    }
  }, [state.activeWorld, setError]);

  const createCharacter = useCallback(async (name: string) => {
    if (!state.activeWorld) return;
    try {
      const character = await api.createCharacter(state.activeWorld.world_id, name);
      const characters = await api.listCharacters(state.activeWorld.world_id);
      setState((s) => ({ ...s, characters, activeCharacter: character, messages: [] }));
    } catch (e) {
      setError(String(e));
    }
  }, [state.activeWorld, setError]);

  const deleteCharacter = useCallback(async (characterId: string) => {
    if (!state.activeWorld) return;
    try {
      await api.deleteCharacter(characterId);
      const characters = await api.listCharacters(state.activeWorld.world_id);
      const activeCharacter = characters[0] ?? null;
      const page = activeCharacter ? await api.getMessages(activeCharacter.character_id, CHAT_PAGE_SIZE, 0) : { messages: [], total: 0 };
      setState((s) => ({ ...s, characters, activeCharacter, messages: page.messages, totalMessages: page.total }));
    } catch (e) {
      setError(String(e));
    }
  }, [state.activeWorld, setError]);

  const clearGroupChatHistory = useCallback(async (groupChatId: string, keepMedia: boolean) => {
    try {
      await api.clearGroupChatHistory(groupChatId, keepMedia);
      // When keeping media, illustrations remain — reload from the DB so they stay visible.
      const page = keepMedia ? await api.getGroupMessages(groupChatId, CHAT_PAGE_SIZE, 0) : { messages: [], total: 0 };
      setState((s) => ({
        ...s,
        messages: page.messages,
        totalMessages: page.total,
        reactions: {},
        chatError: null,
        lastFailedContent: null,
      }));
    } catch (e) {
      setError(String(e));
    }
  }, [setError]);

  const clearChatHistory = useCallback(async (characterId: string, keepMedia: boolean) => {
    try {
      await api.clearChatHistory(characterId, keepMedia);
      const page = keepMedia ? await api.getMessages(characterId, CHAT_PAGE_SIZE, 0) : { messages: [], total: 0 };
      setState((s) => ({
        ...s,
        messages: page.messages,
        totalMessages: page.total,
        reactions: {},
        chatError: null,
        lastFailedContent: null,
      }));
    } catch (e) {
      setError(String(e));
    }
  }, [setError]);

  const archiveCharacter = useCallback(async (characterId: string) => {
    if (!state.activeWorld) return;
    try {
      await api.archiveCharacter(characterId);
      const characters = await api.listCharacters(state.activeWorld.world_id);
      const archivedCharacters = await api.listArchivedCharacters(state.activeWorld.world_id);
      const wasActive = state.activeCharacter?.character_id === characterId;
      const activeCharacter = wasActive ? (characters[0] ?? null) : state.activeCharacter;
      let messages = wasActive ? [] as Message[] : state.messages;
      let totalMessages = wasActive ? 0 : state.totalMessages;
      if (wasActive && activeCharacter) {
        const page = await api.getMessages(activeCharacter.character_id, CHAT_PAGE_SIZE, 0);
        messages = page.messages;
        totalMessages = page.total;
      }
      setState((s) => ({ ...s, characters, archivedCharacters, activeCharacter, messages, totalMessages }));
    } catch (e) {
      setError(String(e));
    }
  }, [state.activeWorld, state.activeCharacter, state.messages, setError]);

  const unarchiveCharacter = useCallback(async (characterId: string) => {
    if (!state.activeWorld) return;
    try {
      await api.unarchiveCharacter(characterId);
      const characters = await api.listCharacters(state.activeWorld.world_id);
      const archivedCharacters = await api.listArchivedCharacters(state.activeWorld.world_id);
      setState((s) => ({ ...s, characters, archivedCharacters }));
    } catch (e) {
      setError(String(e));
    }
  }, [state.activeWorld, setError]);

  const setAutoRespond = useCallback((enabled: boolean) => {
    setState((s) => ({ ...s, autoRespond: enabled }));
    api.setSetting("auto_respond", enabled ? "true" : "false").catch(() => {});
  }, []);

  const setNotifyOnMessage = useCallback((enabled: boolean) => {
    setState((s) => ({ ...s, notifyOnMessage: enabled }));
    api.setSetting("notify_on_message", enabled ? "true" : "false").catch(() => {});
  }, []);

  const setChatFontSize = useCallback((level: number) => {
    const clamped = Math.max(0, Math.min(5, Math.round(level)));
    setState((s) => ({ ...s, chatFontSize: clamped }));
    api.setSetting("chat_font_size", String(clamped)).catch(() => {});
  }, []);

  const sendMessage = useCallback(async (content: string) => {
    if (!state.activeCharacter) return;
    if (state.activeWorld) api.setSetting(`last_chat.${state.activeWorld.world_id}`, `char:${state.activeCharacter.character_id}`).catch(() => {});

    // When auto-respond is off, just save the user message without triggering AI
    if (!state.autoRespond) {
      const optimisticMsg: Message = {
        message_id: `pending-${Date.now()}`,
        thread_id: "",
        role: "user",
        content,
        tokens_estimate: 0,
        created_at: new Date().toISOString(),
        ...worldTimeFields(),
      };

      setState((s) => ({
        ...s,
        chatError: null,
        lastFailedContent: null,
        messages: [...s.messages, optimisticMsg],
      }));

      try {
        const saved = await api.saveUserMessage(state.activeCharacter.character_id, content);
        setState((s) => ({
          ...s,
          messages: [
            ...s.messages.filter((m) => m.message_id !== optimisticMsg.message_id),
            saved,
          ],
          totalMessages: s.totalMessages + 1,
        }));
      } catch (e) {
        setState((s) => ({
          ...s,
          chatError: String(e),
          lastFailedContent: content,
          messages: s.messages.filter((m) => m.message_id !== optimisticMsg.message_id),
        }));
      }
      return;
    }

    if (!state.apiKey) return;

    const optimisticMsg: Message = {
      message_id: `pending-${Date.now()}`,
      thread_id: "",
      role: "user",
      content,
      tokens_estimate: 0,
      created_at: new Date().toISOString(),
      ...worldTimeFields(),
    };

    setState((s) => ({
      ...s,
      sending: state.activeCharacter!.character_id,
      chatError: null,
      lastFailedContent: null,
      messages: [...s.messages, optimisticMsg],
    }));

    try {
      const result = await api.sendMessage(state.apiKey, state.activeCharacter.character_id, content);
      const freshWorld = state.activeWorld ? await api.getWorld(state.activeWorld.world_id) : null;
      const freshCharacters = state.activeWorld ? await api.listCharacters(state.activeWorld.world_id) : [];
      setState((s) => {
        // If the user edited their message while the response was in flight, the
        // optimistic message in state has diverged from what the backend saved.
        // Use the edited content locally and persist it to the real message.
        const pending = s.messages.find((m) => m.message_id === optimisticMsg.message_id);
        const editedDuringWait = pending && pending.content !== content;
        const finalUserMsg = editedDuringWait
          ? { ...result.user_message, content: pending!.content }
          : result.user_message;
        if (editedDuringWait) {
          api.editMessageContent(finalUserMsg.message_id, finalUserMsg.content, false).catch(() => {});
        }
        const merged = { ...s.reactions };
        for (const r of result.ai_reactions) {
          if (!merged[r.message_id]) merged[r.message_id] = [];
          merged[r.message_id].push(r);
        }
        return {
          ...s,
          messages: [
            ...s.messages.filter((m) => m.message_id !== optimisticMsg.message_id),
            finalUserMsg,
            result.assistant_message,
          ],
          totalMessages: s.totalMessages + 2,
          reactions: merged,
          activeWorld: freshWorld,
          characters: freshCharacters,
          activeCharacter: freshCharacters.find(c => c.character_id === s.activeCharacter?.character_id) ?? s.activeCharacter,
          sending: null,
        };
      });
      if (state.notifyOnMessage) playChime();
    } catch (e) {
      setState((s) => ({
        ...s,
        sending: false,
        chatError: String(e),
        lastFailedContent: s.messages.find((m) => m.message_id === optimisticMsg.message_id)?.content ?? content,
        messages: s.messages.filter((m) => m.message_id !== optimisticMsg.message_id),
      }));
    }
  }, [state.activeCharacter, state.apiKey, state.activeWorld, state.autoRespond, state.notifyOnMessage]);

  // ── Proactive pings ────────────────────────────────────────────────────
  //
  // Refresh the per-thread unread badge counts. Cheap single DB call, safe
  // to run on app focus, on tick, or after any user action that might have
  // reset a counter.
  const refreshProactiveUnreadCounts = useCallback(async () => {
    try {
      const counts = await api.getProactiveUnreadCounts();
      setState((s) => ({ ...s, proactiveUnreadCounts: counts }));
    } catch {
      // Non-fatal — badge just won't update this tick.
    }
  }, []);

  // Sweep all non-archived characters and ask the backend to consider a
  // proactive ping for each. Backend enforces eligibility gates (quiet
  // window, cooldown, consecutive-cap); most calls return immediately with
  // a `skipped_reason`. Messages that DO arrive are appended to the active
  // chat if that chat is open, otherwise they just show up on next load.
  const runProactivePingSweep = useCallback(async () => {
    if (!state.apiKey) return;
    const characters = state.characters;
    if (characters.length === 0) return;
    let anyFired = false;
    for (const ch of characters) {
      try {
        const result = await api.tryProactivePing(state.apiKey, ch.character_id);
        if (result.message) {
          anyFired = true;
          setState((s) => {
            // Append to the live message list only if this is the chat
            // the user is currently looking at. Otherwise the message is
            // already persisted and will load when they open the thread;
            // the badge is what flags it in the meantime.
            const inActiveChat = s.activeCharacter?.character_id === ch.character_id;
            return inActiveChat
              ? { ...s, messages: [...s.messages, result.message!], totalMessages: s.totalMessages + 1 }
              : s;
          });
        }
      } catch {
        // Per-character failure shouldn't stop the sweep.
      }
    }
    if (anyFired) {
      if (state.notifyOnMessage) playChime();
      refreshProactiveUnreadCounts();
    }
  }, [state.apiKey, state.characters, state.activeCharacter, state.notifyOnMessage, refreshProactiveUnreadCounts]);

  const promptCharacter = useCallback(async () => {
    if (!state.activeCharacter || !state.apiKey) return;
    if (state.activeWorld) api.setSetting(`last_chat.${state.activeWorld.world_id}`, `char:${state.activeCharacter.character_id}`).catch(() => {});
    setState((s) => ({ ...s, sending: state.activeCharacter!.character_id, chatError: null }));

    try {
      const result = await api.promptCharacter(state.apiKey, state.activeCharacter.character_id);
      setState((s) => ({
        ...s,
        messages: [...s.messages, result.assistant_message],
        totalMessages: s.totalMessages + 1,
        sending: null,
      }));
      if (state.notifyOnMessage) playChime();
    } catch (e) {
      setState((s) => ({
        ...s,
        sending: false,
        chatError: String(e),
      }));
    }
  }, [state.activeCharacter, state.apiKey]);

  // Generate a dream-journal entry for the currently-active solo character.
  // The backend persists it as a "dream"-role message; we append it to the
  // live message list so it renders immediately in the chat.
  const generateDream = useCallback(async () => {
    if (!state.activeCharacter || !state.apiKey) return;
    const charId = state.activeCharacter.character_id;
    setState((s) => ({ ...s, sending: charId, generatingNarrative: charId, chatError: null }));
    try {
      const result = await api.generateDream(state.apiKey, charId);
      setState((s) => ({
        ...s,
        messages: [...s.messages, result.dream_message],
        totalMessages: s.totalMessages + 1,
        sending: null,
        generatingNarrative: null,
      }));
      if (state.notifyOnMessage) playChime();
    } catch (e) {
      setState((s) => ({ ...s, sending: null, generatingNarrative: null, chatError: String(e) }));
    }
  }, [state.activeCharacter, state.apiKey, state.notifyOnMessage]);

  const generateNarrative = useCallback(async (customInstructions?: string) => {
    const isGroup = !!state.activeGroupChat && !state.activeCharacter;
    const entityId = isGroup ? state.activeGroupChat?.group_chat_id : state.activeCharacter?.character_id;
    if (!entityId || !state.apiKey) return;

    setState((s) => ({ ...s, sending: entityId, generatingNarrative: entityId, chatError: null }));
    try {
      const result = isGroup
        ? await api.generateGroupNarrative(state.apiKey, entityId, customInstructions)
        : await api.generateNarrative(state.apiKey, entityId, customInstructions);
      setState((s) => ({ ...s, messages: [...s.messages, result.narrative_message], totalMessages: s.totalMessages + 1, sending: null, generatingNarrative: null }));
      if (state.notifyOnMessage) playChime();
    } catch (e) {
      setState((s) => ({ ...s, sending: null, generatingNarrative: null, chatError: String(e) }));
    }
  }, [state.activeCharacter, state.activeGroupChat, state.apiKey, state.notifyOnMessage]);

  const generateIllustration = useCallback(async (qualityTier?: string, customInstructions?: string, previousIllustrationId?: string, includeSceneSummary?: boolean) => {
    const isGroup = !!state.activeGroupChat && !state.activeCharacter;
    const entityId = isGroup ? state.activeGroupChat?.group_chat_id : state.activeCharacter?.character_id;
    if (!entityId || !state.apiKey) return;

    setState((s) => ({ ...s, sending: entityId, generatingIllustration: entityId, chatError: null }));
    try {
      const result = isGroup
        ? await api.generateGroupIllustration(state.apiKey, entityId, qualityTier, customInstructions, previousIllustrationId, includeSceneSummary)
        : await api.generateIllustration(state.apiKey, entityId, qualityTier, customInstructions, previousIllustrationId, includeSceneSummary);
      setState((s) => ({ ...s, messages: [...s.messages, result.illustration_message], totalMessages: s.totalMessages + 1, sending: null, generatingIllustration: null }));
      if (state.notifyOnMessage) playChime();
    } catch (e) {
      setState((s) => ({ ...s, sending: null, generatingIllustration: null, chatError: String(e) }));
    }
  }, [state.activeCharacter, state.activeGroupChat, state.apiKey, state.notifyOnMessage]);

  const adjustMessage = useCallback(async (messageId: string, instructions: string) => {
    if (!state.apiKey) return;
    const isGroup = !!state.activeGroupChat && !state.activeCharacter;

    setState((s) => ({ ...s, adjustingMessageId: messageId, chatError: null }));

    try {
      const updated = await api.adjustMessage(state.apiKey, messageId, instructions, isGroup);
      setState((s) => ({
        ...s,
        adjustingMessageId: null,
        messages: s.messages.map((m) => m.message_id === messageId ? updated : m),
      }));
      if (state.notifyOnMessage) playChime();
    } catch (e) {
      setState((s) => ({
        ...s,
        adjustingMessageId: null,
        chatError: String(e),
      }));
    }
  }, [state.apiKey, state.activeGroupChat, state.activeCharacter, state.notifyOnMessage]);

  const editMessageContent = useCallback(async (messageId: string, content: string) => {
    // Pending (optimistic) messages have no DB row yet — just update state locally.
    // sendMessage's success path will detect the diverged content and persist it
    // with an editMessageContent call once the real message_id is known.
    if (messageId.startsWith("pending-")) {
      setState((s) => ({
        ...s,
        messages: s.messages.map((m) => m.message_id === messageId ? { ...m, content } : m),
      }));
      return;
    }
    const isGroup = !!state.activeGroupChat && !state.activeCharacter;
    try {
      await api.editMessageContent(messageId, content, isGroup);
      setState((s) => ({
        ...s,
        messages: s.messages.map((m) => m.message_id === messageId ? { ...m, content } : m),
      }));
    } catch (e) {
      setState((s) => ({ ...s, chatError: String(e) }));
    }
  }, [state.activeGroupChat, state.activeCharacter]);

  const deleteMessage = useCallback(async (messageId: string) => {
    const isGroup = !!state.activeGroupChat && !state.activeCharacter;
    try {
      await api.deleteMessage(messageId, isGroup);
      setState((s) => ({
        ...s,
        messages: s.messages.filter((m) => m.message_id !== messageId),
        totalMessages: s.totalMessages - 1,
      }));
    } catch (e) {
      setState((s) => ({ ...s, chatError: String(e) }));
    }
  }, [state.activeGroupChat, state.activeCharacter]);

  const deleteIllustration = useCallback(async (messageId: string) => {
    try {
      await api.deleteIllustration(messageId);
      setState((s) => ({
        ...s,
        messages: s.messages.filter((m) => m.message_id !== messageId),
        totalMessages: s.totalMessages - 1,
      }));
    } catch (e) {
      setState((s) => ({ ...s, chatError: String(e) }));
    }
  }, []);

  const regenerateIllustration = useCallback(async (messageId: string) => {
    const isGroup = !!state.activeGroupChat && !state.activeCharacter;
    const entityId = isGroup ? state.activeGroupChat?.group_chat_id : state.activeCharacter?.character_id;
    if (!entityId || !state.apiKey) return;

    setState((s) => ({ ...s, sending: entityId, generatingIllustration: entityId, chatError: null, messages: s.messages.filter((m) => m.message_id !== messageId), totalMessages: s.totalMessages - 1 }));
    try {
      let result;
      if (isGroup) {
        await api.deleteIllustration(messageId);
        result = await api.generateGroupIllustration(state.apiKey, entityId);
      } else {
        result = await api.regenerateIllustration(state.apiKey, entityId, messageId);
      }
      setState((s) => ({ ...s, messages: [...s.messages, result.illustration_message], totalMessages: s.totalMessages + 1, sending: null, generatingIllustration: null }));
      if (state.notifyOnMessage) playChime();
    } catch (e) {
      setState((s) => ({ ...s, sending: null, generatingIllustration: null, chatError: String(e) }));
    }
  }, [state.activeCharacter, state.activeGroupChat, state.apiKey, state.notifyOnMessage]);

  const adjustIllustration = useCallback(async (messageId: string, instructions: string) => {
    const isGroup = !!state.activeGroupChat && !state.activeCharacter;
    const entityId = isGroup ? state.activeGroupChat?.group_chat_id : state.activeCharacter?.character_id;
    if (!entityId || !state.apiKey) return;

    setState((s) => ({ ...s, sending: entityId, generatingIllustration: entityId, chatError: null, messages: s.messages.filter((m) => m.message_id !== messageId), totalMessages: s.totalMessages - 1 }));
    try {
      let result;
      if (isGroup) {
        result = await api.generateGroupIllustration(state.apiKey, entityId, undefined, instructions, messageId);
        await api.deleteIllustration(messageId).catch(() => {});
      } else {
        result = await api.adjustIllustration(state.apiKey, entityId, messageId, instructions);
      }
      setState((s) => ({ ...s, messages: [...s.messages, result.illustration_message], totalMessages: s.totalMessages + 1, sending: null, generatingIllustration: null }));
      if (state.notifyOnMessage) playChime();
    } catch (e) {
      setState((s) => ({ ...s, sending: null, generatingIllustration: null, chatError: String(e) }));
    }
  }, [state.activeCharacter, state.activeGroupChat, state.apiKey, state.notifyOnMessage]);

  const loadVideoFiles = useCallback(async (messages: Message[]) => {
    const illustrationIds = messages.filter((m) => m.role === "illustration").map((m) => m.message_id);
    if (illustrationIds.length === 0) return {};
    const result: Record<string, string> = {};
    for (const id of illustrationIds) {
      try {
        const vf = await api.getVideoFile(id);
        if (vf) result[id] = vf;
      } catch { /* ignore */ }
    }
    return result;
  }, []);

  const generateVideo = useCallback(async (illustrationMessageId: string, customPrompt?: string, durationSeconds?: number, style?: string, includeContext?: boolean) => {
    const characterId = state.activeCharacter?.character_id ?? "";
    if (!state.apiKey || (!state.activeCharacter && !state.activeGroupChat)) return;

    const googleApiKey = await api.getGoogleApiKey();
    if (!googleApiKey) {
      setState((s) => ({ ...s, chatError: "Google AI Studio API key required for video generation. Add it in Settings." }));
      return;
    }

    setState((s) => ({ ...s, generatingVideo: illustrationMessageId, chatError: null }));

    try {
      const videoFile = await api.generateVideo(state.apiKey, googleApiKey, characterId, illustrationMessageId, customPrompt, durationSeconds, style, includeContext);
      setState((s) => ({
        ...s,
        generatingVideo: null,
    sendingCharacterId: null,
        videoFiles: { ...s.videoFiles, [illustrationMessageId]: videoFile },
      }));
      if (state.notifyOnMessage) playChime();
    } catch (e) {
      const err = String(e);
      const userMsg = err.includes("DAILY_LIMIT_REACHED")
        ? "You've reached the max daily video generations."
        : err.includes("RATE_LIMITED")
          ? "Video generation rate limit reached. Try again in a few minutes."
          : err;
      setState((s) => ({
        ...s,
        generatingVideo: null,
    sendingCharacterId: null,
        chatError: userMsg,
      }));
    }
  }, [state.activeCharacter, state.apiKey]);

  const resetToMessage = useCallback(async (messageId: string) => {
    if ((!state.activeCharacter && !state.activeGroupChat) || !state.apiKey) return;

    const anchorMsg = state.messages.find((m) => m.message_id === messageId);
    const isUserMsg = anchorMsg?.role === "user";
    const isGroupChat = !!state.activeGroupChat;

    setState((s) => ({
      ...s,
      messages: s.messages.filter((m) => {
        if (m.message_id === messageId) return true;
        return m.created_at < anchorMsg!.created_at ||
          (m.created_at === anchorMsg!.created_at && m.message_id === messageId);
      }),
      sending: isUserMsg && !isGroupChat ? (state.activeCharacter?.character_id ?? null) : null,
      chatError: null,
    }));

    try {
      const characterId = isGroupChat ? "" : (state.activeCharacter?.character_id ?? "");
      const result = await api.resetToMessage(state.apiKey, characterId, messageId);

      if (result.new_response) {
        setState((s) => {
          const merged = { ...s.reactions };
          for (const r of result.new_response!.ai_reactions) {
            if (!merged[r.message_id]) merged[r.message_id] = [];
            merged[r.message_id].push(r);
          }
          return {
            ...s,
            messages: [...s.messages, result.new_response!.assistant_message],
            totalMessages: s.totalMessages - result.deleted_count + 1,
            reactions: merged,
            sending: null,
          };
        });
      } else {
        setState((s) => ({
          ...s,
          totalMessages: s.totalMessages - result.deleted_count,
          sending: null,
        }));
      }

      // For group chats: if auto-respond is on and anchor was user message, trigger all character responses
      if (isGroupChat && isUserMsg && state.autoRespond && state.activeGroupChat) {
        const charIds: string[] = Array.isArray(state.activeGroupChat.character_ids) ? state.activeGroupChat.character_ids : [];
        setState((s) => ({ ...s, sending: state.activeGroupChat!.group_chat_id }));
        try {
          for (const cid of charIds) {
            setState((s) => ({ ...s, sendingCharacterId: cid }));
            const res = await api.promptGroupCharacter(state.apiKey, state.activeGroupChat.group_chat_id, cid);
            setState((s) => {
              const merged = { ...s.reactions };
              for (const r of res.ai_reactions) {
                if (!merged[r.message_id]) merged[r.message_id] = [];
                merged[r.message_id].push(r);
              }
              return {
                ...s,
                messages: [...s.messages, res.assistant_message],
                totalMessages: s.totalMessages + 1,
                reactions: merged,
                sendingCharacterId: null,
              };
            });
          }
        } catch { /* non-fatal */ }
        setState((s) => ({ ...s, sending: null }));
      }
    } catch (e) {
      // Reload messages from DB to get correct state after partial failure
      if (state.activeCharacter) {
        const page = await api.getMessages(state.activeCharacter.character_id, CHAT_PAGE_SIZE, 0);
        const reactions = await loadReactions(page.messages);
        setState((s) => ({
          ...s,
          messages: page.messages,
          totalMessages: page.total,
          reactions,
          sending: null,
          chatError: String(e),
        }));
      } else if (state.activeGroupChat) {
        const page = await api.getGroupMessages(state.activeGroupChat.group_chat_id, CHAT_PAGE_SIZE, 0);
        setState((s) => ({
          ...s,
          messages: page.messages,
          totalMessages: page.total,
          sending: null,
          chatError: String(e),
        }));
      }
    }
  }, [state.activeCharacter, state.apiKey, state.messages, loadReactions]);

  const clearChatError = useCallback(() => {
    setState((s) => ({ ...s, chatError: null, lastFailedContent: null }));
  }, []);

  // Load the next page of older messages on demand. Prepends to the
  // existing list. No-op if we're already showing all or another load is
  // in flight.
  const loadEarlierMessages = useCallback(async () => {
    setState((s) => {
      if (s.loadingOlder) return s;
      if (s.messages.length >= s.totalMessages) return s;
      return { ...s, loadingOlder: true };
    });
    try {
      const activeCharacter = state.activeCharacter;
      const activeGroup = state.activeGroupChat;
      const offset = state.messages.length;
      let older: Message[] = [];
      if (activeCharacter) {
        const page = await api.getMessages(activeCharacter.character_id, CHAT_PAGE_SIZE, offset);
        older = page.messages;
      } else if (activeGroup) {
        const page = await api.getGroupMessages(activeGroup.group_chat_id, CHAT_PAGE_SIZE, offset);
        older = page.messages;
      }
      const extraReactions = await loadReactions(older);
      setState((s) => ({
        ...s,
        messages: [...older, ...s.messages],
        reactions: { ...s.reactions, ...extraReactions },
        loadingOlder: false,
      }));
    } catch (e) {
      setState((s) => ({ ...s, loadingOlder: false }));
      setError(String(e));
    }
  }, [state.activeCharacter, state.activeGroupChat, state.messages.length, loadReactions, setError]);

  const setApiKey = useCallback(async (key: string) => {
    try {
      await api.setApiKey(key);
      setState((s) => ({ ...s, apiKey: key }));
    } catch (e) {
      setError(String(e));
    }
  }, [setError]);

  const setModelConfig = useCallback(async (config: ModelConfig) => {
    try {
      await api.setModelConfig(config);
      setState((s) => ({ ...s, modelConfig: config }));
    } catch (e) {
      setError(String(e));
    }
  }, [setError]);

  const setBudgetMode = useCallback(async (enabled: boolean) => {
    try {
      await api.setBudgetMode(enabled);
      setState((s) => ({ ...s, budgetMode: enabled }));
    } catch (e) {
      setError(String(e));
    }
  }, [setError]);

  const updateWorldState = useCallback(async (worldState: World["state"]) => {
    if (!state.activeWorld) return;
    try {
      await api.updateWorldState(state.activeWorld.world_id, worldState);
      const freshWorld = await api.getWorld(state.activeWorld.world_id);
      setState((s) => ({ ...s, activeWorld: freshWorld }));
    } catch (e) {
      setError(String(e));
    }
  }, [state.activeWorld, setError]);


  const updateUserProfile = useCallback(async (profile: UserProfile) => {
    try {
      await api.updateUserProfile(profile);
      setState((s) => ({ ...s, userProfile: profile }));
    } catch (e) {
      setError(String(e));
    }
  }, [setError]);

  const loadUserProfile = useCallback(async (worldId: string) => {
    try {
      const userProfile = await api.getUserProfile(worldId);
      setState((s) => ({ ...s, userProfile }));
    } catch (e) {
      setError(String(e));
    }
  }, [setError]);

  const refreshWorldImage = useCallback(async () => {
    if (!state.activeWorld) return;
    try {
      const activeWorldImage = await api.getActiveWorldImage(state.activeWorld.world_id);
      setState((s) => ({ ...s, activeWorldImage }));
    } catch {
      // ignore
    }
  }, [state.activeWorld]);

  const refreshPortrait = useCallback(async (characterId: string) => {
    try {
      const p = await api.getActivePortrait(characterId);
      setState((s) => {
        const updated = { ...s.activePortraits };
        if (p) { updated[characterId] = p; } else { delete updated[characterId]; }
        return { ...s, activePortraits: updated };
      });
    } catch {
      // ignore
    }
  }, []);

  // Ask the vision model to (re)describe a character from their current
  // active portrait. Cached by portrait_id on the backend — repeated
  // calls with no portrait change are cheap no-ops. Swallows errors so a
  // bad key or offline run doesn't cascade through the UI.
  const refreshVisualDescription = useCallback(async (characterId: string) => {
    if (!state.apiKey) return;
    try {
      const updated = await api.generateCharacterVisualDescription(state.apiKey, characterId);
      setState((s) => ({
        ...s,
        characters: s.characters.map((c) =>
          c.character_id === characterId ? updated : c
        ),
        activeCharacter:
          s.activeCharacter?.character_id === characterId ? updated : s.activeCharacter,
      }));
    } catch {
      // ignore
    }
  }, [state.apiKey]);

  // Ask the backend whether this character is overdue for an inventory
  // refresh; if so, it runs the seed/refresh LLM call and returns the
  // new inventory. Backend no-ops quickly when still fresh. Applies the
  // returned inventory to the local store so any open popover / card
  // updates without a round-trip on the next render.
  const refreshCharacterInventory = useCallback(async (characterId: string) => {
    try {
      const res = await api.refreshCharacterInventory(state.apiKey ?? "", characterId);
      if (!res.refreshed) return;
      setState((s) => ({
        ...s,
        characters: s.characters.map((c) =>
          c.character_id === characterId ? { ...c, inventory: res.inventory } : c
        ),
        activeCharacter:
          s.activeCharacter?.character_id === characterId
            ? { ...s.activeCharacter, inventory: res.inventory }
            : s.activeCharacter,
      }));
    } catch {
      // ignore — backend errors already logged; noop on cooldown is fine
    }
  }, [state.apiKey]);

  // Patch local state with a user-edited inventory (from the settings
  // editor). Skips the backend LLM call — the editor persists to DB
  // via setCharacterInventory directly; this is only the in-memory
  // reflection so popovers/cards update without a round-trip.
  const applyCharacterInventoryEdit = useCallback((characterId: string, inventory: InventoryItem[]) => {
    setState((s) => ({
      ...s,
      characters: s.characters.map((c) =>
        c.character_id === characterId ? { ...c, inventory } : c
      ),
      activeCharacter:
        s.activeCharacter?.character_id === characterId
          ? { ...s.activeCharacter, inventory }
          : s.activeCharacter,
    }));
  }, []);

  // Parallel variant for group chats: one refresh per member. Backend
  // fans out concurrently; we merge any that came back refreshed into
  // the local store in a single setState so the UI doesn't flicker.
  const refreshGroupInventories = useCallback(async (groupChatId: string) => {
    try {
      const results = await api.refreshGroupInventories(state.apiKey ?? "", groupChatId);
      const updatedMap = new Map(results.filter((r) => r.refreshed).map((r) => [r.character_id, r.inventory]));
      if (updatedMap.size === 0) return;
      setState((s) => ({
        ...s,
        characters: s.characters.map((c) => {
          const inv = updatedMap.get(c.character_id);
          return inv ? { ...c, inventory: inv } : c;
        }),
        activeCharacter: s.activeCharacter && updatedMap.has(s.activeCharacter.character_id)
          ? { ...s.activeCharacter, inventory: updatedMap.get(s.activeCharacter.character_id)! }
          : s.activeCharacter,
      }));
    } catch {
      // ignore
    }
  }, [state.apiKey]);

  // Backfill sweep: one call per character whose description is missing
  // or out of date versus the active portrait. Spaced out by 500ms so we
  // don't stampede the vision endpoint on app open.
  const backfillVisualDescriptions = useCallback(async () => {
    if (!state.apiKey || !state.activeWorld) return;
    try {
      const ids = await api.listCharactersNeedingVisualDescription(state.activeWorld.world_id);
      for (const id of ids) {
        await refreshVisualDescription(id);
        await new Promise((r) => setTimeout(r, 500));
      }
    } catch {
      // ignore
    }
  }, [state.apiKey, state.activeWorld, refreshVisualDescription]);

  const toggleReaction = useCallback(async (messageId: string, emoji: string) => {
    const existing = state.reactions[messageId] ?? [];
    const alreadyReacted = existing.some((r) => r.emoji === emoji && r.reactor === "user");

    // Optimistic update
    if (alreadyReacted) {
      setState((s) => ({
        ...s,
        reactions: {
          ...s.reactions,
          [messageId]: (s.reactions[messageId] ?? []).filter(
            (r) => !(r.emoji === emoji && r.reactor === "user")
          ),
        },
      }));
    } else {
      const optimistic: Reaction = {
        reaction_id: `pending-${Date.now()}`,
        message_id: messageId,
        emoji,
        reactor: "user",
        created_at: new Date().toISOString(),
      };
      setState((s) => ({
        ...s,
        reactions: {
          ...s.reactions,
          [messageId]: [...(s.reactions[messageId] ?? []), optimistic],
        },
      }));
    }

    try {
      await api.addReaction(messageId, emoji, "user");
    } catch (e) {
      if (String(e) !== "removed") {
        setError(String(e));
      }
    }
    // Reload reactions for the full visible message set — the backend
    // propagates a reaction to every message in the same "reaction unit"
    // (target + preceding burst + user turn), so we can't assume only
    // `messageId` changed.
    try {
      const ids = state.messages.map((m) => m.message_id);
      const fresh = await api.getReactions(ids);
      const grouped: Record<string, Reaction[]> = {};
      for (const r of fresh) {
        if (!grouped[r.message_id]) grouped[r.message_id] = [];
        grouped[r.message_id].push(r);
      }
      setState((s) => ({
        ...s,
        reactions: { ...s.reactions, ...grouped },
      }));
    } catch {
      // keep optimistic state
    }
  }, [state.reactions, state.messages, setError]);

  return {
    ...state,
    loadWorlds,
    selectWorld,
    selectCharacter,
    selectGroupChat,
    createGroupChat,
    deleteGroupChat,
    sendGroupMessage,
    promptGroupCharacter,
    createWorld,
    updateWorld,
    deleteWorld,
    updateCharacter,
    createCharacter,
    deleteCharacter,
    clearChatHistory,
    clearGroupChatHistory,
    archiveCharacter,
    unarchiveCharacter,
    sendMessage,
    setAutoRespond,
    setNotifyOnMessage,
    setChatFontSize,
    loadEarlierMessages,
    promptCharacter,
    generateNarrative,
    generateIllustration,
    adjustMessage,
    editMessageContent,
    deleteMessage,
    deleteIllustration,
    regenerateIllustration,
    adjustIllustration,
    generateVideo,
    resetToMessage,
    setApiKey,
    setModelConfig,
    setBudgetMode,
    updateWorldState,
    toggleReaction,
    refreshPortrait,
    refreshWorldImage,
    updateUserProfile,
    loadUserProfile,
    selectUserProfile,
    clearChatError,
    setError,
    refreshProactiveUnreadCounts,
    runProactivePingSweep,
    generateDream,
    refreshVisualDescription,
    backfillVisualDescriptions,
    refreshCharacterInventory,
    refreshGroupInventories,
    applyCharacterInventoryEdit,
  };
}
