"""Run one JuryGraph jury with the Claude Agent SDK (D23). Not run by the tests: it needs the SDK and a key.

    pip install claude-agent-sdk            # check the SDK version: the option names below are from its Python docs
    python examples/agent-sdk/run_jury.py --subject home --rule public-surface --model claude-sonnet-5-5

JuryGraph calls no model. This script is the agent: it asks the engine for the briefs, runs ONE fresh SDK session per
brief (one context per juror), has each juror write its verdict to a private folder, then a coordinator session ingests
the verdicts through the `jurygraph mcp` server and asks `matrix` and `apply-check`. The SDK finds its own credentials; this
file reads no environment and holds no key.
"""
import argparse
import asyncio
import json
import subprocess
import tempfile
from pathlib import Path

from claude_agent_sdk import ClaudeAgentOptions, ResultMessage, query

# `jurygraph mcp` over stdio: the tools the coordinator may call, and nothing else
JURYGRAPH_MCP = {"jurygraph": {"type": "stdio", "command": "jurygraph", "args": ["mcp"]}}
COORDINATOR_TOOLS = [
    "Read",
    "mcp__jurygraph__jurygraph_verdict",
    "mcp__jurygraph__jurygraph_matrix",
    "mcp__jurygraph__jurygraph_apply_check",
    "mcp__jurygraph__jurygraph_summary",
]
WAVE = 8  # jurors in flight at once: a large jury never floods the rate limit


def jurygraph(*args: str) -> dict:
    """One engine call (the CLI prints one JSON envelope); the engine never calls a model."""
    out = subprocess.run(["jurygraph", *args], capture_output=True, text=True, check=False)
    env = json.loads(out.stdout)
    if not env["ok"]:
        raise SystemExit(f"jurygraph {' '.join(args)}: {env['error']['code']}: {env['error']['message']}")
    return env["data"]


async def final_text(prompt: str, options: ClaudeAgentOptions) -> str:
    """Run one session to its end and return its final result text."""
    text = ""
    async for message in query(prompt=prompt, options=options):
        if isinstance(message, ResultMessage):
            text = message.result or ""
    return text


async def juror(persona: str, brief: str, private_dir: Path, model: str) -> Path:
    """One persona = one session = one context. It reads its brief and the files the brief lists, and writes one file."""
    verdict = private_dir / f"{persona}.json"
    options = ClaudeAgentOptions(
        model=model,
        cwd=str(private_dir),  # a private folder: the juror never sees another juror's verdict
        allowed_tools=["Read", "Write"],
        permission_mode="acceptEdits",
        max_turns=30,
    )
    await final_text(
        f"Read {brief} and open only the files it lists, by full path. Do not list or glob any folder. "
        f"Write exactly one verdict, in the JSON shape the brief shows, to {verdict}. Reply 'done'.",
        options,
    )
    return verdict


async def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--subject", required=True)
    ap.add_argument("--rule", required=True, help="a rule class of jurygraph.toml")
    ap.add_argument("--model", required=True)
    ap.add_argument("--lens", default=None, help="one lens (default: the rule's first)")
    args = ap.parse_args()

    brief_args = ["brief", "--subject", args.subject, "--rule", args.rule, "--model", args.model]
    if args.lens:
        brief_args += ["--lens", args.lens]
    plan = jurygraph(*brief_args)
    run = plan["run"]
    private = Path(tempfile.mkdtemp(prefix=f"jury-{run}-"))  # outside the repo: verdicts are ingested, not committed

    # one session per brief, a wave at a time
    briefs = [(b["persona"], str(Path(b["path"]).resolve())) for b in plan["briefs"]]
    files: list[Path] = []
    for i in range(0, len(briefs), WAVE):
        files += await asyncio.gather(*(juror(p, b, private, args.model) for p, b in briefs[i : i + WAVE]))

    # the coordinator ingests through MCP: a refused verdict goes back to its juror, never edited here
    names = ", ".join(str(f) for f in files)
    coordinator = ClaudeAgentOptions(
        model=args.model,
        mcp_servers=JURYGRAPH_MCP,
        allowed_tools=COORDINATOR_TOOLS,
        max_turns=60,
    )
    print(
        await final_text(
            f"For run {run}: read each verdict file ({names}) and call jurygraph_verdict with run={run} and its content. "
            f"If one is refused, report the refusal and stop; do not change the evidence. Then call jurygraph_matrix "
            f"for run {run}, then jurygraph_apply_check for run {run}, then jurygraph_summary for run {run}. "
            "Show the summary text as it is, then the apply-check answer.",
            coordinator,
        )
    )


if __name__ == "__main__":
    asyncio.run(main())
