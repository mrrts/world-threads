import { useCallback, useEffect, useState } from "react";
import { api, type World, type Character, type Message, type ModelConfig, type Reaction, type PortraitInfo, type UserProfile, type WorldImageInfo, type GroupChat } from "@/lib/tauri";

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
}

const PAGE_SIZE = 20;

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
      const [worlds, modelConfig, apiKey, budgetMode, autoRespondSetting] = await Promise.all([
        api.listWorlds(),
        api.getModelConfig(),
        api.migrateApiKey(),
        api.getBudgetMode(),
        api.getSetting("auto_respond").catch(() => null),
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
            const page = await api.getGroupMessages(gc.group_chat_id);
            messages = page.messages;
            totalMessages = page.total;
            reactions = await loadReactions(messages);
          }
        } else if (lastChat?.startsWith("char:")) {
          const charId = lastChat.slice(5);
          const ch = characters.find((c) => c.character_id === charId);
          if (ch) {
            activeCharacter = ch;
            const page = await api.getMessages(ch.character_id);
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
          const page = await api.getMessages(activeCharacter.character_id);
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
        loadingOlder: false,
    loadingChat: false,
        loading: false,
        sending: null,
        generatingNarrative: null,
        generatingIllustration: null,
        generatingVideo: null,
    sendingCharacterId: null,
        videoFiles: {},
    aspectRatios: {},
        error: null,
        editingUserProfile: false,
        chatError: null,
        lastFailedContent: null,
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
          const page = await api.getGroupMessages(gc.group_chat_id);
          messages = page.messages;
          totalMessages = page.total;
          reactions = await loadReactions(messages);
        }
      } else if (lastChat?.startsWith("char:")) {
        const charId = lastChat.slice(5);
        const ch = characters.find((c) => c.character_id === charId);
        if (ch) {
          activeCharacter = ch;
          const page = await api.getMessages(ch.character_id);
          messages = page.messages;
          totalMessages = page.total;
          reactions = await loadReactions(messages);
        }
      }

      // Fallback to first character if no last chat found
      if (!activeCharacter && !activeGroupChat && characters.length > 0) {
        activeCharacter = characters[0];
        const page = await api.getMessages(activeCharacter.character_id);
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
      const page = await api.getMessages(character.character_id);
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
      const page = await api.getGroupMessages(groupChat.group_chat_id);
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

  const sendGroupMessage = useCallback(async (content: string) => {
    if (!state.activeGroupChat || !state.apiKey) return;

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

      // Then prompt each character sequentially
      const charIds: string[] = Array.isArray(state.activeGroupChat.character_ids) ? state.activeGroupChat.character_ids : [];
      for (const cid of charIds) {
        setState((s) => ({ ...s, sending: state.activeGroupChat!.group_chat_id, sendingCharacterId: cid }));
        const msg = await api.promptGroupCharacter(state.apiKey, state.activeGroupChat!.group_chat_id, cid);
        setState((s) => ({
          ...s,
          messages: [...s.messages, msg],
          totalMessages: s.totalMessages + 1,
          sending: null,
          sendingCharacterId: null,
        }));
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
  }, [state.activeGroupChat, state.apiKey, state.autoRespond]);

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
        const msg = await api.promptGroupCharacter(state.apiKey, state.activeGroupChat!.group_chat_id, cid, addressTo);
        setState((s) => ({
          ...s,
          messages: [...s.messages, msg],
          totalMessages: s.totalMessages + 1,
          sendingCharacterId: null,
        }));
      }
      setState((s) => ({ ...s, sending: null }));
    } catch (e) {
      setState((s) => ({
        ...s,
        sending: null,
        chatError: String(e),
      }));
    }
  }, [state.activeGroupChat, state.apiKey, state.autoRespond]);

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
      const page = activeCharacter ? await api.getMessages(activeCharacter.character_id) : { messages: [], total: 0 };
      setState((s) => ({ ...s, characters, activeCharacter, messages: page.messages, totalMessages: page.total }));
    } catch (e) {
      setError(String(e));
    }
  }, [state.activeWorld, setError]);

  const clearGroupChatHistory = useCallback(async (groupChatId: string) => {
    try {
      await api.clearGroupChatHistory(groupChatId);
      setState((s) => ({
        ...s,
        messages: [],
        totalMessages: 0,
        reactions: {},
        chatError: null,
        lastFailedContent: null,
      }));
    } catch (e) {
      setError(String(e));
    }
  }, [setError]);

  const clearChatHistory = useCallback(async (characterId: string) => {
    try {
      await api.clearChatHistory(characterId);
      setState((s) => ({
        ...s,
        messages: [],
        totalMessages: 0,
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
        const page = await api.getMessages(activeCharacter.character_id);
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

  const sendMessage = useCallback(async (content: string) => {
    if (!state.activeCharacter) return;

    // When auto-respond is off, just save the user message without triggering AI
    if (!state.autoRespond) {
      const optimisticMsg: Message = {
        message_id: `pending-${Date.now()}`,
        thread_id: "",
        role: "user",
        content,
        tokens_estimate: 0,
        created_at: new Date().toISOString(),
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
        const merged = { ...s.reactions };
        for (const r of result.ai_reactions) {
          if (!merged[r.message_id]) merged[r.message_id] = [];
          merged[r.message_id].push(r);
        }
        return {
          ...s,
          messages: [
            ...s.messages.filter((m) => m.message_id !== optimisticMsg.message_id),
            result.user_message,
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
    } catch (e) {
      setState((s) => ({
        ...s,
        sending: false,
        chatError: String(e),
        lastFailedContent: content,
        messages: s.messages.filter((m) => m.message_id !== optimisticMsg.message_id),
      }));
    }
  }, [state.activeCharacter, state.apiKey, state.activeWorld, state.autoRespond]);

  const promptCharacter = useCallback(async () => {
    if (!state.activeCharacter || !state.apiKey) return;

    setState((s) => ({ ...s, sending: state.activeCharacter!.character_id, chatError: null }));

    try {
      const result = await api.promptCharacter(state.apiKey, state.activeCharacter.character_id);
      setState((s) => {
        const merged = { ...s.reactions };
        for (const r of result.ai_reactions) {
          if (!merged[r.message_id]) merged[r.message_id] = [];
          merged[r.message_id].push(r);
        }
        return {
          ...s,
          messages: [...s.messages, result.assistant_message],
          totalMessages: s.totalMessages + 1,
          reactions: merged,
          sending: null,
        };
      });
    } catch (e) {
      setState((s) => ({
        ...s,
        sending: false,
        chatError: String(e),
      }));
    }
  }, [state.activeCharacter, state.apiKey]);

  const generateNarrative = useCallback(async (customInstructions?: string) => {
    if (!state.activeCharacter || !state.apiKey) return;

    setState((s) => ({ ...s, sending: state.activeCharacter!.character_id, generatingNarrative: state.activeCharacter!.character_id, chatError: null }));

    try {
      const result = await api.generateNarrative(state.apiKey, state.activeCharacter.character_id, customInstructions);
      setState((s) => ({
        ...s,
        messages: [...s.messages, result.narrative_message],
        totalMessages: s.totalMessages + 1,
        sending: false,
        generatingNarrative: false,
      }));
    } catch (e) {
      setState((s) => ({
        ...s,
        sending: false,
        generatingNarrative: false,
        chatError: String(e),
      }));
    }
  }, [state.activeCharacter, state.apiKey]);

  const generateIllustration = useCallback(async (qualityTier?: string, customInstructions?: string, previousIllustrationId?: string, includeSceneSummary?: boolean) => {
    if (!state.activeCharacter || !state.apiKey) return;

    setState((s) => ({ ...s, sending: state.activeCharacter!.character_id, generatingIllustration: state.activeCharacter!.character_id, chatError: null }));

    try {
      const result = await api.generateIllustration(state.apiKey, state.activeCharacter.character_id, qualityTier, customInstructions, previousIllustrationId, includeSceneSummary);
      setState((s) => ({
        ...s,
        messages: [...s.messages, result.illustration_message],
        totalMessages: s.totalMessages + 1,
        sending: false,
        generatingIllustration: false,
      }));
    } catch (e) {
      setState((s) => ({
        ...s,
        sending: false,
        generatingIllustration: false,
        chatError: String(e),
      }));
    }
  }, [state.activeCharacter, state.apiKey]);

  const generateGroupNarrative = useCallback(async (customInstructions?: string) => {
    if (!state.activeGroupChat || !state.apiKey) return;

    setState((s) => ({ ...s, sending: state.activeGroupChat!.group_chat_id, generatingNarrative: state.activeGroupChat!.group_chat_id, chatError: null }));

    try {
      const result = await api.generateGroupNarrative(state.apiKey, state.activeGroupChat.group_chat_id, customInstructions);
      setState((s) => ({
        ...s,
        messages: [...s.messages, result.narrative_message],
        totalMessages: s.totalMessages + 1,
        sending: null,
        generatingNarrative: null,
      }));
    } catch (e) {
      setState((s) => ({
        ...s,
        sending: null,
        generatingNarrative: null,
        chatError: String(e),
      }));
    }
  }, [state.activeGroupChat, state.apiKey]);

  const generateGroupIllustration = useCallback(async (qualityTier?: string, customInstructions?: string, previousIllustrationId?: string, includeSceneSummary?: boolean) => {
    if (!state.activeGroupChat || !state.apiKey) return;

    setState((s) => ({ ...s, sending: state.activeGroupChat!.group_chat_id, generatingIllustration: state.activeGroupChat!.group_chat_id, chatError: null }));

    try {
      const result = await api.generateGroupIllustration(state.apiKey, state.activeGroupChat.group_chat_id, qualityTier, customInstructions, previousIllustrationId, includeSceneSummary);
      setState((s) => ({
        ...s,
        messages: [...s.messages, result.illustration_message],
        totalMessages: s.totalMessages + 1,
        sending: null,
        generatingIllustration: null,
      }));
    } catch (e) {
      setState((s) => ({
        ...s,
        sending: null,
        generatingIllustration: null,
        chatError: String(e),
      }));
    }
  }, [state.activeGroupChat, state.apiKey]);

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
    } catch (e) {
      setState((s) => ({
        ...s,
        adjustingMessageId: null,
        chatError: String(e),
      }));
    }
  }, [state.apiKey, state.activeGroupChat, state.activeCharacter]);

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
    if (!state.activeCharacter || !state.apiKey) return;

    setState((s) => ({
      ...s,
      sending: state.activeCharacter!.character_id,
      generatingIllustration: state.activeCharacter!.character_id,
      chatError: null,
      messages: s.messages.filter((m) => m.message_id !== messageId),
      totalMessages: s.totalMessages - 1,
    }));

    try {
      const result = await api.regenerateIllustration(state.apiKey, state.activeCharacter.character_id, messageId);
      setState((s) => ({
        ...s,
        messages: [...s.messages, result.illustration_message],
        totalMessages: s.totalMessages + 1,
        sending: false,
        generatingIllustration: false,
      }));
    } catch (e) {
      setState((s) => ({
        ...s,
        sending: false,
        generatingIllustration: false,
        chatError: String(e),
      }));
    }
  }, [state.activeCharacter, state.apiKey]);

  const regenerateGroupIllustration = useCallback(async (messageId: string) => {
    if (!state.activeGroupChat || !state.apiKey) return;
    const gcId = state.activeGroupChat.group_chat_id;

    setState((s) => ({
      ...s,
      sending: gcId,
      generatingIllustration: gcId,
      chatError: null,
      messages: s.messages.filter((m) => m.message_id !== messageId),
      totalMessages: s.totalMessages - 1,
    }));

    try {
      await api.deleteIllustration(messageId);
      const result = await api.generateGroupIllustration(state.apiKey, gcId);
      setState((s) => ({
        ...s,
        messages: [...s.messages, result.illustration_message],
        totalMessages: s.totalMessages + 1,
        sending: false,
        generatingIllustration: false,
      }));
    } catch (e) {
      setState((s) => ({
        ...s,
        sending: false,
        generatingIllustration: false,
        chatError: String(e),
      }));
    }
  }, [state.activeGroupChat, state.apiKey]);

  const adjustIllustration = useCallback(async (messageId: string, instructions: string) => {
    if (!state.activeCharacter || !state.apiKey) return;

    setState((s) => ({
      ...s,
      sending: state.activeCharacter!.character_id,
      generatingIllustration: state.activeCharacter!.character_id,
      chatError: null,
      messages: s.messages.filter((m) => m.message_id !== messageId),
      totalMessages: s.totalMessages - 1,
    }));

    try {
      const result = await api.adjustIllustration(state.apiKey, state.activeCharacter.character_id, messageId, instructions);
      setState((s) => ({
        ...s,
        messages: [...s.messages, result.illustration_message],
        totalMessages: s.totalMessages + 1,
        sending: false,
        generatingIllustration: false,
      }));
    } catch (e) {
      setState((s) => ({
        ...s,
        sending: false,
        generatingIllustration: false,
        chatError: String(e),
      }));
    }
  }, [state.activeCharacter, state.apiKey]);

  const adjustGroupIllustration = useCallback(async (messageId: string, instructions: string) => {
    if (!state.activeGroupChat || !state.apiKey) return;
    const gcId = state.activeGroupChat.group_chat_id;

    setState((s) => ({
      ...s,
      sending: gcId,
      generatingIllustration: gcId,
      chatError: null,
      messages: s.messages.filter((m) => m.message_id !== messageId),
      totalMessages: s.totalMessages - 1,
    }));

    try {
      const result = await api.generateGroupIllustration(state.apiKey, gcId, undefined, instructions, messageId);
      // Clean up old illustration after new one is generated
      await api.deleteIllustration(messageId).catch(() => {});
      setState((s) => ({
        ...s,
        messages: [...s.messages, result.illustration_message],
        totalMessages: s.totalMessages + 1,
        sending: false,
        generatingIllustration: false,
      }));
    } catch (e) {
      setState((s) => ({
        ...s,
        sending: false,
        generatingIllustration: false,
        chatError: String(e),
      }));
    }
  }, [state.activeGroupChat, state.apiKey]);

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
            const msg = await api.promptGroupCharacter(state.apiKey, state.activeGroupChat.group_chat_id, cid);
            setState((s) => ({
              ...s,
              messages: [...s.messages, msg],
              totalMessages: s.totalMessages + 1,
              sendingCharacterId: null,
            }));
          }
        } catch { /* non-fatal */ }
        setState((s) => ({ ...s, sending: null }));
      }
    } catch (e) {
      // Reload messages from DB to get correct state after partial failure
      if (state.activeCharacter) {
        const page = await api.getMessages(state.activeCharacter.character_id);
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
        const page = await api.getGroupMessages(state.activeGroupChat.group_chat_id);
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

  // loadOlderMessages is no longer needed — all messages are loaded at once

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
    // Reload actual reactions from DB to get correct state
    try {
      const fresh = await api.getReactions([messageId]);
      setState((s) => ({
        ...s,
        reactions: { ...s.reactions, [messageId]: fresh },
      }));
    } catch {
      // keep optimistic state
    }
  }, [state.reactions, setError]);

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
    promptCharacter,
    generateNarrative,
    generateGroupNarrative,
    generateIllustration,
    generateGroupIllustration,
    adjustMessage,
    deleteIllustration,
    regenerateIllustration,
    adjustIllustration,
    regenerateGroupIllustration,
    adjustGroupIllustration,
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
  };
}
