#!/bin/sh
# The GitHub Action's one step (D22): classify the pull request's diff against personajury.toml and, when a committed store
# is given, run `check` and `apply-check`. No model, no network, no secret: it reads the diff and the store.
# Environment (action.yml sets these; tests set them to a local binary):
#   JG_BIN       the binary (default: personajury on PATH)
#   JG_CONFIG    the rules file (default personajury.toml)
#   JG_BASE      the base to diff against (default: origin/$GITHUB_BASE_REF on a pull request, else HEAD~1)
#   JG_STORE     a project root holding a committed .personajury (optional)
#   JG_FAIL_ON   comma list of answers that fail the job: human, escalate, advise, unavailable
#                (default human,escalate,unavailable); a failing `check` always fails it
# Fails closed (D13): a missing binary, an error from should-run, check or apply-check, or output that is not JSON makes the
# answer `unavailable`, which fails the job unless JG_FAIL_ON is set without `unavailable`.
# With a store, `apply-check --latest` judges every subject that has runs and answers the worst; when should-run matched
# exactly one rule its class is passed as --rule (the store's root must hold the same personajury.toml).
# Writes job outputs to $GITHUB_OUTPUT (quorum, spend, effort, answer, check) and a table to $GITHUB_STEP_SUMMARY. Needs jq.

# Markdown-escape one line for the step summary: file names come from the pull request (a fork's names included).
md_escape() {
  printf '%s' "$1" | sed -e 's/[][\\`*_{}()#+!|<>~]/\\&/g'
}
# True when $1 is a JSON envelope with ok = true; false for an error envelope, empty output or anything not JSON.
envelope_ok() {
  printf '%s' "$1" | jq -e '.ok == true' >/dev/null 2>&1
}

main() {
  set -u
  bin="${JG_BIN:-personajury}"
  config="${JG_CONFIG:-personajury.toml}"
  store="${JG_STORE:-}"
  fail_on="${JG_FAIL_ON:-human,escalate,unavailable}"
  out="${GITHUB_OUTPUT:-/dev/null}"
  summary="${GITHUB_STEP_SUMMARY:-/dev/null}"
  tmp="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"

  quorum=unavailable
  spend='[]'
  effort=medium
  answer=none
  check=skipped
  reasons=""
  unmatched_n=0
  unmatched=""
  spend_words=""
  why=""
  unavailable() {
    answer=unavailable
    why="${why:+$why; }$1"
    echo "personajury action: $1" >&2
  }

  if ! command -v jq >/dev/null 2>&1; then
    unavailable "jq is needed to read the engine's answers"
  elif ! command -v "$bin" >/dev/null 2>&1; then
    unavailable "no personajury binary at $bin"
  else
    base="${JG_BASE:-}"
    if [ -z "$base" ]; then
      if [ -n "${GITHUB_BASE_REF:-}" ]; then base="origin/$GITHUB_BASE_REF"; else base="HEAD~1"; fi
    fi
    diff="$tmp/personajury.diff"
    if ! git diff "$base...HEAD" > "$diff"; then
      unavailable "git diff $base...HEAD failed (fetch-depth: 0 on checkout?)"
    else
      sr="$("$bin" should-run --diff "$diff" --config "$config" 2>/dev/null)" || true
      if ! envelope_ok "$sr"; then
        unavailable "should-run failed or did not answer JSON"
      else
        quorum="$(printf '%s' "$sr" | jq -r '.data.quorum')"
        spend="$(printf '%s' "$sr" | jq -c '.data.spend')"
        spend_words="$(printf '%s' "$sr" | jq -r '[.data.spend[] | "\(.class): \(.personas), \(.lenses) lens(es)"] | join("; ")')"
        unmatched_n="$(printf '%s' "$sr" | jq -r '.data.unmatched | length')"
        unmatched="$(printf '%s' "$sr" | jq -r '.data.unmatched | join(", ")')"
        rule="$(printf '%s' "$sr" | jq -r 'if (.data.hits | length) == 1 then .data.hits[0].class else "" end')"
        e="$(sed -n 's/^[[:space:]]*effort[[:space:]]*=[[:space:]]*"\([a-z]*\)".*/\1/p' "$config" 2>/dev/null | head -1)"
        [ -z "$e" ] || effort="$e"
        if [ -n "$store" ]; then
          ck="$("$bin" --root "$store" check 2>/dev/null)" || true
          if envelope_ok "$ck"; then
            check=ok
          elif [ "$(printf '%s' "$ck" | jq -r '.error.code' 2>/dev/null)" = check_failed ]; then
            check=failed
          else
            check=unavailable
            unavailable "check did not run (an error, not a failing check)"
          fi
          if [ -n "$rule" ]; then
            ac="$("$bin" --root "$store" apply-check --latest --rule "$rule" 2>/dev/null)" || true
          else
            ac="$("$bin" --root "$store" apply-check --latest 2>/dev/null)" || true
          fi
          if envelope_ok "$ac"; then
            [ "$answer" = unavailable ] || answer="$(printf '%s' "$ac" | jq -r '.data.answer')"
            reasons="$(printf '%s' "$ac" | jq -r '[.data.reasons[] | select(.effect != "info") | .code] | unique | join(", ")')"
          else
            unavailable "apply-check failed: $(printf '%s' "$ac" | jq -r '.error.message' 2>/dev/null | head -1)"
          fi
        fi
      fi
    fi
  fi

  {
    echo "quorum=$quorum"
    echo "spend=$spend"
    echo "effort=$effort"
    echo "answer=$answer"
    echo "check=$check"
  } >> "$out"

  {
    echo "### PersonaJury"
    echo
    echo "| | |"
    echo "|---|---|"
    echo "| Quorum | $quorum |"
    echo "| Effort | $effort |"
    echo "| Spend | $(md_escape "$spend_words") |"
    [ -z "$store" ] || echo "| Check | $check |"
    [ -z "$store" ] && [ "$answer" != unavailable ] || echo "| Apply-check | $answer${reasons:+ ($reasons)} |"
    echo
    if [ "$answer" = unavailable ]; then
      echo "The answer is unavailable, so nothing is proven: $(md_escape "$why")."
      echo
    fi
    case "$quorum" in
      none) echo "No jury needed: the tests and the adversarial read decide." ;;
      human) echo "A person decides: the jury may advise but cannot apply this change." ;;
      unavailable) echo "The rules could not be read." ;;
      *) echo "A $quorum jury is called for. The engine runs no model; the agent that opens this pull request runs it." ;;
    esac
    if [ "$unmatched_n" -gt 0 ]; then
      noun=files; [ "$unmatched_n" -eq 1 ] && noun=file
      echo
      echo "$unmatched_n $noun matched no rule and got no jury: $(md_escape "$unmatched"); add a rule if one of them is something people see."
    fi
  } >> "$summary"

  rc=0
  if [ "$check" = failed ]; then echo "personajury action: check failed: findings with no decision, or a fix that did not land" >&2; rc=1; fi
  for a in $(echo "$fail_on" | tr ',' ' '); do
    if [ "$a" = "$answer" ] || { [ "$a" = human ] && [ "$quorum" = human ]; }; then
      echo "personajury action: failing on $a (fail-on: $fail_on)" >&2; rc=1
    fi
  done
  exit "$rc"
}
main "$@"
