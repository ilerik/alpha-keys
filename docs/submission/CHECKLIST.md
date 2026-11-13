# Alpha Keys go-live checklist

Repo copy. The video, x-post.txt and form-answers.md stay in Hackathon Hunter's submission folder, not in git.

Checked Nov 13, 2026 against main 8a04448. Deadline: Nov 14 23:59 UTC (Nov 15 04:59 YEKT). Submit sooner if you can: Near Hunt lists the project only after the form is sent, so earlier means more upvote time.

## Already done

- [x] Hosted demo is live on 8a04448: https://alpha-keys.vercel.app (Vercel deploy dpl_93odb1r7yeEVKeJrE5YSNNwiMJqA, READY). /, /t/alpha-1, /verify, /feed/alpha-1 all return 200.
- [x] Dashes: no em or en dashes left anywhere in the repo (README, PLAN.md, docs/TASKS.md, docs/video-outline.md fixed in 46f1e83).
- [x] README line 3 now says the TEE is the design and the visible row is relayed until a matching OutLayer callback is stored. Matches the chain.
- [x] Full-history secrets scan clean: gitleaks (8 commits, no leaks), trufflehog (0 findings), leak_check ("no address-shaped hex"), HL address appears 0 times in history, no .env ever committed (only .env.example, all secret fields empty).
- [x] LICENSE (MIT) present.
- [x] Tests on the box: Rust pnl-core, feed-core, outlayer-job, demo-seed (15 pass), web (9 pass), gateway (4 pass), copier pytest (10 pass) + ruff clean, web production build OK.
- [x] Demo video v2: alpha-keys-demo-v2.mp4 (71 s, 1080p, 12.8 MB), in Drive. Its last-slide caveat is still accurate.
- [x] X post text: x-post.txt. Form answers: form-answers.md (idea refreshed for the TEE worker).

## Your steps, in order

0. [ ] NEW: make the TEE live first. Follow TEE-RUNBOOK.md (branch tee-live, 4 testnet transactions you sign), then apply POST-ATTESTATION-TEXT.md. Then continue below.

1. [ ] Optional but recommended: run `make check` on your machine once. The contract crate (alpha-keys, near-workspaces sandbox) cannot build on the box (no make/pkg-config, apt mirror down).
2. [ ] Fix the GitHub About text (it still says "TEE-attested"):
       gh repo edit ilerik/alpha-keys --description "Viewing-key NFTs for a hidden trader. A TEE is the design for attested PnL. This demo's record is relayed from a fixture, not a live enclave."
3. [ ] Make the repo public: GitHub, Settings, Danger Zone, Change visibility, Public. Then check `gh repo view ilerik/alpha-keys --json visibility` says PUBLIC.
4. [ ] Post on X from your account: paste x-post.txt as is and attach alpha-keys-demo-v2.mp4. It must contain "built at NEAR HACKATHON", @NEARProtocol and #NEARHackathon. Copy the post URL (https://x.com/<handle>/status/<id>).
5. [ ] Put the post URL in the README: replace "Demo video: _TBD (X post)._" with "Demo video: <post URL>". Commit and push to main yourself.
6. [ ] Send me your mainnet NEAR address (lowercase, not a testnet account) for nearAddress, and confirm the contact email and consent.
7. [ ] Submit once at https://nearhackathon.com/#apply with the fields in form-answers.md: requestId 8ddf2554-5e38-4b61-95cb-c3d97d35c720, projectName Alpha Keys, leadName Ilya Eriklintsev, email, size 1, tracks nfts, privacy, defi, nearAddress, projectUrl https://github.com/ilerik/alpha-keys, demoUrl = the X post, idea, consent. Never fill `website`. If anything needs fixing later, use https://nearhackathon.com/request-changes, never a second form.
8. [ ] Save the receipt: screenshot the confirmation (or the 201 body) and put it in docs/submission-receipt.md.

## Live TEE

Ilya chose real attestation before going public (Nov 13). Fixes are on branch tee-live (9775046). See TEE-RUNBOOK.md.
