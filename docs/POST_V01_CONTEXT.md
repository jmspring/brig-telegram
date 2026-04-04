# Brig v0.1.0 — Project Context for Continuing Development

Paste this into Claude Code after /clear when starting post-v0.1 work.
It provides the full design context that shaped the project.

---

Read CLAUDE.md and docs/DEVELOPMENT_SESSIONS.md for the full
project specification and phase prompts.

Here is additional architectural context from the original design
sessions that shaped Brig. This is important background for any
work going forward.

## What Brig Is

Brig is a FreeBSD-native LLM-driven task execution tool. The
operator describes tasks in natural language, an LLM decomposes
them into tool calls, and each tool call executes in an isolated
FreeBSD jail. It is v0.1.0, tagged and released.

Brig is a PROGRAM, not a platform. One binary. BSD philosophy:
flat config (key=value like rc.conf), man pages in mdoc(7), rc.d
scripts, syslog logging, minimal dependencies (10 crates), no
async runtime, synchronous control flow, composability with Unix
tools. Output works with pipes, grep, mail.

The name comes from a ship's brig (naval jail).

## Design Philosophy (Non-Negotiable)

These principles were established through extensive design review
and must not be violated:

- The OPERATOR is in control. The agent proposes, never acts
  autonomously. Agent-proposed skills go to ~/.brig/proposals/,
  not the registry. Default auto_approve_skills=false.

- Use OS PRIMITIVES. Jails, ZFS, pf, rctl, cron, syslog, kqueue.
  Don't reimplement what FreeBSD provides. Scheduling uses cron(8).
  Logging uses syslog. Service management uses rc.d.

- MINIMAL dependencies. 10 crates, each justified. No nix crate
  (we shell out to commands instead — more debuggable, matches
  what the operator would type, man pages document jail(8) not
  jail_set(2)). No tokio/async-std. No clap. No framework.

- SYNCHRONOUS control flow. Single-threaded agent loop. Visible
  control flow. A developer can trace a request from input to
  jail creation to output by reading the code linearly.

- FLAT config. key=value like rc.conf. Not TOML, not YAML, not
  JSON for system config. Skill manifests use TOML because they
  have nested structure.

- ONE binary. brig does everything. Gateways (Telegram, Discord,
  Slack) are SEPARATE programs in separate repos that connect via
  the unix socket protocol.

- Shell out via std::process::Command with .arg() per argument.
  Never string interpolation through a shell. Validate all names
  against [a-zA-Z0-9_-]{1,63} before passing to commands.

## Architecture

