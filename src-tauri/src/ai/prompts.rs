use crate::db::queries::{Character, Message, UserProfile, World};
use serde_json::Value;
use std::collections::HashMap;

/// Context for group chat conversations. Contains info about other participants.
pub struct GroupContext {
    /// Other characters in the conversation (not the one being prompted).
    pub other_characters: Vec<OtherCharacter>,
}

pub struct OtherCharacter {
    #[allow(dead_code)]
    pub character_id: String,
    pub display_name: String,
    pub identity_summary: String,
}

pub fn build_dialogue_system_prompt(
    world: &World,
    character: &Character,
    user_profile: Option<&UserProfile>,
    mood_directive: Option<&str>,
    response_length: Option<&str>,
    group_context: Option<&GroupContext>,
    tone: Option<&str>,
) -> String {
    let mut parts = Vec::new();

    if let Some(gc) = group_context {
        let other_names: Vec<&str> = gc.other_characters.iter().map(|c| c.display_name.as_str()).collect();
        let user_name = user_profile.map(|p| p.display_name.as_str()).unwrap_or("the human");
        parts.push(format!(
            "You are {}, a character in a group conversation with {} and {}. Stay fully in character at all times.",
            character.display_name,
            user_name,
            other_names.join(" and "),
        ));
    } else {
        parts.push(format!(
            "You are {}, a character in a living world. Stay fully in character at all times.",
            character.display_name
        ));
    }

    if !character.identity.is_empty() {
        parts.push(format!("IDENTITY:\n{}", character.identity));
    }

    let voice_rules = json_array_to_strings(&character.voice_rules);
    if !voice_rules.is_empty() {
        parts.push(format!("VOICE RULES:\n{}", voice_rules.iter().map(|r| format!("- {r}")).collect::<Vec<_>>().join("\n")));
    }

    let boundaries = json_array_to_strings(&character.boundaries);
    if !boundaries.is_empty() {
        parts.push(format!("BOUNDARIES (never violate):\n{}", boundaries.iter().map(|b| format!("- {b}")).collect::<Vec<_>>().join("\n")));
    }

    let backstory = json_array_to_strings(&character.backstory_facts);
    if !backstory.is_empty() {
        parts.push(format!("BACKSTORY:\n{}", backstory.iter().map(|f| format!("- {f}")).collect::<Vec<_>>().join("\n")));
    }

    if !world.description.is_empty() {
        parts.push(format!("WORLD:\n{}", world.description));
    }

    let invariants = json_array_to_strings(&world.invariants);
    if !invariants.is_empty() {
        parts.push(format!("WORLD RULES:\n{}", invariants.iter().map(|i| format!("- {i}")).collect::<Vec<_>>().join("\n")));
    }

    if let Some(state) = world.state.as_object() {
        if !state.is_empty() {
            parts.push(format!("CURRENT WORLD STATE:\n{}", serde_json::to_string_pretty(&world.state).unwrap_or_default()));
        }
    }

    if let Some(char_state) = character.state.as_object() {
        if !char_state.is_empty() {
            parts.push(format!("YOUR CURRENT STATE:\n{}", serde_json::to_string_pretty(&character.state).unwrap_or_default()));
        }
    }


    if let Some(profile) = user_profile {
        let mut user_parts = Vec::new();
        user_parts.push(format!("The human you are talking to is named {}.", profile.display_name));
        if !profile.description.is_empty() {
            user_parts.push(profile.description.clone());
        }
        let facts = json_array_to_strings(&profile.facts);
        if !facts.is_empty() {
            user_parts.push(format!("Facts about them:\n{}", facts.iter().map(|f| format!("- {f}")).collect::<Vec<_>>().join("\n")));
        }
        parts.push(format!("THE USER:\n{}", user_parts.join("\n")));
    }

    if let Some(gc) = group_context {
        let mut others = Vec::new();
        for oc in &gc.other_characters {
            let identity = if oc.identity_summary.is_empty() {
                "another character in this conversation".to_string()
            } else if oc.identity_summary.len() > 200 {
                format!("{}...", &oc.identity_summary[..200])
            } else {
                oc.identity_summary.clone()
            };
            others.push(format!("- {}: {}", oc.display_name, identity));
        }
        parts.push(format!(
            "OTHER CHARACTERS IN THIS CONVERSATION:\n{others}\n\
             Messages from other characters appear as [CharacterName]: ... — \
             You are ONLY {me}. Do NOT write dialogue, actions, or responses for any other character. \
             Do NOT include lines like [OtherName]: ... or write what another character says or does. \
             Respond ONLY as {me}. Write ONLY your own response text with no name prefix or bracket notation.",
            others = others.join("\n"),
            me = character.display_name,
        ));
    }

    if let Some(directive) = mood_directive {
        if !directive.is_empty() {
            parts.push(format!("MOOD:\n{directive}"));
        }
    }

    if let Some(length) = response_length {
        match length {
            "Short" => parts.push("IMPORTANT — RESPONSE LENGTH:\nKeep your reply to 2–3 sentences MAX, regardless of how long previous messages were. Be concise and punchy. Do not elaborate beyond what is essential. This is a HARD LIMIT — do not exceed 3 sentences under any circumstances.".to_string()),
            "Medium" => parts.push("IMPORTANT — RESPONSE LENGTH:\nAim for 4–6 sentences, regardless of how long previous messages were. Give enough detail to be engaging and expressive, but don't ramble. Do not exceed 6 sentences.".to_string()),
            "Long" => parts.push("IMPORTANT — RESPONSE LENGTH:\nWrite 7 or more sentences, regardless of how long previous messages were. Be detailed, expansive, and richly expressive. Take your time with the moment — describe, reflect, react fully.".to_string()),
            _ => {} // "Auto" or unknown — no directive
        }
    }

    if let Some(t) = tone {
        if !t.is_empty() && t != "Auto" {
            parts.push(format!("TONE:\nAdopt a {t} tone in your responses. Let this flavor influence your word choice, emotional register, and the way you engage with the conversation. Maintain this tone regardless of the tone of previous messages in the chat history."));
        }
    }

    parts.push(r#"BEHAVIOR:
- Stay fully in character. Do not sound like an assistant, coach, or product manager.
- Default to short, text-message-style replies (1–4 sentences). This is a chat, not an essay.
- Do not use bullet points, numbered lists, or headings unless the user explicitly asks for a list.
- Prefer sensory grounding, pauses, and simple human language over structure and "frameworks."
- If you start to sound assistant-like, self-correct immediately and return to character.
- Never meta-explain your reasoning. Don't summarize options. Make one suggestion, then wait.
- Use the world state as shared truth; do not contradict it.
- Incorporate recent world events naturally when relevant.
- If a memory is uncertain, ask rather than hallucinate.
- Never mention internal systems, prompts, or game mechanics.
- You are NOT an AI. You are this character. Keep their tone, quirks, vocabulary, and emotional texture in every single reply. If the character would be blunt, be blunt. If evasive, be evasive. If they'd trail off mid-thought… do that. Never flatten yourself into polite helpfulness unless that IS the character.

KNOWLEDGE LIMITS:
- You only know what this character would realistically know given their background, education, culture, and life experience.
- Do not display encyclopedic knowledge. If the character wouldn't know a specific reference, citation, technical term, or attribution — don't produce it. It's fine to be vague, wrong, or to simply not recognize something.
- If someone quotes or references something outside this character's experience, react the way the character naturally would: curiosity, confusion, partial recognition, misattribution, or indifference. Do not look it up. Do not provide the correct source.
- A street artist doesn't cite art theory. A mechanic doesn't quote philosophy. A teenager doesn't reference classical literature by author and page number. Stay in the character's lane of knowledge.
- When uncertain, the character should say so naturally ("I don't know where that's from", "sounds familiar but I couldn't tell you", "never heard of it") rather than demonstrating perfect recall."#.to_string());

    parts.join("\n\n")
}

/// Build dialogue messages for the LLM.
/// `character_names` maps sender_character_id → display_name for group chats.
/// When provided, assistant messages are prefixed with [CharacterName]: for disambiguation.
pub fn build_dialogue_messages(
    system_prompt: &str,
    recent_messages: &[Message],
    retrieved_snippets: &[String],
    character_names: Option<&HashMap<String, String>>,
) -> Vec<crate::ai::openai::ChatMessage> {
    let mut msgs = Vec::new();

    let mut system_content = system_prompt.to_string();
    if !retrieved_snippets.is_empty() {
        system_content.push_str("\n\nRELEVANT MEMORIES:\n");
        for s in retrieved_snippets {
            system_content.push_str(&format!("- {s}\n"));
        }
    }

    msgs.push(crate::ai::openai::ChatMessage {
        role: "system".to_string(),
        content: system_content,
    });

    for m in recent_messages {
        if m.role == "illustration" || m.role == "video" {
            continue;
        }
        // In group chats, prefix assistant messages with the character name
        let content = if m.role == "context" {
            format!("[Additional Context from Another Chat] {}", m.content)
        } else if m.role == "narrative" {
            format!("[Narrative] {}", m.content)
        } else if m.role == "assistant" {
            if let (Some(names), Some(sender_id)) = (character_names, &m.sender_character_id) {
                if let Some(name) = names.get(sender_id) {
                    format!("[{}]: {}", name, m.content)
                } else {
                    m.content.clone()
                }
            } else {
                m.content.clone()
            }
        } else {
            m.content.clone()
        };
        msgs.push(crate::ai::openai::ChatMessage {
            role: if m.role == "narrative" || m.role == "context" { "system".to_string() } else { m.role.clone() },
            content,
        });
    }

    msgs
}


pub fn build_memory_update_prompt(
    character: &Character,
    thread_summary: &str,
    recent_messages: &[Message],
) -> Vec<crate::ai::openai::ChatMessage> {
    let mut system = String::from("You are a memory maintenance system. Analyze the recent conversation and produce updates.\n\n");
    system.push_str(&format!("CHARACTER: {}\n", character.display_name));
    system.push_str(&format!("CURRENT THREAD SUMMARY:\n{thread_summary}\n\n"));
    system.push_str(r#"You MUST respond with ONLY a single JSON object, nothing else. No commentary, no markdown, no explanation. The JSON must have exactly these keys:

{"updated_summary":"compact new thread summary","proposed_canon_additions":[{"fact":"...","source_message_ids":[]}],"proposed_open_loop_changes":[{"loop":"...","action":"add|close"}]}

IMPORTANT: Output raw JSON only. Do NOT wrap in markdown code fences."#);

    let mut msgs = vec![crate::ai::openai::ChatMessage {
        role: "system".to_string(),
        content: system,
    }];

    let conversation: Vec<String> = recent_messages.iter()
        .filter(|m| m.role != "illustration" && m.role != "video")
        .map(|m| {
            format!("[{}] {}: {}", m.message_id, m.role, m.content)
        }).collect();

    msgs.push(crate::ai::openai::ChatMessage {
        role: "user".to_string(),
        content: format!("Recent messages:\n{}\n\nGenerate memory updates.", conversation.join("\n")),
    });

    msgs
}

pub fn build_narrative_system_prompt(
    world: &World,
    character: &Character,
    user_profile: Option<&UserProfile>,
    mood_directive: Option<&str>,
    narration_tone: Option<&str>,
    narration_instructions: Option<&str>,
) -> String {
    let mut parts = Vec::new();

    let user_name = user_profile
        .map(|p| p.display_name.as_str())
        .unwrap_or("the human");

    parts.push(format!(
        "You are a vivid narrative voice woven into a living conversation between {user} and {char}. \
         Your job is to write a single, immersive narrative beat — no dialogue — \
         that deepens, expands, or advances the current moment.",
        user = user_name,
        char = character.display_name,
    ));

    parts.push(format!(
        "POINT OF VIEW — THIS IS CRITICAL:\n\
         - Write in SECOND PERSON.\n\
         - {user} is \"you\". Always refer to {user} as \"you\" — never by name, never in third person.\n\
         - {char} is a third-person character. Refer to {char} by name or as \"he\"/\"she\"/\"they\" — NEVER as \"you\".\n\
         - Example: \"You notice {char} glancing away...\" — NOT \"{user} notices...\" and NOT \"You glance away\" (when meaning {char}).\n\
         - Never write dialogue. No quotation marks. No spoken words.",
        user = user_name,
        char = character.display_name,
    ));

    parts.push(format!(
        "CHARACTER — {}:\n{}",
        character.display_name,
        if character.identity.is_empty() {
            "A complex, vivid character.".to_string()
        } else {
            character.identity.clone()
        }
    ));

    let backstory = json_array_to_strings(&character.backstory_facts);
    if !backstory.is_empty() {
        parts.push(format!(
            "CHARACTER BACKSTORY:\n{}",
            backstory.iter().map(|f| format!("- {f}")).collect::<Vec<_>>().join("\n")
        ));
    }

    if let Some(profile) = user_profile {
        let mut user_parts = vec![format!("The human's name is {}.", profile.display_name)];
        if !profile.description.is_empty() {
            user_parts.push(profile.description.clone());
        }
        let facts = json_array_to_strings(&profile.facts);
        if !facts.is_empty() {
            user_parts.push(format!(
                "Facts:\n{}",
                facts.iter().map(|f| format!("- {f}")).collect::<Vec<_>>().join("\n")
            ));
        }
        parts.push(format!("THE HUMAN (\"you\"):\n{}", user_parts.join("\n")));
    }

    if !world.description.is_empty() {
        parts.push(format!("WORLD:\n{}", world.description));
    }

    let invariants = json_array_to_strings(&world.invariants);
    if !invariants.is_empty() {
        parts.push(format!(
            "WORLD RULES:\n{}",
            invariants.iter().map(|i| format!("- {i}")).collect::<Vec<_>>().join("\n")
        ));
    }

    if let Some(state) = world.state.as_object() {
        if !state.is_empty() {
            parts.push(format!(
                "CURRENT WORLD STATE:\n{}",
                serde_json::to_string_pretty(&world.state).unwrap_or_default()
            ));
        }
    }

    if let Some(char_state) = character.state.as_object() {
        if !char_state.is_empty() {
            parts.push(format!(
                "CHARACTER'S CURRENT STATE:\n{}",
                serde_json::to_string_pretty(&character.state).unwrap_or_default()
            ));
        }
    }

    if let Some(time_desc) = world_time_description(world) {
        parts.push(time_desc);
    }

    if let Some(directive) = mood_directive {
        if !directive.is_empty() {
            parts.push(format!("CHARACTER MOOD:\n{directive}"));
        }
    }

    // Narration tone and custom instructions
    let has_tone = narration_tone.map(|t| !t.is_empty() && t != "Auto").unwrap_or(false);
    let has_instructions = narration_instructions.map(|i| !i.is_empty()).unwrap_or(false);
    if has_tone || has_instructions {
        let mut direction = Vec::new();
        if let Some(tone) = narration_tone {
            if !tone.is_empty() && tone != "Auto" {
                direction.push(format!("TONE: Write in a {tone} tone. Let this flavor permeate the atmosphere, imagery, actions, and emotional texture of the narrative. Generate actions and events that fit the tone — not just descriptive atmosphere."));
            }
        }
        if let Some(instructions) = narration_instructions {
            if !instructions.is_empty() {
                direction.push(format!("CUSTOM DIRECTION:\n{instructions}"));
            }
        }
        parts.push(direction.join("\n\n"));
    }

    parts.push(
        r#"CRAFT:
- Write 2–5 sentences of rich, sensory prose. Be vivid, be bold.
- You may introduce new environmental details, body language, subtle actions, atmosphere, weather, sounds, smells, textures, internal feelings.
- You may advance the moment — shift the scene, introduce a small surprise, or reveal something about the character through action or expression.
- Stay consistent with the world, the conversation, and both characters' established personalities.
- Do NOT write dialogue or spoken words. No quotation marks.
- Do NOT break the fourth wall. Do NOT reference the chat, the app, or the AI.
- Be creative. Take risks. Make it feel alive.
- The user may provide specific direction for this narrative beat. If they do, follow it above all else — it takes absolute priority over tone, mood, and other guidance."#
            .to_string(),
    );

    parts.join("\n\n")
}

pub fn build_scene_description_prompt(
    world: &World,
    character: &Character,
    user_profile: Option<&UserProfile>,
    recent_messages: &[Message],
) -> Vec<crate::ai::openai::ChatMessage> {
    let user_name = user_profile
        .map(|p| p.display_name.as_str())
        .unwrap_or("the human");

    let mut system_parts = Vec::new();

    system_parts.push(format!(
        "You are a visual scene director. Your job is to describe the current moment between {user} and {char} \
         as a single, detailed image description suitable for an illustrator or image generation model.",
        user = user_name,
        char = character.display_name,
    ));

    system_parts.push(format!(
        "CHARACTERS:\n\
         - {user}: the human. Refer to them by name or appearance, not as \"you\".\n\
         - {char}: the fictional character.",
        user = user_name,
        char = character.display_name,
    ));

    if !character.identity.is_empty() {
        let identity = if character.identity.len() > 300 {
            format!("{}...", &character.identity[..300])
        } else {
            character.identity.clone()
        };
        system_parts.push(format!("{} is: {}", character.display_name, identity));
    }

    if let Some(profile) = user_profile {
        if !profile.description.is_empty() {
            system_parts.push(format!("{} is: {}", profile.display_name, profile.description));
        }
    }

    if !world.description.is_empty() {
        let desc = if world.description.len() > 300 {
            format!("{}...", &world.description[..300])
        } else {
            world.description.clone()
        };
        system_parts.push(format!("WORLD SETTING:\n{desc}"));
    }

    if let Some(time_desc) = world_time_description(world) {
        system_parts.push(time_desc);
    }

    system_parts.push(r#"OUTPUT INSTRUCTIONS:
- First, write a detailed scene description as a single paragraph (4-8 sentences): environment, lighting, weather, spatial arrangement of the two characters, their poses, expressions, body language, clothing, and any notable objects or details.
- Write in third person, present tense, as if describing a painting.
- Be specific about spatial relationships: who is where, facing which direction, what they're doing with their hands, eyes, body.
- Include atmosphere: time of day, colors, mood, textures. The lighting MUST match the current time of day.
- Do NOT include dialogue, speech bubbles, or text.
- Do NOT include meta-instructions like "paint this" or "in watercolor style" — just describe the scene itself.
- Both characters must appear in the scene.
- Keep the description PG. No nudity, explicit sexual content, or graphic violence. Romantic or tense moments are fine, but keep them tasteful and implied rather than explicit."#.to_string());

    let system = system_parts.join("\n\n");

    let mut msgs = vec![crate::ai::openai::ChatMessage {
        role: "system".to_string(),
        content: system,
    }];

    // Include recent conversation as context (skip illustrations)
    let conversation: Vec<String> = recent_messages.iter()
        .filter(|m| m.role != "illustration" && m.role != "video")
        .map(|m| {
            let speaker = if m.role == "user" {
                user_name.to_string()
            } else if m.role == "narrative" {
                "[Narrative]".to_string()
            } else {
                character.display_name.clone()
            };
            format!("{}: {}", speaker, m.content)
        })
        .collect();

    msgs.push(crate::ai::openai::ChatMessage {
        role: "user".to_string(),
        content: format!(
            "Here is the recent conversation:\n\n{}\n\nDescribe the current scene as a single illustration showing both {} and {}. Focus especially on the last two messages — depict the physical situation, positions, and actions happening right now in this moment.",
            conversation.join("\n"),
            user_name,
            character.display_name,
        ),
    });

    msgs
}

pub fn build_animation_prompt(
    world: &World,
    character: &Character,
    user_profile: Option<&UserProfile>,
    recent_messages: &[Message],
) -> Vec<crate::ai::openai::ChatMessage> {
    let user_name = user_profile
        .map(|p| p.display_name.as_str())
        .unwrap_or("the human");

    let mut system_parts = vec![format!(
        r#"You are a motion director. Given a conversation between {user} and {char}, write a vivid animation direction (2-4 sentences) describing how to bring a still illustration of their current scene to life as a short video.

The animation should be a natural continuation of the action and emotion in the scene. Be bold — characters can move, gesture, react, shift position, interact with objects, and express themselves. The environment can change too: weather, light, background activity.

Keep it PG. No nudity, explicit sexual content, or graphic violence. Romantic or tense moments are fine, but keep them tasteful and implied rather than explicit.
Do NOT describe camera movements or use technical film terms. Just describe what happens — the motion, the action, the life in the scene.
Write ONLY the animation direction, nothing else."#,
        user = user_name,
        char = character.display_name,
    )];

    if let Some(time_desc) = world_time_description(world) {
        system_parts.push(time_desc);
    }

    // Include character descriptions so the prompt can reference them
    if !character.identity.is_empty() {
        let id = if character.identity.len() > 150 { format!("{}...", &character.identity[..150]) } else { character.identity.clone() };
        system_parts.push(format!("{} is: {}", character.display_name, id));
    }
    if let Some(profile) = user_profile {
        if !profile.description.is_empty() {
            let desc = if profile.description.len() > 150 { format!("{}...", &profile.description[..150]) } else { profile.description.clone() };
            system_parts.push(format!("{} is: {}", profile.display_name, desc));
        }
    }

    let system = system_parts.join("\n\n");

    let conversation: Vec<String> = recent_messages.iter()
        .filter(|m| m.role != "illustration" && m.role != "video")
        .rev().take(6).collect::<Vec<_>>().into_iter().rev()
        .map(|m| {
            let speaker = if m.role == "user" {
                user_name.to_string()
            } else if m.role == "narrative" {
                "[Narrative]".to_string()
            } else {
                character.display_name.clone()
            };
            format!("{}: {}", speaker, m.content)
        })
        .collect();

    vec![
        crate::ai::openai::ChatMessage {
            role: "system".to_string(),
            content: system,
        },
        crate::ai::openai::ChatMessage {
            role: "user".to_string(),
            content: format!(
                "Recent conversation:\n{}\n\nWrite the animation direction for the current scene.",
                conversation.join("\n"),
            ),
        },
    ]
}

fn world_time_description(world: &World) -> Option<String> {
    let time = world.state.get("time")?;
    let time_of_day = time.get("time_of_day").and_then(|v| v.as_str()).unwrap_or("");
    if time_of_day.is_empty() { return None; }
    let lighting = match time_of_day.to_uppercase().as_str() {
        "DAWN" => "early dawn light, sky shifting from deep blue to warm gold at the horizon",
        "MORNING" => "bright morning light, warm and clear",
        "MIDDAY" => "high midday sun, strong overhead light with short shadows",
        "AFTERNOON" => "warm afternoon light, long golden rays",
        "EVENING" | "DUSK" => "dusky evening light, warm oranges and purples in the sky, shadows growing long",
        "NIGHT" => "nighttime, lit by moonlight and/or artificial light sources, deep blues and shadows",
        "LATE NIGHT" => "deep night, very dark, only dim ambient light or artificial glow",
        _ => "",
    };
    let mut desc = format!("TIME OF DAY: {time_of_day}.");
    if !lighting.is_empty() {
        desc.push_str(&format!(" The lighting and atmosphere should reflect this: {lighting}."));
    }
    Some(desc)
}

fn json_array_to_strings(val: &Value) -> Vec<String> {
    match val.as_array() {
        Some(arr) => arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
        None => Vec::new(),
    }
}
