<!-- adapted from comparegraph SECURITY.md -->
# Security policy

PersonaJury reads files you attach and writes a local SQLite store. It makes no network call and never calls a model; the
personas are run by your own agent. A report is in scope when PersonaJury can be made to execute code from an attached
file, read or write outside the workspace, hang or exhaust memory on a crafted input (a verdict, a walk, a diff, a
`personajury.toml`), accept a verdict the rules say it must refuse, or when a dependency it ships has a known vulnerability.

## Reporting

Write to **support@massrelay.ai** or open a private security advisory on this repository (Security tab, "Report a
vulnerability"). Include the version (`personajury --version`), the platform, and a minimal input. You will get an
acknowledgement within seven days and a fix or a reasoned decision within ninety. Please do not open a public issue for
a vulnerability before a fix is released.

## Supported versions

The latest release on `main` and the latest tagged version. Older tags are not patched.

## Privacy

- **The engine sends nothing anywhere.** It makes no network call and never calls a model; `deny.toml` bans HTTP client
  crates and `tests/guards.rs` checks the lock file and the source for a key read.
- **Attachments may hold your customers' data.** An ARIA walk, a screenshot or a diff of a real product can contain names,
  emails, prices or tokens that belong to other people. Attachments are copied into `.personajury/blobs` inside the workspace
  and stay there with the rest of `.personajury/` (the store, `.personajury/store.db`). Treat that directory as local state: keep
  it out of version control and out of what you share, and check an attachment before you attach it.
- **Your agent's model provider sees whatever your agent sends it.** The briefs and the attachments they list are read by
  your agent's subagents, and what they send to their provider is the agent's choice and the agent's provider terms, not the
  engine's. If an attachment must not leave the machine, do not give it to an agent that sends it away.
- The fixtures in this repository are our own made-up site; they contain no personal data.
