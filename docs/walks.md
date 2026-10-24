# Page walks: `tools/walk.mjs`

A walk is the ARIA tree of a page with stable node ids, the attachment a `seen` line cites (`pricing.walk.json#price-team`).
`tools/walk.mjs` captures one. It is a helper you run before `subject add`; the engine never runs it, never calls it, and
stays offline. No npm dependency: Node >= 22 (global `WebSocket`) and a local Chrome or Chromium.

```
node tools/walk.mjs --out shots/ pricing=https://example.com/pricing home=file:///site/index.html
node tools/walk.mjs --out shots/ --phone pricing=site/pricing.html     # a bare path becomes file://
```

Per page `<name>`: `<name>.walk.json` and `<name>.png` (1280x800, full height). `--phone` adds `<name>.phone.walk.json` and
`<name>.phone.png` at 390x844 (mobile emulation). Options: `--chrome PATH` (else `CHROME_BIN`, then the usual install paths),
`--cdp ws://host:port/devtools/browser/<id>` to attach to a running browser instead of launching one. No Chrome: a message and
exit 2.

## Format

Exactly the shape of `fixtures/site-walk/*.walk.json`: `{ "url", "viewport": [w, h], "nodes": [ { "id", "role", "name",
"children"? } ] }`. Links carry their `href` as written in the page, appended to the name: `Pricing (pricing.html)`.
Paragraphs, list items and cells that have no accessible name are named by their text.

## Node ids

`<role>-<slug(name)>`, with the name without the href; slug = ASCII-folded, lower case, runs of non-alphanumerics become `-`,
at most 40 characters. A node with no name is just `<role>` (`article`, `list`). Repeats are numbered `-2`, `-3` in document
order (`link-more`, `link-more-2`). Ignored and generic nodes (`generic`, `none`, text runs, list markers) are dropped and
their children move up. The same page gives the same ids and a byte-identical walk (tested on `fixtures/site`, twice).

Ids change when the visible name changes, which is the point: a cited id that no longer exists means the page moved. The
hand-written ids in `fixtures/site-walk` (`price-team`, `plan-solo`) predate the tool; a captured walk of the same page names
the same nodes `paragraph-contact-us`, `article`. Re-cite from the captured walk when you attach a real subject.

## Privacy

A walk is the page's text, including anything a signed-in session shows. Keep walks and screenshots of private pages out of
git (add the output folder to `.gitignore`); share only what you would publish.

## Test

`node --test tools/test/walk.test.mjs` captures `fixtures/site/` over `file://`, checks that every node the fixture verdicts
cite exists (matched by role, name and named descendants against the recorded walk), and that two captures are identical. The
capture tests skip with a message when no Chrome or no global `WebSocket` is present. Four cited ids belong to the comparator
snapshot (`fixtures/comparators/acme-pricing`), not the site, and are asserted as such.
