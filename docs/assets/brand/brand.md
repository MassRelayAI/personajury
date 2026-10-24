# PersonaJury brand

The mark is a verdict check drawn as a graph: two nodes joined by two edges, with the long edge ending in a ring,
the evidence the verdict cites. It was chosen by a persona jury, 7 of 7 in round 2 (`../jury/matrix-r2.md`).

## Files

| file | use |
|---|---|
| `mark.svg` | square mark on light backgrounds |
| `mark-dark.svg` | square mark on dark backgrounds |
| `lockup.svg` / `lockup-dark.svg` | mark + outlined wordmark, 305 x 64, on light / dark |
| `favicon.svg` | favicon; switches to the dark colours under `prefers-color-scheme: dark` |
| `favicon-16.png`, `favicon-32.png` | raster favicons, transparent corners |
| `social-preview.svg` / `.png` | GitHub repository card, 1280 x 640 |

## Palette

Taken from the README figures. Never more than three colours in the mark.

| role | light | dark |
|---|---|---|
| tile | `#0550ae` blue | `#ddf4ff` sky |
| edges and nodes | `#ddf4ff` sky | `#0550ae` blue |
| evidence ring | `#fff8c5` butter | `#1f2328` ink |
| wordmark | `#1f2328` ink | `#ddf4ff` sky |

Supporting colours (social preview, cards only): `#57606a` muted text, `#d0d7de` card border, `#dafbe1` mint.

## Dark and light rule

- Light backgrounds (white, `#f6f8fa`, anything lighter than mid-grey): `mark.svg`, `lockup.svg`.
- Dark backgrounds (`#0d1117`, `#161b22`, anything darker than mid-grey): `mark-dark.svg`, `lockup-dark.svg`.
- The square mark works on either background, but the lockup does not: ink text vanishes on dark and sky text vanishes
  on white. Match the lockup to the background every time.
- For a GitHub README, use `<picture>` with `media="(prefers-color-scheme: dark)"` to pick the variant.
- Don't recolour, outline, add a shadow to, rotate or stretch the mark, and don't put the blue tile on a blue background.

## Minimum size

- Mark: 16 px. It was drawn on a 64-unit grid (4 units per pixel at 16 px) and checked at 16 px. Below 16 px, use nothing.
- Lockup: 24 px tall (about 100 px wide). Below that, use the mark alone.

## Clear space

Keep clear space of a quarter of the mark's height on every side: 16 units at the native 64 (16 px around a 64 px
mark). In the lockup, the gap between the tile and the wordmark is fixed at 16 units. Don't change it.

## Wordmark

"PersonaJury" is one word in camel case: "Persona" in Noto Sans Bold, "Jury" in Noto Sans Regular, outlined to paths
(see `NOTICE.md`). Elsewhere, in running text, write PersonaJury in the surrounding typeface without styling.
