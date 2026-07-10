#!/usr/bin/env bash
# Cascade Splits Registry — Issues #6 through #30
# Requires: gh CLI installed and authenticated (`gh auth login`), run from inside the target repo
# (or pass --repo owner/name to each call — see REPO variable below).
#
# Usage:
#   1. Edit REPO below to your actual "owner/repo" (e.g. "tali-dev/cascade")
#   2. chmod +x create_issues.sh
#   3. ./create_issues.sh
#
# Each issue is created serially, in order, matching the dependency chain
# (#6 depends on #5, #7 depends on #6, etc.) as documented in each body file.

set -euo pipefail

REPO="tali-creator/stellar-cascade"   # <-- EDIT THIS
LABEL="contracts-registry"                    # optional label; comment out the --label line below if unused
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

declare -a TITLES=(
  "[Cascade-Registry] Validate Split Percentages Sum to 100%"
  "[Cascade-Registry] Reject Duplicate Receiver Addresses"
  "[Cascade-Registry] Enforce Minimum and Maximum Receiver Count"
  "[Cascade-Registry] Define Contract Error Enum"
  "[Cascade-Registry] Wire Error Enum into Validation and Registration"
  "[Cascade-Registry] Require Owner Authorization on Registration"
  "[Cascade-Registry] Prevent Re-registration of an Existing Project ID"
  "[Cascade-Registry] Implement Public get_project Read Function"
  "[Cascade-Registry] Implement Public has_project Convenience Function"
  "[Cascade-Registry] Emit Event on Project Registration"
  "[Cascade-Registry] Implement update_splits Function Skeleton"
  "[Cascade-Registry] Require Owner Authorization on update_splits"
  "[Cascade-Registry] Reuse Split Validation Logic in update_splits"
  "[Cascade-Registry] Emit Event on Splits Update"
  "[Cascade-Registry] Consistent ProjectNotFound Handling Across Public Functions"
  "[Cascade-Registry] Unit Tests — register_project Success Path"
  "[Cascade-Registry] Unit Tests — register_project Failure Paths"
  "[Cascade-Registry] Unit Tests — get_project and has_project"
  "[Cascade-Registry] Unit Tests — update_splits Success and Failure"
  "[Cascade-Registry] Unit Tests — Authorization Failures"
  "[Cascade-Registry] Refactor lib.rs into Modules"
  "[Cascade-Registry] Add WASM Build Optimization Script"
  "[Cascade-Registry] Integration Test — Multi-Account Registration and Update Flow"
  "[Cascade-Registry] Deploy Registry Contract to Stellar Testnet"
  "[Cascade-Registry] Document Registry Contract Usage in README"
)

# issue-06.md through issue-30.md, matching TITLES order above
START_NUM=6
END_NUM=30

i=0
for num in $(seq $START_NUM $END_NUM); do
  padded=$(printf "%02d" "$num")
  body_file="$DIR/issue-${padded}.md"
  title="${TITLES[$i]}"

  if [[ ! -f "$body_file" ]]; then
    echo "ERROR: missing body file $body_file — stopping so numbering doesn't drift." >&2
    exit 1
  fi

  echo "Creating issue #$num: $title"
  gh issue create \
    --repo "$REPO" \
    --title "$title" \
    --body-file "$body_file" \
    --label "$LABEL"

  # Small delay to avoid hitting secondary rate limits when creating many issues back to back
  sleep 2
  i=$((i + 1))
done

echo "Done. Created issues #$START_NUM through #$END_NUM in $REPO."
echo "Note: GitHub assigns issue numbers automatically based on repo history —"
echo "if the repo already has existing issues/PRs, these may not land exactly as #6-#30."
echo "Verify the numbers gh reports match your expected sequence before wiring up cross-references."
