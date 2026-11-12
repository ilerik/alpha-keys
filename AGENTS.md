# Guidance for coding agents (Cursor, Claude Code, Codex)

Read this file, then PLAN.md, docs/ARCHITECTURE.md, and the task you were given from docs/TASKS.md. Work on **one task per PR**.

## Ground rules
1. **Never commit secrets.** No private keys, seed phrases, API keys or JWTs, HL addresses of the hidden trader account, or `.env` files. Only `.env.example` with variable **names** and public testnet values. If you need a new variable, add it to `.env.example` in the same PR.
2. **Never print secrets.** No logging of env values, request headers with keys, or the trader's HL address. Scripts read secrets from env and never echo them.
3. **No mainnet funds in tests or CI.** Tests use fixtures, mocks, `near-workspaces` sandbox, or NEAR testnet.
4. **1Click is `dry: true` by default.** A live quote needs an explicit `ONECLICK_LIVE=1` and is author-run only. Never automate deposits.
5. **Tests required.** Every PR adds or updates tests for what it changes, and `make check` must pass.
6. **Small PRs.** About 400 changed lines or fewer, excluding lockfiles and fixtures. One concern per PR. Title format: `[TASK-ID] short description`.
7. **Keep the README Quickstart runnable.** If you change a command, port, env var, or contract method that the Quickstart uses, update README.md in the same PR. Each step is "do X, see Y".
8. **Judge reproducibility.** Everything a judge runs must work from a fresh clone with no secrets and no funds. Pin versions; commit lockfiles.
9. **Interfaces live in docs/ARCHITECTURE.md.** Change the doc in the same PR as the interface.
10. **Disclose baseline code.** Code reused from before 2026-10-19 goes under `vendor/` with a `baseline:` header and gets listed in the README.
11. **Don't use Shade Agents** (deprecated). Use OutLayer for TEE work.
12. **Privacy invariant:** the hidden HL address exists only as an OutLayer secret (and in Ilya's local `.env`). No component output, log, fixture, or UI may contain it. CI leak test H-1 enforces this.

## Stack (see ARCHITECTURE.md §1 for versions and links)
- Contracts: Rust, `near-sdk` 5.30, `near-contract-standards` 5.30, `cargo-near`, `near-workspaces` for sandbox tests. Testnet first.
- TEE job: Rust → `wasm32-wasip2`, `outlayer` crate; logic lives in `attestor/pnl-core` (pure, native-testable).
- Gateway: TypeScript, Node 22+, Hono, `near-api-js` 7.x, vitest.
- Copier: Python 3.11+, `uv`, pytest, ruff; pure `policy.py`.
- Web: Next.js + TypeScript, `@hot-labs/near-connect`, `near-api-js` 7.x.
- Package managers: cargo workspace at the root, pnpm workspace (`gateway`, `web`), uv for `copier`.

## Commands
- `make check`: the fast quality gate (tests, and linters once they are wired). Run it after every change and fix failures before calling the work done.
- `make demo-seed`: offline simulation (after H-3).
- `cargo near build non-reproducible-wasm` (in `contracts/alpha-keys`).

## Quality gates
Adopted from `.cursor/skills/ai-agent-code-quality-gates/SKILL.md`. Follow that skill's Binding section.

1. `make check` is the only task runner. Do not add a second one.
2. Formatting and lint are gate failures, not review comments. A new strict check starts from a baseline of current violations and fails only new ones.
3. Business logic stays in the contract, `attestor/pnl-core`, and `copier` policy. Other packages call those layers.
4. Design the interface in `docs/ARCHITECTURE.md` before the handler. Use `near-sdk` and `near-contract-standards` instead of hand-rolled NEAR boilerplate.
5. There is no SQL database. Contract tests use `near-workspaces`. External calls stay on fixtures or `dry: true`.
6. Never `git switch` or `git checkout` another branch in the main checkout. Parallel work uses a git worktree.

## Before opening a PR
- [ ] `make check` passes
- [ ] `git diff --cached | grep -nEi '(secret|private_key|seed|mnemonic|ed25519:[1-9A-HJ-NP-Za-km-z]{40,}|0x[0-9a-f]{64})'` finds nothing real
- [ ] README/ARCHITECTURE updated if needed
- [ ] PR description lists the AC from TASKS.md and how each was checked
