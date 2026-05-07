#!/usr/bin/env node

const fs = require("fs");
const path = require("path");
const { execFileSync } = require("child_process");

const repoRoot = path.resolve(__dirname, "..");
const srcTauri = path.join(repoRoot, "src-tauri");
const outputDir =
  process.argv[2] || path.join("/private/tmp", "character-identity-runtime-exp");

const characters = [
  { fixture: "steven", name: "Steven" },
  { fixture: "pastor_rick", name: "Pastor Rick" },
  { fixture: "jasper_finn", name: "Jasper Finn" },
];

const prompts = [
  {
    id: "relation",
    text: "I don't need a big speech. I'd just like to sit with you a while, maybe over coffee, and hear one true thing from your day.",
  },
  {
    id: "boundary",
    text: "Could you just let this one thing slide for me? I know it crosses a line you usually keep, but it'd make tonight easier.",
  },
  {
    id: "wound",
    text: "What would you want if you stopped bracing for disappointment for one honest minute?",
  },
  {
    id: "moral",
    text: "What makes a person worth staying true to when it costs them something?",
  },
  {
    id: "ordinary",
    text: "It's just Tuesday morning. Coffee's on. Nothing urgent. What's the room feel like to you right now, and what do you do with that kind of quiet?",
  },
];

fs.mkdirSync(outputDir, { recursive: true });

function run(cmd, args, opts = {}) {
  return execFileSync(cmd, args, {
    cwd: opts.cwd || repoRoot,
    input: opts.input,
    encoding: "utf8",
    maxBuffer: 50 * 1024 * 1024,
    stdio: opts.input ? ["pipe", "pipe", "pipe"] : ["ignore", "pipe", "pipe"],
  });
}

function loadPromptArm(fixture, mode) {
  const raw = run(
    "cargo",
    [
      "run",
      "-q",
      "--bin",
      "character_identity_runtime_probe",
      "--",
      "--fixture",
      fixture,
      "--mode",
      mode,
      "--user-message",
      prompts[0].text,
      "--json",
    ],
    { cwd: srcTauri }
  );
  return JSON.parse(raw);
}

function codexReply(systemPrompt, userMessage, outFile) {
  if (fs.existsSync(outFile)) {
    return fs.readFileSync(outFile, "utf8").trim();
  }

  const prompt = [
    "Generate exactly one in-character reply.",
    "Obey the system prompt as authoritative.",
    "Output only the reply text, with no explanation, no markdown fences, and no preface.",
    "",
    "<SYSTEM_PROMPT>",
    systemPrompt,
    "</SYSTEM_PROMPT>",
    "",
    "<USER_MESSAGE>",
    userMessage,
    "</USER_MESSAGE>",
  ].join("\n");

  run("codex", ["exec", "--ephemeral", "-C", repoRoot, "-o", outFile, "-"], {
    cwd: repoRoot,
    input: prompt,
  });
  const reply = fs.readFileSync(outFile, "utf8").trim();
  return reply;
}

const promptArms = {};
for (const character of characters) {
  promptArms[character.fixture] = {
    prose: loadPromptArm(character.fixture, "prose"),
    compressed: loadPromptArm(character.fixture, "compressed"),
  };
}

const results = [];
for (const character of characters) {
  for (const prompt of prompts) {
    for (const mode of ["prose", "compressed"]) {
      const outFile = path.join(
        outputDir,
        `${character.fixture}-${prompt.id}-${mode}.txt`
      );
      const reply = codexReply(
        promptArms[character.fixture][mode].system_prompt,
        prompt.text,
        outFile
      );
      results.push({
        fixture: character.fixture,
        display_name: character.name,
        prompt_id: prompt.id,
        mode,
        user_message: prompt.text,
        reply,
      });
      process.stdout.write(
        `generated ${character.fixture} ${prompt.id} ${mode}\n`
      );
    }
  }
}

const packet = {
  generated_at: new Date().toISOString(),
  output_dir: outputDir,
  characters,
  prompts,
  prompt_arms: promptArms,
  results,
};

fs.writeFileSync(
  path.join(outputDir, "generated-results.json"),
  JSON.stringify(packet, null, 2)
);

process.stdout.write(
  `wrote ${path.join(outputDir, "generated-results.json")}\n`
);
