# Alpha Keys go-live checklist

Repo copy. The video, x-post.txt and form-answers.md stay in Hackathon Hunter's submission folder, not in git.

Checked Nov 13, 2026 against main 8a04448. Deadline: Nov 14 23:59 UTC (Nov 15 04:59 YEKT). Submit sooner if you can: Near Hunt lists the project only after the form is sent, so earlier means more upvote time.

## Already done

- [x] Hosted demo is live on 8a04448: https://alpha-keys.vercel.app (Vercel deploy dpl_93odb1r7yeEVKeJrE5YSNNwiMJqA, READY). /, /t/alpha-1, /verify, /feed/alpha-1 all return 200.
- [x] Dashes: no em or en dashes left anywhere in the repo (README, PLAN.md, docs/TASKS.md, docs/video-outline.md fixed in 46f1e83).
- [x] README describes the live OutLayer callback (tx `ECyHsbAmWfuo19Hp8GFdUn5C2wNpxri97ABrHJzihapa`) and keeps the fixture row for the offline Quickstart.
- [x] Full-history secrets scan clean: gitleaks (8 commits, no leaks), trufflehog (0 findings), leak_check ("no address-shaped hex"), HL address appears 0 times in history, no .env ever committed (only .env.example, all secret fields empty).
- [x] LICENSE (MIT) present.
- [x] Tests on the box: Rust pnl-core, feed-core, outlayer-job, demo-seed (15 pass), web (9 pass), gateway (4 pass), copier pytest (10 pass) + ruff clean, web production build OK.
- [x] Demo video v2 is in Drive, but its last slide still says live attestation is coming. Record a new cut before posting. See `docs/video-outline.md`.
- [x] X post text: x-post.txt. Form answers: form-answers.md (idea refreshed for the TEE worker).

## Your steps, in order

0. [x] TEE is live. Callback tx `ECyHsbAmWfuo19Hp8GFdUn5C2wNpxri97ABrHJzihapa`, worker hash `cb9c3130c6858cf4b460950281cff3468eabc0d79f1652fb05061c7fe41978dd`. Docs updated. Repo is not public yet.

1. [ ] Optional but recommended: run `make check` on your machine once. The contract crate (alpha-keys, near-workspaces sandbox) cannot build on the box (no make/pkg-config, apt mirror down).
2. [ ] Fix the GitHub About text (it still says "TEE-attested"):
       gh repo edit ilerik/alpha-keys --description "Viewing-key NFTs for a hidden trader. PnL is computed in an OutLayer TEE (Intel TDX) and attested on NEAR; the account stays sealed."
3. [ ] Make the repo public: GitHub, Settings, Danger Zone, Change visibility, Public. Then check `gh repo view ilerik/alpha-keys --json visibility` says PUBLIC.
4. [ ] Post on X from your account: paste x-post.txt as is and attach alpha-keys-demo-v2.mp4. It must contain "built at NEAR HACKATHON", @NEARProtocol and #NEARHackathon. Copy the post URL (https://x.com/<handle>/status/<id>).
5. [ ] Put the post URL in the README: replace "Demo video: _TBD (X post)._" with "Demo video: <post URL>". Commit and push to main yourself.
6. [ ] Send me your mainnet NEAR address (lowercase, not a testnet account) for nearAddress, and confirm the contact email and consent.
7. [ ] Submit once at https://nearhackathon.com/#apply with the fields in form-answers.md: requestId 8ddf2554-5e38-4b61-95cb-c3d97d35c720, projectName Alpha Keys, leadName Ilya Eriklintsev, email, size 1, tracks nfts, privacy, defi, nearAddress, projectUrl https://github.com/ilerik/alpha-keys, demoUrl = the X post, idea, consent. Never fill `website`. If anything needs fixing later, use https://nearhackathon.com/request-changes, never a second form.
8. [ ] Save the receipt: screenshot the confirmation (or the 201 body) and put it in docs/submission-receipt.md.

## Live TEE

The callback is on main. Tx `ECyHsbAmWfuo19Hp8GFdUn5C2wNpxri97ABrHJzihapa`. See TEE-RUNBOOK.md for how it was produced.