Two kinds of callable tools (the LLM doesn't know the difference):

1. COMMANDS — execute in the supervisor process. Fast, privileged.
   Can modify supervisor state. memory_add, memory_search,
   memory_remove, llm_config, schedule_add.

2. SKILLS — execute in ephemeral FreeBSD jails. Sandboxed by jail
   isolation, pf network policy, rctl resource limits. User-
   installable, potentially untrusted. shell, code, fetch, file.

Skills declare INTENT (network=["github.com"]), brig translates
to MECHANISM (pf anchor rules). Skill authors don't write jail.conf
or pf rules — they say what they need, brig figures out how.

Persistent skills (kind="persistent") run in long-lived jails with
rc.d scripts. They don't register tools with the LLM — they're
infrastructure (gateways, deployed services) that calls brig, not
the other way around.

## Jail Hardening

Every generated jail.conf includes:
- exec.clean (no env leakage)
- enforce_statfs=2 (hide host mount table)
- children.max=0 (no nested jails)
- allow.raw_sockets=0, allow.chflags=0
- sysvmsg/sysvsem/sysvshm=new (per-jail IPC namespaces)
- devfs_ruleset=100 (restrictive device access)

EpairPool tracks system-wide epair interface allocation (0-255).
Orphan cleanup on startup scans jls for stale brig-w-* jails.
Operation timeouts prevent blocking on degraded ZFS.
Pre-task ZFS snapshots enable rollback on failure.

## Memory System

SQLite + FTS5. Three tables: facts (explicit knowledge with
supersession model — old facts get superseded_by pointing to
corrections), sessions (metadata + summaries), messages (full
transcripts, FTS5 indexed for recall search).

Memory is injected into the system prompt — top 20 facts by
recency, ~800 token budget.

## Socket Protocol

Newline-delimited JSON over unix domain socket. Documented in
man/brig-protocol.7. Gateway jails access the socket via nullfs
mount of ~/.brig/sock/. Each gateway gets limited capabilities
(submit_task, read_status). Only CLI has Admin.

## Metacharacter Handling

The shell skill's command parameter IS a shell command — pipes,
redirects, semicolons are legitimate. Metacharacter validation
only applies to values substituted into {{param}} placeholders
in multi-param command templates, NOT to parameters where the
entire template is just "{{command}}".

## Running on FreeBSD

Brig needs root for jail operations. Build as user, run as root:
  cargo build && sudo -E ./target/debug/brig run "task"

The -c flag specifies the config file path — needed when running
as root to use the operator's config:
  sudo ./target/debug/brig -c /home/jims/.brig/brig.conf run "task"

ZFS datasets: zroot/brig (root), zroot/brig/jails (ephemeral),
zroot/brig/persistent (services), zroot/brig/template (base jail
template with @current snapshot).

## Current Status: v0.1.0

All of these are implemented and working:
- CLI (REPL, one-shot, daemon mode, all subcommands)
- Config parser (flat key=value)
- Agent loop (synchronous, command/skill dispatch)
- LLM client (Anthropic API + llama.cpp)
- Memory system (facts, sessions, FTS5 recall, supersession)
- Skill registry (multi-directory with precedence)
- Declarative skill execution (template substitution)
- Real jail execution on FreeBSD (ZFS clone, hardened jail.conf,
  pf anchors, rctl limits, jexec, cleanup)
- Orphan jail cleanup
- EpairPool management
- Secret management
- Persistent skill lifecycle (rc.d generation)
- Skill proposals (agent proposes, operator approves)
- Scheduling (cron integration)
- IPC socket server (daemon mode)
- Secret redaction
- Doctor health checks
- Man pages (brig.1, brig.conf.5, brig-skill.5, brig-protocol.7)
- Unit tests for all modules

The project compiles on any platform; FreeBSD-specific code uses
`#[cfg(target_os = "freebsd")]` with stubs elsewhere.

## What's Next

Post-v0.1 phases are in docs/DEVELOPMENT_SESSIONS.md, phases 10-20.
The recommended order:

In brig repo (~/code/brig-dev/brig/):
  Phase 11: Persistent task jails (multi-step tasks share one jail)
  Phase 12: Session summarization (LLM summary at session end)
  Phase 13: Context compression (handle long conversations)
  Phase 14: Socket authentication (pre-shared key handshake)
  Phase 15: brig skill test (invoke skills without LLM)
  Phase 17: Warm jail pool (pre-created jails for lower latency)
  Phase 18: DTrace probes (USDT probes at lifecycle points)
  Phase 19: Capsicum inside workers (cap_enter after init)

Separate repos (~/code/brig-dev/<name>/):
  Phase 10: brig-telegram (first gateway, proves the pattern)
  Phase 16: brig-discord (second gateway, adds websocket)
  Phase 20: brig-port (FreeBSD port Makefile for pkg install)

## Environment

- FreeBSD 15.0-RELEASE on "scratchy" (Minisforum DMAF5, AMD Ryzen 5)
- ZFS on NVMe (zroot) + 4TB SSD (zdata)
- Local LLM: Qwen3.5-35B via llama.cpp on a separate machine
  (RTX 3090, Void Linux) at http://10.69.42.31:8080
- Anthropic API available as alternative (pay-per-use, not subscription)
- Repository: github.com/jmspring/brig

### Workspace Layout

All brig-related projects live under ~/code/brig-dev/:

```
~/code/brig-dev/
├── brig/              # main project (github.com/jmspring/brig)
├── brig-telegram/     # telegram gateway (separate repo)
├── brig-discord/      # discord gateway (separate repo)
├── brig-port/         # FreeBSD port files (separate repo)
└── ...                # future gateways (slack, irc, matrix)
```

Each project is its own git repo with its own CLAUDE.md. Run 
Claude Code from within each project directory, not from brig-dev/.
Standalone projects share no code with brig — gateways communicate
via the unix socket protocol documented in brig's man/brig-protocol.7.
The port builds brig from its GitHub release tarball.

## Code Style

- No unnecessary abstractions. Match > trait hierarchy.
- Comments explain *why*, not *what*.
- Error messages are actionable.
- No `unwrap()` in non-test code.
- `#[cfg(target_os = "freebsd")]` for platform code.
- Functions do one thing. Short enough to hold in your head.
- Modules are flat files in `src/`, not directories with `mod.rs`.
- Shell out via `std::process::Command` with `.arg()` per argument.
  Never interpolate into shell strings.

## Guard Rails

**Adding dependencies:** "No new dependencies. We have 10 crates.
Implement with stdlib and existing deps only."

**Adding async:** "No async. The agent loop is synchronous. Use
std::thread if you need concurrency."

**Over-abstracting:** "Stop. That adds unnecessary complexity. A match
statement is better than a trait hierarchy with two variants."

**Wrong architecture:** "Commands execute in-process. Skills execute
in jails. The LLM doesn't know the difference."

**Not updating CLAUDE.md:** "Before we commit, update CLAUDE.md to
reflect what we just built."

**AI-style commits:** "Write commit messages like a developer — 
specific, descriptive. Break into logical commits when changes cover
distinct areas. No 'phase N' labels, no 'update code', no boilerplate."

## Commit Style

All commits should read as if a human developer wrote them.
Specific, descriptive, broken into logical units.

Good:
  implement memory command handlers (add, search, remove)
  fix FTS5 query quoting for hyphenated search terms
  add persistent task jails for multi-step workflows
  telegram gateway: bridge bot messages to brig socket

Bad:
  phase 10: telegram gateway
  update code based on AI suggestions
  implement features
  various fixes and improvements

If a session touches multiple areas, make multiple commits:
  git add src/jail.rs
  git commit -m "fix metacharacter validation: allow pipes in shell_exec"
  git add tests/
  git commit -m "backfill unit tests for jail lifecycle"

Tell me which phase to work on and I'll provide the specific
prompt, or read docs/DEVELOPMENT_SESSIONS.md for the full prompt.
