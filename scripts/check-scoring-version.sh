#!/usr/bin/env bash
# check-scoring-version.sh — verifies that SCORING_VERSION in version.rs matches
# the golden fixture metadata. Run in CI via: make scoring-version-check.
#
# Exits non-zero if the version is changed without updating the golden fixture file.
set -euo pipefail

SCORING_VERSION=$(grep -m1 'pub const SCORING_VERSION' core/heeczer-core/src/version.rs \
    | sed 's/.*"\(.*\)".*/\1/')

GOLDEN_FILE="core/schema/fixtures/golden/score_result.json"

if [[ ! -f "$GOLDEN_FILE" ]]; then
    echo "ERROR: golden fixture not found at $GOLDEN_FILE"
    exit 1
fi

GOLDEN_VERSION=$(python3 -c "import json,sys; d=json.load(open('$GOLDEN_FILE')); print(d.get('scoring_version',''))" 2>/dev/null || true)

if [[ "$SCORING_VERSION" != "$GOLDEN_VERSION" ]]; then
    echo "ERROR: SCORING_VERSION mismatch!"
    echo "  version.rs: $SCORING_VERSION"
    echo "  $GOLDEN_FILE: $GOLDEN_VERSION"
    echo ""
    echo "When bumping SCORING_VERSION you MUST also:"
    echo "  1. Regenerate the golden fixture: cargo test -p heeczer-core --test golden_scoring -- --nocapture"
    echo "  2. Commit both changes together."
    exit 1
fi

echo "SCORING_VERSION check OK: $SCORING_VERSION matches golden fixture"
