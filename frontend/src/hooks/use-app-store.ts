import { useCallback, useEffect, useState } from "react";
import { api, type World, type Character, type Message, type ModelConfig, type Reaction, type PortraitInfo, type UserProfile, type WorldImageInfo } from "@/lib/tauri";

export interface AppState {
  worlds: World[];
  activeWorld: World | null;
  characters: Character[];
  archivedCharacters: Character[];
  activeCharacter: Character | null;
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
  sending: boolean;
  generatingNarrative: boolean;
  totalMessages: number;
  loadingOlder: boolean;
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
    autoRespond: true,
    loading: true,
    sending: false,
    generatingNarrative: false,
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
      const [worlds, modelConfig, apiKey, budgetMode] = await Promise.all([
        api.listWorlds(),
        api.getModelConfig(),
        api.migrateApiKey(),
        api.getBudgetMode(),
      ]);

      let activeWorld: World | null = null;
      let characters: Character[] = [];
      let archivedCharacters: Character[] = [];
      let activeCharacter: Character | null = null;
      let messages: Message[] = [];
      let totalMessages = 0;
      let reactions: Record<string, Reaction[]> = {};
      let activePortraits: Record<string, PortraitInfo> = {};
      let activeWorldImage: WorldImageInfo | null = null;
      let userProfile: UserProfile | null = null;

      if (worlds.length > 0) {
        activeWorld = worlds[0];
        const wid = activeWorld.world_id;
        const [chars, archived, wImage, uProfile] = await Promise.all([
          api.listCharacters(wid),
          api.listArchivedCharacters(wid),
          api.getActiveWorldImage(wid),
          api.getUserProfile(wid),
        ]);
        characters = chars;
        archivedCharacters = archived;
        activeWorldImage = wImage;
        userProfile = uProfile;
        activePortraits = await loadActivePortraits([...characters, ...archivedCharacters]);
        if (characters.length > 0) {
          activeCharacter = characters[0];
          const page = await api.getMessages(activeCharacter.character_id, PAGE_SIZE);
          messages = page.messages;
          totalMessages = page.total;
          reactions = await loadReactions(messages);
        }
      }

