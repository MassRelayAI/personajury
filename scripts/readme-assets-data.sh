#!/usr/bin/env bash
# Writes docs/assets/spend.json: the bytes of the briefs for --spend 1, 3, 7 x --detail compact, full on the critical lens,
# measured by running the binary on the fixture workspace the way fixtures/run/setup.sh does. Deterministic.
# Usage: scripts/readme-assets-data.sh <jurypersona binary>     (strata.json and matrix.json are derived by readme-assets.mjs --derive)
set -euo pipefail
J="${1:?usage: readme-assets-data.sh <jurypersona binary>}"
R="$(cd "$(dirname "$0")/.." && pwd)"
W="$(mktemp -d)"; trap 'rm -rf "$W"' EXIT
j() { "$J" --root "$W" "$@"; }
for p in "$R"/fixtures/personas/*.json; do j persona add --file "$p" >/dev/null; done
j panel add fixture --query "3 builder, 2 company, 2 others" >/dev/null
j subject add lantern-site --kind page --attach "$R"/fixtures/site-walk/home.walk.json "$R"/fixtures/site-walk/home.png "$R"/fixtures/site-walk/pricing.walk.json "$R"/fixtures/site-walk/pricing.png >/dev/null
STAKES="builder-ana=primary,builder-mei=primary,builder-ravi=primary,newcomer-sofia=primary,company-george=secondary,company-hanna=secondary,investor-claire=bystander"
OUT="$W/runs.jsonl"
for spend in 1 3 7; do for detail in compact full; do
  j brief --subject lantern-site --panel fixture --lens critical --stake "$STAKES" --spend "$spend" --detail "$detail" --model claude-sonnet-5-5 \
    | node -e 'const d=JSON.parse(require("fs").readFileSync(0,"utf8")).data;
      console.log(JSON.stringify({spend:d.spend,detail:d.detail,briefs:d.briefs.length,bytes:d.briefs.reduce((a,b)=>a+b.bytes,0)}))' >> "$OUT"
done; done
TOK="$(sed -n 's/^tokens_per_brief *= *\([0-9]*\).*/\1/p' "$R/fixtures/jurypersona.toml")"
node -e '
const rows=require("fs").readFileSync(process.argv[1],"utf8").trim().split("\n").map(JSON.parse);
const out={title:"What a jury costs the agent: brief bytes by spend and detail",
 command:"jurypersona brief --subject lantern-site --panel fixture --lens critical --stake <seven stakes> --spend N --detail compact|full --model claude-sonnet-5-5, on the workspace fixtures/run/setup.sh builds; the bytes of every brief of the run, summed",
 tokens_per_brief:+process.argv[2],
 caption:"Two knobs: how many, and how much each carries. The token figure is the project toml’s own estimate, not a measurement.",
 rows};
require("fs").writeFileSync(process.argv[3],JSON.stringify(out,null,1)+"\n");
' "$OUT" "$TOK" "$R/docs/assets/spend.json"
