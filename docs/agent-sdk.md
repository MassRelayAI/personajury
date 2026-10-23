# Running a jury with the Claude Agent SDK

JuryGraph calls no model; your agent does. The Claude Agent SDK is a good agent for it: one SDK session is one context, so
one session per brief is one independent juror. `examples/agent-sdk/run_jury.py` is the whole recipe in about 90 lines
(Python, `claude-agent-sdk`). **Check the SDK version** (check the SDK version before you rely on a name): the option names (`mcp_servers`, `allowed_tools`, `permission_mode`,
`cwd`, `query`, `ResultMessage`) are those of the SDK's Python documentation when this was written, and the SDK moves.

## The shape
1. **Register `jurygraph mcp` as an MCP server** for the coordinator session:
   `mcp_servers={"jurygraph": {"type": "stdio", "command": "jurygraph", "args": ["mcp"]}}`, and allow only the tools it needs
   (`mcp__jurygraph__jurygraph_verdict`, `_matrix`, `_apply_check`, `_summary`).
2. **Brief** with the engine (`jurygraph brief --rule ... --model ...`; the example calls the CLI so it can fan out). The answer
   lists one brief file per persona.
3. **One agent per brief, one context per juror.** Each brief gets its own `query()` session, so no juror sees another's
   reasoning. Give each a **private** working folder (`cwd`), only `Read` and `Write`, and the brief's path; tell it to open
   the files the brief lists by full path and never to glob a shared folder. It writes one verdict JSON there. Run the
   sessions a wave at a time (the example uses 8).
4. **Ingest through MCP.** A coordinator session reads each verdict file and calls `jurygraph_verdict`. A refusal goes back
   to the juror; nobody edits the evidence to make it pass.
5. **Read**: `jurygraph_matrix` (per stratum, never averaged), `jurygraph_apply_check` (obey it), `jurygraph_summary`
   (show the text as it is).

Credentials: the SDK reads its own environment, as its documentation says. This repository holds no key, the example reads
no environment variable, and JuryGraph never reads one. The example is not run by the tests (it needs a key); a test checks it
compiles. If one process plays every juror it is not independent: brief with `--independence sequential` and the answer
will be `advise`, never `apply`.

`apply-check` is a verdict the engine cannot enforce by itself; wire it where it binds (see `docs/ci.md`).
