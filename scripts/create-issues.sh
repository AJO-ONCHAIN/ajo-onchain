#!/usr/bin/env bash
#
# Open every issue in docs/ISSUES.md on GitHub, in order, with labels.
#
# ## Why this script exists rather than 23 manual clicks
#
# The issue numbers are not cosmetic. Every stub in the source carries a
# `TODO(#n)` marker, and `docs/ISSUES.md` promises those numbers are stable.
# If issue #14 is not "Handle `created`: upsert the circle", a contributor who
# greps for the marker lands on the wrong issue and the whole contributor
# workflow quietly stops making sense.
#
# GitHub assigns issue numbers from a counter that **pull requests also
# consume**. So the mapping only holds if these are created in order, in a
# repository where nothing has been opened yet. This script enforces that
# instead of hoping.
#
# Usage:
#   ./scripts/create-issues.sh --dry-run     # print what would be created
#   ./scripts/create-issues.sh               # create them for real
#
#   --repo OWNER/NAME   target a specific repository (default: git remote)
#   --force             skip the empty-tracker precondition. Read the warning
#                       it prints before you reach for this.
#
# Requires: gh (https://cli.github.com), authenticated with `gh auth login`.

set -euo pipefail

ISSUES_FILE="docs/ISSUES.md"
FIRST_ISSUE=2
LAST_ISSUE=23

DRY_RUN=false
FORCE=false
REPO=""

info() { printf '\033[0;34m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[0;33mwarning:\033[0m %s\n' "$*" >&2; }
fail() { printf '\033[0;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run) DRY_RUN=true; shift ;;
    --force)   FORCE=true; shift ;;
    --repo)    REPO="${2:-}"; [ -n "$REPO" ] || fail "--repo needs OWNER/NAME"; shift 2 ;;
    -h|--help) sed -n '2,28p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *)         fail "unknown argument: $1" ;;
  esac
done

# --- Preconditions ----------------------------------------------------------

[ -f "$ISSUES_FILE" ] || fail "$ISSUES_FILE not found. Run this from the repository root."

# A dry run only parses the file, so it deliberately works before `gh` is
# installed or a remote exists. That is the point: you can check the titles,
# labels and bodies are right while the repository is still local.
if [ "$DRY_RUN" != true ]; then
  command -v gh >/dev/null 2>&1 || fail "gh not found. Install it: https://cli.github.com"

  gh auth status >/dev/null 2>&1 || fail "gh is not authenticated. Run: gh auth login"

  if [ -z "$REPO" ]; then
    REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null)" \
      || fail "could not determine the repository. Add a git remote, or pass --repo OWNER/NAME"
  fi
  info "target repository: $REPO"

  # The numbering guarantee. A repository that already has an issue or a pull
  # request has already advanced the counter, and every TODO(#n) marker in the
  # source would point somewhere wrong.
  EXISTING="$(gh issue list --repo "$REPO" --state all --limit 1 --json number -q 'length')"
  EXISTING_PRS="$(gh pr list --repo "$REPO" --state all --limit 1 --json number -q 'length')"

  if [ "$EXISTING" != "0" ] || [ "$EXISTING_PRS" != "0" ]; then
    warn "this repository already has issues or pull requests."
    warn "GitHub numbers issues and PRs from one shared counter, so the numbers"
    warn "created here will NOT match the TODO(#n) markers in the source."
    if [ "$FORCE" != true ]; then
      fail "refusing to create mismatched issues. Use --force only if you intend to
    fix the markers in the source afterwards."
    fi
    warn "--force given; continuing with numbering that will not match."
  fi
else
  REPO="${REPO:-<not resolved — dry run>}"
fi

# --- Labels -----------------------------------------------------------------

# `good first issue` and `help wanted` exist by default in a new repository,
# but not in one created from a template or with a customised label set, so
# they are created idempotently along with the two area labels.
ensure_label() {
  local name="$1" color="$2" desc="$3"
  if [ "$DRY_RUN" = true ]; then
    printf '    label: %s\n' "$name"
    return
  fi
  gh label create "$name" --repo "$REPO" --color "$color" --description "$desc" \
    --force >/dev/null 2>&1 || warn "could not create or update label: $name"
}

info "ensuring labels exist"
ensure_label "good first issue" "7057ff" "Scoped for a first-time contributor"
ensure_label "help wanted"      "008672" "Open for anyone to claim"
ensure_label "contracts"        "b60205" "Rust / Soroban, in contracts/"
ensure_label "indexer"          "0e8a16" "TypeScript, in apps/indexer/"
ensure_label "tracking"         "fbca04" "Umbrella issue"

# --- Extraction -------------------------------------------------------------

# Title of issue N: the `## #N — ...` heading, minus the marker.
issue_title() {
  awk -v want="$1" '
    /^## #/ {
      line = substr($0, 5)                  # drop "## #"
      num = line; sub(/ .*$/, "", num)
      if (num == want) {
        title = substr(line, length(num) + 1)
        sub(/^[[:space:]]*/, "", title)
        sub(/^—[[:space:]]*/, "", title)    # em dash separator
        print title
        exit
      }
    }
  ' "$ISSUES_FILE"
}

