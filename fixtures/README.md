# Fixtures: Lantern Notes

A made-up product, a shared notebook for small teams, written for this repository. Nothing here can be bought. The held-out
check runs a jury on it and compares what the jury finds with the answer key below, so do not edit the site without
editing the key.

| path | what it is |
|---|---|
| `site/index.html`, `site/pricing.html`, `site/style.css` | the site (the subject) |
| `site-walk/home.walk.json`, `site-walk/pricing.walk.json` | the ARIA walks of the two pages (hand-written to match the HTML) |
| `site-walk/home.png`, `site-walk/pricing.png` | 1280x800 screenshots, rendered with headless Chrome from `site/` |
| `comparators/acme-pricing/` | the comparator, "Acme Pads": its page, walk and screenshot |
| `personas/*.json` | seven personas: 3 `builder`, 2 `company`, 1 `newcomer`, 1 `investor` |
| `lenses/site.toml` | the detector's `site` lens: the six criteria of a hand-scored jury table the maintainer keeps privately, on 1–10, anchors at 1, 4, 7, 10; loaded with `lens add`, never seeded |
| `jurygraph.toml` | the trigger rules of the fixture project; `jurygraph-bad.toml` is refused on purpose (`money` under `low`) |

Walk convention: every id is unique within a file; a link's `name` carries its target in parentheses, e.g.
`Sign up free (#)`, because the walk has no field for a link's destination.

## Answer key: the defects planted in the site

| # | defect | where | visible in the walk | visible in the PNG |
|---|---|---|---|---|
| D1 | The price of the Team plan is hidden behind "Contact us"; the other two plans show a price | `pricing.html`, Team card | `price-team` is "Contact us" next to `price-solo` "$0 forever" and `price-studio` "$24 per person, per month" | the Team card shows "Contact us" where the others show a number |
| D2 | The primary button on the home has low contrast: white text on pale blue (about 2.1:1; WCAG AA asks 4.5:1) | `index.html`, "Get started" | `btn-start` (the walk records no colour; a jury sees this only in the PNG) | the pale button beside the dark secondary one |
| D3 | The home hero is jargon: "Orchestrate asynchronous knowledge synergies across your team's cognitive workflows" and "AI-native collaborative substrate that operationalizes tacit insight at scale"; it never says it is a notebook | `index.html`, hero | `h1` and `hero-text` | the headline |
| D4 | The signup link on the pricing page points to `#`: it goes nowhere | `pricing.html`, Solo card | `btn-signup` is "Sign up free (#)" | not visible in the PNG (a link target is not drawn) |

Not planted, and so not findings: the Studio plan's "Talk to sales" button next to a visible price, the "See how it works"
link (`#features`, which does exist), and the footer note that the product is made up.

A jury that finds D1, D3 and D4 from the walk and D2 from the PNG has read both attachments. A `seen` line about D2 that
cites only a walk node is suspect: the walk carries no colour.
