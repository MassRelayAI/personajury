#!/usr/bin/env bash
# The fixture jury, setup half: builds a workspace and composes the 14 briefs (7 personas x critical, compare).
# Usage: fixtures/run/setup.sh <personajury binary> <workspace> <repo root>. tests/fixture_replay.rs does the same in Rust.
set -euo pipefail
J="$1"; W="$2"; R="$3"
j() { "$J" --root "$W" "$@"; }
j trait install >/dev/null
for p in "$R"/fixtures/personas/*.json; do j persona add --file "$p" >/dev/null; done
j panel add fixture --query "3 builder, 2 company, 2 others" >/dev/null
j subject add lantern-site --kind page --attach "$R"/fixtures/site-walk/home.walk.json "$R"/fixtures/site-walk/home.png "$R"/fixtures/site-walk/pricing.walk.json "$R"/fixtures/site-walk/pricing.png >/dev/null
j comparator add acme-pricing --kind prices --attach "$R"/fixtures/comparators/acme-pricing/pricing.walk.json "$R"/fixtures/comparators/acme-pricing/pricing.png >/dev/null
STAKES="builder-ana=primary,builder-mei=primary,builder-ravi=primary,newcomer-sofia=primary,company-george=secondary,company-hanna=secondary,investor-claire=bystander"
j brief --subject lantern-site --panel fixture --lens critical --stake "$STAKES" --spend 7 --model claude-sonnet-5-5
j brief --subject lantern-site --panel fixture --lens compare --stake "$STAKES" --spend 7 --model claude-sonnet-5-5 --comparator acme-pricing