      setState({
        worlds,
        activeWorld,
        characters,
        archivedCharacters,
        activeCharacter,
        messages,
        totalMessages,
        reactions,
        activePortraits,
        activeWorldImage,
        userProfile,
        modelConfig,
        apiKey: apiKey ?? "",
        budgetMode,
        autoRespond: true,
        loadingOlder: false,
        loading: false,
        sending: false,
        generatingNarrative: false,
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
      messages: [],
      totalMessages: 0,
      reactions: {},
      activePortraits: {},
      activeWorldImage: null,
      userProfile: null,
    }));
    try {
      const [characters, archivedCharacters, activeWorldImage, userProfile] = await Promise.all([
        api.listCharacters(world.world_id),
        api.listArchivedCharacters(world.world_id),
        api.getActiveWorldImage(world.world_id),
        api.getUserProfile(world.world_id),
      ]);
      const activePortraits = await loadActivePortraits([...characters, ...archivedCharacters]);
      let activeCharacter: Character | null = null;
      let messages: Message[] = [];
      let totalMessages = 0;
      let reactions: Record<string, Reaction[]> = {};
      if (characters.length > 0) {
        activeCharacter = characters[0];
        const page = await api.getMessages(activeCharacter.character_id, PAGE_SIZE);
        messages = page.messages;
        totalMessages = page.total;
        reactions = await loadReactions(messages);
      }
      setState((s) => {
        if (s.activeWorld?.world_id !== world.world_id) return s;
        return { ...s, characters, archivedCharacters, activeCharacter, messages, totalMessages, reactions, activePortraits, activeWorldImage, userProfile };
      });
    } catch (e) {
      setError(String(e));
    }
  }, [setError, loadReactions, loadActivePortraits]);

  const selectCharacter = useCallback(async (character: Character) => {
    setState((s) => ({ ...s, activeCharacter: character, messages: [], totalMessages: 0, reactions: {}, editingUserProfile: false, chatError: null, lastFailedContent: null }));
    try {
      const page = await api.getMessages(character.character_id, PAGE_SIZE);
      const reactions = await loadReactions(page.messages);
      setState((s) => {
        if (s.activeCharacter?.character_id !== character.character_id) return s;
        return { ...s, messages: page.messages, totalMessages: page.total, reactions };
      });
    } catch (e) {
      setError(String(e));
    }
  }, [setError, loadReactions]);

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
      const page = activeCharacter ? await api.getMessages(activeCharacter.character_id, PAGE_SIZE) : { messages: [], total: 0 };
      setState((s) => ({ ...s, characters, activeCharacter, messages: page.messages, totalMessages: page.total }));
    } catch (e) {
      setError(String(e));
    }
  }, [state.activeWorld, setError]);

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
        const page = await api.getMessages(activeCharacter.character_id, PAGE_SIZE);
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
      sending: true,
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
          sending: false,
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

    setState((s) => ({ ...s, sending: true, chatError: null }));

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
          sending: false,
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

  const generateNarrative = useCallback(async () => {
    if (!state.activeCharacter || !state.apiKey) return;

    setState((s) => ({ ...s, sending: true, generatingNarrative: true, chatError: null }));

    try {
      const result = await api.generateNarrative(state.apiKey, state.activeCharacter.character_id);
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

  const resetToMessage = useCallback(async (messageId: string) => {
    if (!state.activeCharacter || !state.apiKey) return;

    const anchorMsg = state.messages.find((m) => m.message_id === messageId);
    const isUserMsg = anchorMsg?.role === "user";

    setState((s) => ({
      ...s,
      messages: s.messages.filter((m) => {
        if (m.message_id === messageId) return true;
        return m.created_at < anchorMsg!.created_at ||
          (m.created_at === anchorMsg!.created_at && m.message_id === messageId);
      }),
      sending: isUserMsg,
      chatError: null,
    }));

    try {
      const result = await api.resetToMessage(state.apiKey, state.activeCharacter.character_id, messageId);

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
            sending: false,
          };
        });
      } else {
        setState((s) => ({
          ...s,
          totalMessages: s.totalMessages - result.deleted_count,
          sending: false,
        }));
      }
    } catch (e) {
      // Reload messages from DB to get correct state after partial failure
      if (state.activeCharacter) {
        const page = await api.getMessages(state.activeCharacter.character_id, PAGE_SIZE);
        const reactions = await loadReactions(page.messages);
        setState((s) => ({
          ...s,
          messages: page.messages,
          totalMessages: page.total,
          reactions,
          sending: false,
          chatError: String(e),
        }));
      }
    }
  }, [state.activeCharacter, state.apiKey, state.messages, loadReactions]);

  const clearChatError = useCallback(() => {
    setState((s) => ({ ...s, chatError: null, lastFailedContent: null }));
  }, []);

  const loadOlderMessages = useCallback(async () => {
    if (!state.activeCharacter || state.loadingOlder) return;
    if (state.messages.length >= state.totalMessages) return;
    setState((s) => ({ ...s, loadingOlder: true }));
    try {
      const offset = state.messages.length;
      const page = await api.getMessages(state.activeCharacter.character_id, PAGE_SIZE, offset);
      const olderReactions = await loadReactions(page.messages);
      setState((s) => ({
        ...s,
        messages: [...page.messages, ...s.messages],
        reactions: { ...olderReactions, ...s.reactions },
        loadingOlder: false,
      }));
    } catch (e) {
      setError(String(e));
      setState((s) => ({ ...s, loadingOlder: false }));
    }
  }, [state.activeCharacter, state.messages.length, state.totalMessages, state.loadingOlder, setError, loadReactions]);

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

  const hasMoreMessages = state.messages.length < state.totalMessages;

  return {
    ...state,
    hasMoreMessages,
    loadWorlds,
    selectWorld,
    selectCharacter,
    createWorld,
    updateWorld,
    deleteWorld,
    updateCharacter,
    createCharacter,
    deleteCharacter,
    clearChatHistory,
    archiveCharacter,
    unarchiveCharacter,
    sendMessage,
    setAutoRespond,
    promptCharacter,
    generateNarrative,
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
    loadOlderMessages,
    setError,
  };
}
