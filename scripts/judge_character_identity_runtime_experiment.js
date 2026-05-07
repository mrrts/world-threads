#!/usr/bin/env node

const fs = require("fs");
const path = require("path");
const { execFileSync } = require("child_process");

const repoRoot = path.resolve(__dirname, "..");
const srcTauri = path.join(repoRoot, "src-tauri");
const outputDir =
  process.argv[2] || path.join("/private/tmp", "character-identity-runtime-exp");

function run(cmd, args, opts = {}) {
  return execFileSync(cmd, args, {
    cwd: opts.cwd || repoRoot,
    input: opts.input,
    encoding: "utf8",
    maxBuffer: 50 * 1024 * 1024,
    stdio: opts.input ? ["pipe", "pipe", "pipe"] : ["ignore", "pipe", "pipe"],
  });
}

function loadFixture(fixture) {
  const raw = fs.readFileSync(
    path.join(srcTauri, "tests", "fixtures", "character_identity", `${fixture}.json`),
    "utf8"
  );
  return JSON.parse(raw);
}

const generated = JSON.parse(
  fs.readFileSync(path.join(outputDir, "generated-results.json"), "utf8")
);

const byCell = new Map();
for (const row of generated.results) {
  const key = `${row.fixture}:${row.prompt_id}`;
  const cell = byCell.get(key) || {};
  cell[row.mode] = row;
  byCell.set(key, cell);
}

const cells = [];
for (const [key, pair] of byCell.entries()) {
  const [fixture, promptId] = key.split(":");
  const source = loadFixture(fixture);
  const flip = cells.length % 2 === 0;
  const left = flip ? pair.prose.reply : pair.compressed.reply;
  const right = flip ? pair.compressed.reply : pair.prose.reply;
  const leftMode = flip ? "prose" : "compressed";
  const rightMode = flip ? "compressed" : "prose";
  cells.push({
    cell_id: key,
    fixture,
    prompt_id: promptId,
    display_name: source.display_name,
    user_message: pair.prose.user_message,
    source: {
      identity: source.identity,
      voice_rules: source.voice_rules,
      boundaries: source.boundaries,
      backstory_facts: source.backstory_facts,
      relationships: source.relationships,
      state: source.state,
    },
    left,
    right,
    left_mode: leftMode,
    right_mode: rightMode,
  });
}

const judgePrompt = [
  "You are blindly judging whether two candidate replies preserve the same character from source materials.",
  "You do NOT know which side came from prose identity and which came from compressed identity.",
  "For each cell, judge LEFT and RIGHT independently against the source character.",
  "Use only these ratings: faithful, mixed, drift.",
  "Then name which side is stronger overall: left, right, tie.",
  "Also name the main failure mode if present: role_drift, relation_drift, voice_flattening, refusal_loss, wound_simplification, theological_blur, generic_warmth, or none.",
  "Return valid JSON only: an array of objects with keys cell_id, left_rating, right_rating, stronger_side, failure_mode, notes.",
  "",
  JSON.stringify(cells, null, 2),
].join("\n");

const outFile = path.join(outputDir, "blind-judge.json");
run("codex", ["exec", "--ephemeral", "-C", repoRoot, "-o", outFile, "-"], {
  cwd: repoRoot,
  input: judgePrompt,
});

const blindText = fs.readFileSync(outFile, "utf8").trim();
const blind = JSON.parse(blindText);
const summary = {
  generated_at: new Date().toISOString(),
  cells,
  blind,
};
fs.writeFileSync(
  path.join(outputDir, "blind-judge-packet.json"),
  JSON.stringify(summary, null, 2)
);

const aggregate = {
  compressed_faithful: 0,
  compressed_mixed: 0,
  compressed_drift: 0,
  prose_faithful: 0,
  prose_mixed: 0,
  prose_drift: 0,
  compressed_wins: 0,
  prose_wins: 0,
  ties: 0,
};

for (const verdict of blind) {
  const cell = cells.find((c) => c.cell_id === verdict.cell_id);
  if (!cell) continue;
  const compressedRating =
    cell.left_mode === "compressed" ? verdict.left_rating : verdict.right_rating;
  const proseRating =
    cell.left_mode === "prose" ? verdict.left_rating : verdict.right_rating;
  aggregate[`compressed_${compressedRating}`] += 1;
  aggregate[`prose_${proseRating}`] += 1;
  if (verdict.stronger_side === "tie") aggregate.ties += 1;
  else if (
    (verdict.stronger_side === "left" && cell.left_mode === "compressed") ||
    (verdict.stronger_side === "right" && cell.right_mode === "compressed")
  ) {
    aggregate.compressed_wins += 1;
  } else {
    aggregate.prose_wins += 1;
  }
}

fs.writeFileSync(
  path.join(outputDir, "aggregate-summary.json"),
  JSON.stringify(aggregate, null, 2)
);

process.stdout.write(`${JSON.stringify(aggregate, null, 2)}\n`);
