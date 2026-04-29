#!/usr/bin/env bash
# Source this file to enable simple completions for helper scripts:
#   source scripts/completions/register-shift-tools.bash

_wc_latest_register_shift_run() {
  COMPREPLY=()
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local opts="--help --json --pretty --name-only --list-files"
  COMPREPLY=( $(compgen -W "$opts" -- "$cur") )
}

_wc_show_latest_register_shift_run() {
  COMPREPLY=()
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local prev="${COMP_WORDS[COMP_CWORD-1]}"
  if [[ "$prev" == "--format" ]]; then
    COMPREPLY=( $(compgen -W "text csv json" -- "$cur") )
    return
  fi
  if [[ "$prev" == "--latest-only" ]]; then
    COMPREPLY=( $(compgen -W "all shift pack rebound" -- "$cur") )
    return
  fi
  local opts="--help --quiet --format --latest-only"
  COMPREPLY=( $(compgen -W "$opts" -- "$cur") )
}

_wc_register_shift_dashboard() {
  COMPREPLY=()
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local opts="loose medium strict --help --commit-artifacts"
  COMPREPLY=( $(compgen -W "$opts" -- "$cur") )
}

_wc_run_rebound_strict() {
  COMPREPLY=()
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local opts="--help --commit-artifacts"
  COMPREPLY=( $(compgen -W "$opts" -- "$cur") )
}

complete -F _wc_latest_register_shift_run ./scripts/latest-register-shift-run.sh
complete -F _wc_show_latest_register_shift_run ./scripts/show-latest-register-shift-run.sh
complete -F _wc_register_shift_dashboard ./scripts/register-shift-dashboard.sh
complete -F _wc_run_rebound_strict ./scripts/run-rebound-strict.sh