# Body of issue N: everything between its heading and the next one.
issue_body() {
  awk -v want="$1" '
    /^## #/ {
      line = substr($0, 5)
      num = line; sub(/ .*$/, "", num)
      if (num == want) { inblock = 1; next }
      if (inblock) { exit }
    }
    inblock { print }
  ' "$ISSUES_FILE" | sed -e '/./,$!d' | awk 'NF {blank = 0; print; next} {blank++; if (blank < 2) print}'
}

# Area label for issue N, read from the summary table at the top of the file.
issue_area() {
  awk -F'|' -v want="$1" '
    /^\| *[0-9]+ *\|/ {
      num = $2; gsub(/ /, "", num)
      if (num == want) {
        area = $4; gsub(/^ +| +$/, "", area)
        print area
        exit
      }
    }
  ' "$ISSUES_FILE"
}

# Priority label for issue N: the backticked label on its first content line.
issue_label() {
  issue_body "$1" | grep -m1 '^\*\*Label:\*\*' | sed -e 's/.*`\([^`]*\)`.*/\1/'
}

# --- The tracking issue (#1) ------------------------------------------------

TRACKING_BODY="$(cat <<'BODY'
Umbrella issue for the Tranche 1 contributor backlog.

Every stub in this repository carries a `TODO(#n)` marker whose number is the
issue number below. Find them all with:

```bash
grep -rn "TODO(#" --include="*.rs" --include="*.ts" .
```

The full write-up for each — definition of done, skills, effort — lives in
[`docs/ISSUES.md`](docs/ISSUES.md). Read
[`CONTRIBUTING.md`](CONTRIBUTING.md) first; it explains the setup and the
three security invariants that a pull request may not weaken.

**New to Rust, Soroban, or open source?** The issues labelled
`good first issue` are scoped for you, and a broken setup is our bug, not
yours — open an issue if `task contracts:test` does not pass.

### Contracts

- [ ] #2 · #3 · #4 · #5 · #6 · #7 · #8 · #9 · #10 · #12

### Indexer

- [ ] #11 · #13 · #14 · #15 · #16 · #17 · #18 · #19 · #20 · #21 · #22 · #23

When you finish one, delete the `TODO(#n)` marker and its entry in
`docs/ISSUES.md` in the same pull request.
BODY
)"

# --- Create -----------------------------------------------------------------

# Pull the number out of the URL gh prints, so a mismatch is caught on the
# issue that caused it rather than twenty issues later.
created_number() {
  printf '%s\n' "$1" | sed -n 's#.*/issues/\([0-9]\{1,\}\).*#\1#p' | tail -1
}

create_issue() {
  local expected="$1" title="$2" body="$3"
  shift 3
  local labels=("$@")

  if [ "$DRY_RUN" = true ]; then
    printf '\n  #%s  %s\n' "$expected" "$title"
    printf '        labels: %s\n' "$(IFS=', '; echo "${labels[*]}")"
    printf '        body:   %s lines\n' "$(printf '%s\n' "$body" | wc -l | tr -d ' ')"
    return
  fi

  local args=(--repo "$REPO" --title "$title" --body "$body")
  local label
  for label in "${labels[@]}"; do
    args+=(--label "$label")
  done

  local url actual
  url="$(gh issue create "${args[@]}")" || fail "failed creating issue #$expected: $title"
  actual="$(created_number "$url")"

  if [ "$actual" != "$expected" ] && [ "$FORCE" != true ]; then
    fail "numbering drift: expected #$expected, GitHub assigned #$actual
    ($url)
    Stopping here so the mismatch stays small. Either delete the issues
    created so far and start again in a clean repository, or renumber the
    TODO markers in the source to match."
  fi

  printf '  #%-3s %s\n' "$actual" "$url"
}

if [ "$DRY_RUN" = true ]; then
  info "dry run — nothing will be created"
fi

info "creating the tracking issue (#1)"
create_issue 1 "Tranche 1 contributor backlog" "$TRACKING_BODY" "tracking"

info "creating issues #$FIRST_ISSUE–#$LAST_ISSUE from $ISSUES_FILE"
for n in $(seq "$FIRST_ISSUE" "$LAST_ISSUE"); do
  title="$(issue_title "$n")"
  [ -n "$title" ] || fail "no heading '## #$n' found in $ISSUES_FILE"

  body="$(issue_body "$n")"
  [ -n "$body" ] || fail "empty body for issue #$n"

  label="$(issue_label "$n")"
  [ -n "$label" ] || fail "no **Label:** line found for issue #$n"

  area="$(issue_area "$n")"
  [ -n "$area" ] || fail "issue #$n is missing from the summary table in $ISSUES_FILE"

  body="${body}

---
Full write-up: [\`docs/ISSUES.md\`](docs/ISSUES.md) · Before you start, read [\`CONTRIBUTING.md\`](CONTRIBUTING.md). Part of #1."

  create_issue "$n" "$title" "$body" "$label" "$area"
done

if [ "$DRY_RUN" = true ]; then
  info "dry run complete. Re-run without --dry-run to create them."
else
  info "done. $((LAST_ISSUE - FIRST_ISSUE + 2)) issues created in $REPO"
  info "check them: gh issue list --repo $REPO"
fi
