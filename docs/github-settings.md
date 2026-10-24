# GitHub settings

For the maintainer. These are set by hand on github.com/massrelay-ai/jurypersona (Settings, then the About box on the repository
page); nothing in the repository applies them.

## Repository

- **Name:** `jurypersona` (organisation `massrelay-ai`).
- **Description** (the first sentence of the README):

  > JuryPersona: synthetic users as a jury. AI personas judge your page, docs, deck or change, and only verdicts with evidence count. Local, no model calls.

- **Website:** the documentation or leave empty until `jurypersona.com` is registered.
- **Topics** (ten):

  ```
  synthetic-users  ai-personas  persona-testing  llm-as-a-jury  focus-group
  ux-research      mcp          claude-code      codex           agents
  ```

- **Social preview:** upload `docs/assets/brand/social-preview.png` (1280 x 640) under Settings, then General, then Social preview.
- **Features:** Issues on, Discussions optional, Wikis off. Issue templates and the pull request template are in `.github/`.

## Before the first release

- Branch protection on `main`: require the `ci` workflow (`.github/workflows/ci.yml`) and a linear history.
- Actions: allow GitHub-authored actions and the ones the workflows pin; the release job needs `id-token: write` for the
  attestation and the registry login (`docs/dev/release.md`).
- Security: enable private vulnerability reporting (`SECURITY.md` points to it).
