---
name: ai-agent-code-quality-gates
description: >-
  When writing, reviewing, or auditing code in this repository, run the
  automated quality gates inside the change loop and follow the Binding
  section before the generic advice. Also applies when setting up or auditing
  linters, formatters, type and schema checks, tests, git hooks, CI, secret
  scanning, dependency bots, API breaking-change checks, architecture rules,
  or parallel-agent worktrees. Design-first where a spec exists, one task
  runner, baseline-then-burn-down for new gates.
---

# AI-Agent Code Quality Gates

## Binding for this repository

Read this section before the source notes below. `AGENTS.md`, `docs/TASKS.md`, and `docs/ARCHITECTURE.md` win if a later section conflicts with them.

The loop on every change: edit, run `make check`, read the failures, fix them, repeat. Claim the work is done only when `make check` exits 0. Formatting, lint, and spelling are gate failures. Review time goes to layer boundaries, the privacy invariant (no hidden trader address), and business rules.

1. **`make check` is the fast gate and the only task runner.** Judges, the README Quickstart, and task S-1 run Make. Do not add mise or a second runner.
2. **Interfaces are designed in `docs/ARCHITECTURE.md` before handlers.** Repetitive NEAR surface area comes from `near-sdk` and `near-contract-standards`, not a hand-rolled NFT. Add TypeSpec or OpenAPI codegen only when a task adds an HTTP API spec.
3. **Business logic stays in a layer that tests can call on its own.** Contract behavior lives in `contracts/alpha-keys`. Metrics live in `attestor/pnl-core` (pure: no I/O, no clock, no randomness). Copy decisions live in `copier` `policy.py` (pure). The gateway, the OutLayer job, and the web app call those layers.
4. **There is no SQL database.** Contract tests run the real WASM through `near-workspaces`. `pnl-core` and `policy.py` are pure functions. Hyperliquid and 1Click stay on fixtures or `dry: true`.
5. **Parallel work uses a git worktree.** Do not `git switch` or `git checkout` another branch in the main checkout. Do not start a second dev server on a port that is already taken. There is no per-worktree database to create.
6. **A new strict check starts from a baseline.** Record current violations so the check is green, fail only new violations, and remove baseline entries in later tasks.
7. **One tool per job.** Before adding a checker, compare it with the one this repo already uses and wire the winner into `make check` or CI. A suggested tool that is not executed is not a gate.
8. **Secrets stay out of the tree and out of logs.** The pre-commit scan in `AGENTS.md` is the secret gate until gitleaks from `docs/ci/ci.yml.example` is wired (task V-3).

Gates that run today:

| Check | Where |
|---|---|
| `cargo test --workspace`, `pnpm -r test`, `uv run pytest` | `make check` |
| Secret-pattern grep before a commit | `AGENTS.md` "Before opening a PR" |
| Prose architecture and privacy rules | `AGENTS.md`, `.cursor/rules/` |

Still prose, until a task wires them with a baseline: `cargo fmt`, `clippy`, `ruff` (already the copier linter, not yet invoked by `make check`), TypeScript typecheck, link and spell checks, commit messages, git hooks, dependency-update bot, architecture dependency rules, worktree setup commands. Do not turn a red strict gate on over the whole tree.

## Source notes

