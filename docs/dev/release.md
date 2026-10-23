<!-- adapted from comparegraph docs/dev/release.md -->
# Cutting a release

Maintainers only. A release is a tag; the workflow in `.github/workflows/release.yml` does the rest.

## Before the tag

1. The version is `[workspace.package] version` in `Cargo.toml`; `Cargo.lock` (the `jurygraph` entry), `server.json`,
   `.claude-plugin/plugin.json` and `.claude-plugin/marketplace.json` repeat it. CI checks `server.json`. Bump them in the
   commit that will be tagged; the build jobs exit 4 when the tag disagrees with `Cargo.toml`.
2. Date the section in `CHANGELOG.md` (`## [x.y.z] - YYYY-MM-DD`) and leave an empty `## [Unreleased]` above it.
3. `cargo test -p jurygraph` and `cargo clippy -p jurygraph --tests -- -D warnings` green; the `ci` workflow green on
   `main` for that commit.

## The tag

```bash
git tag -a vX.Y.Z -m "vX.Y.Z"
git push origin vX.Y.Z
```

## What the workflow does

- `build`: five jobs, one per platform (linux-x64, linux-arm64, darwin-arm64, darwin-x64, win32-x64), each on a runner of
  that platform. Each checks the version, builds with `--locked`, strips the Linux binaries, and uploads the binary with
  its `.sha256`.
- `release`: downloads the five binaries, writes `SHA256SUMS`, checks it, publishes a GitHub release with the binaries,
  the sums and generated notes, and signs a build-provenance attestation over them.

There is no npm package and no MCP Registry publish job in v0.1; `server.json` is kept in step so a registry entry can be
added later. The install script (`scripts/install.sh`) reads `releases/latest/download/`, so it serves the new version as
soon as the release exists.

## Attestations

Verify a download with the GitHub CLI (2.49 or later):

```bash
gh attestation verify jurygraph-linux-x64 -R massrelay-ai/jurygraph
```

## CI audit

`deny.toml` drives the `deny` job in `ci.yml` (`cargo deny --locked check`): licence allowlist, advisories, bans (every
HTTP client crate) and sources. An `ignore` entry in `[advisories]` carries its reason.

## After the release

On a clean machine:

```bash
curl -fsSL https://raw.githubusercontent.com/massrelay-ai/jurygraph/main/scripts/install.sh | sh
jurygraph --version
```

In Claude Code: `/plugin install jurygraph --marketplace massrelay-ai/jurygraph`, then `jurygraph doctor` in a workspace.
