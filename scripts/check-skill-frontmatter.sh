#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

ruby - "$ROOT_DIR" <<'RUBY'
require "yaml"

root = ARGV.fetch(0)
skill_files = Dir.glob(File.join(root, "{.agents,.claude}", "skills", "*", "SKILL.md")).sort

errors = []

skill_files.each do |path|
  raw = File.read(path)

  unless raw.start_with?("---\n")
    errors << "#{path}: missing YAML frontmatter delimited by ---"
    next
  end

  parts = raw.split(/^---\s*$\n/, 3)
  if parts.length < 3
    errors << "#{path}: frontmatter fence is incomplete"
    next
  end

  frontmatter = parts[1]

  begin
    data = YAML.safe_load(frontmatter)
  rescue Psych::SyntaxError => e
    errors << "#{path}: invalid YAML: #{e.problem} at line #{e.line} column #{e.column}"
    next
  end

  unless data.is_a?(Hash)
    errors << "#{path}: frontmatter must parse to a mapping"
    next
  end

  value = data["description"]
  if !value.is_a?(String) || value.strip.empty?
    errors << "#{path}: frontmatter missing non-empty description"
  end
end

if errors.empty?
  puts "skill-frontmatter | ok | checked=#{skill_files.length}"
  exit 0
end

puts "skill-frontmatter | errors=#{errors.length}"
errors.each { |msg| puts "⚠ #{msg}" }
exit 1
RUBY