Source: Kirill Mokevnin, "Как контролировать код ИИ-агентов: Quality Gates, архитектура и инструменты разработчика" (Organized Programming podcast, https://www.youtube.com/watch?v=x-P_xC6No0I, 2026-11-01). Timestamps point into the video. Tool names were checked against github.com/mokevnin/dev-quality-gates. The notes below are the generic method. The Binding section is how this repository applies it.

## Core idea

- A quality gate is an automated check the agent runs inside its own loop: make a change, run the gates, read the errors, fix them, repeat. Build the gates for the agent.
- Rules in AGENTS.md or skills are recommendations. Only automated checks enforce them. Written architecture rules, once turned into a check, found hundreds of violations in a large codebase the author had been watching.
- Do not spend review time on style trivia. That is the linter's job. Human attention goes to architecture, layer boundaries, and business logic.
- Keep observability no matter how good the model is: error collectors, logs, tracing, metrics.

## 1. Generate instead of hand-writing boilerplate

Hand-written infrastructure blurs module boundaries into business logic.

1. Design first. In the source talk that means TypeSpec, then OpenAPI, then generators. In this repository the design record is `docs/ARCHITECTURE.md`.
2. Generate clients, validation, and handlers from the spec when the stack has a generator. Spec drift should fail at generation time.
3. Choose libraries by how far they support that generation, plus middleware and built-in tests.
4. Before hand-writing repetitive code, check whether a generator or an extension already covers it. Hand-written code is for business logic.

## 2. Enforce architecture layers explicitly

Agents do not put business logic in its own layer unless told. Each layer has one responsibility and can be tested without the layers above it. Missing layers scatter logic. Tests catch small bugs inside a layer; they do not catch a missing layer.

## 3. Adopt a curated quality-gate set

Use one tool per job. The talk's list is `mokevnin/dev-quality-gates`. Categories called out there: link checking (lychee), spelling (typos), Markdown lint that also lints fenced code, git hooks, commitlint, agent-config lint (agnix), Dockerfile lint, Dependabot or Renovate, secret scanning, frontend bundle budget, OpenAPI breaking-change checks (oasdiff), schema-vs-model checks, architecture dependency rules.

Before adding a tool, compare it with the current pick and record the comparison. Then wire it in. A list with no command is not adopted.

## 4. One toolchain

The talk uses mise for language pins, env, tasks, hooks, and dev processes. This repository already has one task entry point, `make check`, and the judge Quickstart depends on it. Extend that entry point. Do not install a second one.

Pin versions that affect builds (see `rust-toolchain.toml`, `packageManager`, and `requires-python`). Share dependency stores across worktrees: pnpm, the Cargo cache, and the uv cache.

## 5. Make the repo safe for parallel agents

Plan tasks one after another, run them in parallel, and review the result. Correcting an agent mid-run is the old way.

1. Never switch branches in the main checkout. Create a worktree.
2. Automate worktree setup and teardown once parallel work is real, and name those commands in `AGENTS.md`.
3. Give each worktree its own database only when this repo has a database. It does not.
4. Give each worktree its own ports when more than one dev server runs.
5. Limit heavy commands machine-wide when several worktrees run test suites at once. Wrap the command itself (`cargo test`, `pnpm test`, pytest), and leave quick commands alone. On a hook, a busy machine should wait and then run, not fail the hook.
6. Keep dependency caches outside the worktree.

## 6. Tests

Test the real boundary. For the contract, that is `near-workspaces` running the WASM. For `pnl-core` and `policy.py`, that is the pure function with fixtures. Do not mock those layers. External network calls stay on fixtures or `dry: true`, because tests and CI must not move mainnet funds.

A database-consistency linter applies only after a SQL schema exists.

## 7. Architecture rules as a gate

Declare layers by path, allowed dependencies, and exceptions. Prose in `.cursor/rules/` is the current form. A type-aware checker (the talk uses easement; dependency-cruiser is the usual JS option) replaces that prose only after it is wired, with a baseline if the tree is already red. A slow architecture check can run nightly while `make check` stays fast.

Procedure: take a written rule and ask whether a tool can check it. What remains is judgment.

## 8. Baseline, then burn down

When adding a gate to existing code:

1. Run it and record current violations in a baseline file, so the check passes and only new violations fail.
2. In later tasks, remove a few baseline entries and fix the code, until the baseline is empty.

## Checklist

- [ ] Binding section followed for this change
- [ ] `make check` run after the change and left green
- [ ] Repetitive code checked for an existing generator before writing it by hand
- [ ] Business logic left in the contract, `pnl-core`, or `policy.py`
- [ ] Interface change recorded in `docs/ARCHITECTURE.md` in the same change
- [ ] No branch switch in the main checkout
- [ ] Any new strict gate added with a baseline
- [ ] Secrets absent from the diff, logs, and fixtures

## Anti-patterns

- Generating infrastructure by hand when `near-sdk` or `near-contract-standards` already provides it
- Treating `AGENTS.md` prose as if it enforced architecture
- Review comments about formatting
- Several agents sharing one checkout via `git switch`
- A second task runner beside `make check`
- Turning on a strict gate with no baseline, so the tree stays red
- A tool named in a doc and never executed
