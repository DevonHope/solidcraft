# Drawings beta: progress and handoff

This file lets any machine pick up the local **Drawings beta** work where the last one stopped. It
is the single source of truth for status. Notes in `plan/` are gitignored and stay on the machine
that wrote them; everything another machine needs is copied here.

**Every session:**
1. **Start:** read [Status right now](#1-status-right-now), then [Next steps](#6-next-steps-in-detail).
2. **While working:** keep the [checklist](#2-progress-checklist) current.
3. **Stop:** update the Status block, add an entry to the [Event log](#7-event-log), then commit
   and move the branch. See [Moving work between machines](#3-moving-work-between-machines).

Times in the log are local time on the machine that did the work, with the UTC offset shown.

---

## 1. Status right now

| | |
|---|---|
| **Last updated** | 2026-10-27 midday, on the Mac (machine B, see [Machines](#machines)) |
| **Branch** | `beta/drawings`: **b6312c4a**, the merge of the new `upstream/drawings` ebafe074, plus the commit that updates this file. |
| **Working tree** | Clean, with no merge in progress. |
| **Pushed?** | **No.** `origin/beta/drawings` is still 88f65b4d. The Mac's commits need the owner's OK before they go up (§5). |
| **Phase** | **Phase 1 of 6 (integration base), partly done.** The base is now the current `upstream/drawings`. |
| **Running** | Nothing. |
| **Next action** | The owner decides on #3891 ([§6.2](#62-decide-3891-the-owners-call)). Three gates are still owed on the merge: the engine targeted tests, `clippy -D warnings` and `cargo check -p solidcraft`. |
| **Blockers** | The owner's decision on the drawing storage model, and **machine B's build limits** (see the caps note in [§5](#5-working-rules-agreed-with-the-owner)). |

---

## 2. Progress checklist

Legend: `[x]` done · `[~]` partly done · `[ ]` not started.

### Before the beta: upstream help (done)
- [x] Researched the drawings effort upstream (RFC #860, milestone 8, the drawing PRs and their claims). Summary in [§8.4](#84-upstream-picture-as-of-2026-10-27).
- [x] Reviewed #3877, #4817, #2457 and #6788 locally, then posted four comment reviews with the owner's approval ([§8.5](#85-reviews-posted-upstream)).

### Phase 0: set up
- [x] Rust updated to 1.99.1 (the workspace needs ≥ 1.95), with the `wasm32-unknown-unknown` target.
- [x] Local `main` fast-forwarded to `upstream/main` 84a3a0c4. The fork's `origin/main` is still 56bcd872, because nothing has been pushed.
- [x] Dev build runs beside the installed 0.24.0 release, with separate data ([Appendix A](#appendix-a-launchers)).
- [x] Three test parts built and their volumes checked ([Appendix B](#appendix-b-test-parts)).
- [~] Baseline of `cargo xtask ci` on `main`:
  - **Passing:** fmt, render assets, assets, icons, licences, lists, skills, layers, ui-containers.
  - **Failing:** clippy, on Windows only. Upstream `main` already fails the same way ([§8.6](#86-known-problems-and-fixes)).
  - **Not run:** wasm, the full test suite and doc tests. They were stopped because the laptop was overloaded.
  - **Skipped:** the web offline test, because Chrome isn't installed.

### Phase 1: integration base (in progress)
**Done when:** the branch builds, the app starts, and the existing tests pass. No new behaviour yet.
- [x] Created `beta/drawings` from `upstream/main` 23734142.
- [x] Merged `upstream/drawings` (f18616e1) as **c4434126**. All 8 conflicting files are resolved ([§8.3](#83-how-the-drawings-merge-was-resolved-c4434126)).
- [x] Merged #7316 (55389724, which contains #4817) as **eaac419a**.
- [x] Targeted tests pass:
  - `solidcraft-doc`: 86;
  - `solidcraft-io`: 67;
  - `solidcraft-engine`, filtered to bom and metadata: 36.
- [x] fmt, lists and skills pass.
- [x] **Merged the new `upstream/drawings` (ebafe074)** on machine B, 2026-10-27. This replaced
  "merge #5867 for #3877": upstream merged **#4817, #3877 and #2457** into the drawings branch
  overnight, so taking the branch brings all three ([§6.1](#61-merge-the-new-upstreamdrawings-done)).
  24 conflict regions in 7 files, all from this branch holding #4817's older head.
- [x] `cargo build -p solidcraft` on macOS: clean, 3 m 55 s. fmt, lists and skills pass.
- [ ] **Decide the drawing storage model** before #3891 ([§6.2](#62-decide-3891-the-owners-call)). Upstream
  dropped the separate drawing file; the beta's D1 and item 6 assume `.scdwg`.
- [ ] Merge #7316's new head (1403e678): the rebased ArcMap commit, independent of the decision.
- [ ] Merge #3891 (36cf48a3) and #5867 (cce39b40), **or** hold to the plan and write Phase 2 by hand.
- [ ] Merge #7430 (1aaff713, dashes and PNG). Independent: stacked on `drawings`, not on #3891.
- [ ] Cherry-pick or merge #6482 (2c084c49, `mesh_lines`). Independent: its base is `main`.
- [ ] Merge #6788 (c70ec84e, tolerance text). Optional.
- [ ] Fix the `checked!` macro mismatch when #5867 lands: `main`'s macro at
  `crates/engine/src/schema/mod.rs:69` takes **2** arguments (`$id:literal, $handler:path`);
  #5867's call sites pass 3. Adapt the call sites, not the macro.
- [ ] Launch-check the dev app, take a screenshot over the control channel, then close the app.
- [ ] Record the base in `BETA-BASE.md`: every merged SHA, in order.

### Phases 2–6 (not started)
- [ ] **Phase 2:** headless drawing core: session, records, views, layout, paper scene, commands, Update, SVG.
- [ ] **Phase 3:** text, notes and dimensions.
- [ ] **Phase 4:** DXF, PDF and PNG export.
- [ ] **Phase 5:** Drawing workspace UI.
- [ ] **Phase 6:** hardening: tests, hostile parameters, a scenario, screenshots, performance, docs.

Full task lists are in [§8.2](#82-phases-26-full-task-lists).

---

## 3. Moving work between machines

The commits on `beta/drawings` and this file both have to reach the other machine. `plan/` is
gitignored, so it never travels.

**Option A (recommended): a branch on your fork.**
- `origin` is the fork, DevonHope/solidcraft. Decision D5 allows the beta to live there.
- The branch is public on the fork. It was first pushed on 2026-10-27 with the owner's OK. **Agents: confirm with the owner before each later push.**

```bash
# leaving a machine: commit this file with the work, then push
git add PROGRESS.md
git commit -m "Drawings beta: progress notes"
git push -u origin beta/drawings

# arriving on a machine
git fetch origin
git switch beta/drawings          # first time: git switch -c beta/drawings origin/beta/drawings
git pull --ff-only
```

**Option B: a bundle file, with no push.** Use it if the beta must stay off GitHub. Copy the file
by USB or a cloud drive.

```bash
# leaving
git bundle create beta-drawings.bundle upstream/main..beta/drawings
# arriving, in a clone that already has upstream/main
git fetch /path/to/beta-drawings.bundle beta/drawings:beta/drawings
git switch beta/drawings
```

**Rules for both options:**
- Never work on both machines at once. Hand off by committing, then moving the branch.
- Never force-push. If the branches diverged, merge them and note it in the log.
- **Rebuilding instead of moving:** the base can also be rebuilt from the SHAs in [§8.7](#87-shas). Redoing the resolutions in §8.3 takes about an hour, so prefer moving the commits.

### Machines

| Name | What it is | Notes |
|---|---|---|
| **A: laptop** | Windows 11 Home, 16 threads; user folder `C:\Users\D`; repo at `C:\Users\D\Documents\PROJECTS\3D\Software\solidcraft` | Everyday machine, so **keep builds light** (§5). The installed release 0.24.0 is in `C:\Program Files\SolidCraft`: never touch it. Git 2.47, Rust 1.99.1, `gh` signed in as DevonHope. |
| **B: Mac** | macOS 27.0.1 (Darwin 27.0.0), Apple silicon, 14 cores, 48 GB; repo at `~/dev/solidcraft`, trial worktree at `~/dev/sc-trial` | Added 2026-10-27. Rust 1.99.1 + wasm32, git 2.55, `gh` signed in as DevonHope. **The NAS copy at `~/Library/CloudStorage/SynologyDrive-Software/solidcraft` is not a git repo** and must not be built in: Synology filters dot-folders, so `.git` never arrives. Builds use `CARGO_BUILD_JOBS=8`. A clean `cargo build -p solidcraft` takes **3 m 55 s**. |

---

## 4. Setting up a new machine

1. **Clone with both remotes.**
   ```bash
   git clone https://github.com/DevonHope/solidcraft.git
   cd solidcraft
   git remote add upstream https://github.com/tangency-labs/solidcraft.git
   ```
2. **Windows only: switch to LF before checking anything out.** Git for Windows defaults to CRLF, and the skills gate then fails on every SKILL.md. Three files are CRLF in the repo itself, and that's correct.
   ```bash
   git config core.autocrlf input
   # only if files were already checked out as CRLF, and only on a clean tree (it discards local edits)
   git rm --cached -r -q . && git reset -q --hard HEAD
   ```
3. **Rust ≥ 1.95** (1.99.1 on the laptop) with the wasm target. Rust 1.89 fails: the `emath` crate needs `vec_deque_pop_if`.
   ```bash
   rustup update stable
   rustup target add wasm32-unknown-unknown
   ```
4. **Fetch the upstream branches and the drawing PR heads** into `refs/remotes/prs/pr-N`:
   ```bash
   git fetch upstream main drawings
   for n in 2457 3013 3877 3891 4817 5349 5867 6482 6788 7255 7316 7430; do
     git fetch upstream "pull/$n/head:refs/remotes/prs/pr-$n"
   done
   ```
   PR heads move. Compare them with [§8.7](#87-shas) and log any change before merging a newer head.
5. **Get the branch** using §3.
6. **Install `gh`** and sign in yourself with `gh auth login --web`. It's needed only for reading or posting on GitHub.
7. **Windows only: a python3 shim** for `cargo xtask ci`, whose icon checks call `python3`, which Windows maps to the Store stub. `plan\ci.cmd` creates a venv with a `python3.exe` copy and puts it first on PATH ([Appendix A](#appendix-a-launchers)). On Linux or macOS, nothing is needed.
8. **Create the dev data folder and the launchers** from [Appendix A](#appendix-a-launchers), so the dev build never shares settings, autosaves or control tokens with an installed release.
9. **Rebuild the test parts** from [Appendix B](#appendix-b-test-parts) if you need them. Recreate `plan/` notes if you want them; this file is enough.

---

## 5. Working rules agreed with the owner

- **Before posting to GitHub, show the findings first.** Post one item at a time, showing each before it goes up.
- **No push, PR, comment or claim without the owner's explicit OK.** Nothing has been pushed yet.
- **Keep the laptop usable:**
  - `CARGO_BUILD_JOBS=4` and `RUST_TEST_THREADS=2`;
  - targeted tests only (`cargo test -p <crate> <filter>`), never `cargo xtask test` or the full `cargo xtask ci` unless the owner asks;
  - close the dev app as soon as a check is done.
  - A stronger machine B can relax this if the owner agrees. Note it in the log.
- **Machine B (the Mac) crashed once at 8 jobs. Use 2.** On 2026-10-27, `CARGO_BUILD_JOBS=8` with
  `RUST_TEST_THREADS=4` **rebooted the Mac** during the engine test build and clippy.
  **The cause is not confirmed.** The same 8 jobs had already built `-p solidcraft` cleanly in
  3 m 55 s and run the geom, doc and io tests, so raw RAM is not the obvious answer. What was also
  true at the time: the shared `CARGO_TARGET_DIR` had grown to **31 GB** (free disk 106 GB → 76 GB),
  Spotlight was indexing all of it, and the `dev` profile is `optimized + debuginfo`, so the engine
  link is heavy. A busy-wait shell loop was spinning a core as well.
  Precautions on machine B until the cause is known: **`CARGO_BUILD_JOBS=2`**,
  **`RUST_TEST_THREADS=1`**, `nice -n 19`, `cargo check` in preference to `cargo build`, one crate
  at a time, and never a `while pgrep` wait loop. `target/.metadata_never_index` now keeps Spotlight
  out of the build directory. The owner's words: "be conscience of this machines limits and work
  within them".
- **Never touch the installed release.** Never change system or security settings: PowerShell's execution policy stays Restricted, so use `.cmd` launchers.
- **Project rules (AGENTS.md):**
  - Clean room: no GPL, LGPL or AGPL code, not even reading it (OpenCASCADE, FreeCAD, SolveSpace, CadQuery, LibreCAD, QCAD), and nothing copied from other CAD products.
  - No unwrap, expect or panic in non-test code.
  - Everything is a command, with input and output schemas.
  - New format changes, strings and attribution go in pending files.
  - Sorted lists stay sorted.
- **Merge strategy:** take `main`'s newer behaviour, then layer the drawings generalisations on top. Check that an auto-merge didn't silently drop `main` code: it happened once (§8.3).

---

## 6. Next steps in detail

Export the caps first in every shell:
- Bash: `export CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=2`
- cmd: `set CARGO_BUILD_JOBS=4` and `set RUST_TEST_THREADS=2`

### 6.1 Merge the new `upstream/drawings` (done)

Done on machine B, 2026-10-27. Kept here because it explains the branch's shape.

Upstream merged **#4817** (de7bf4fc), **#3877** (3575c997) and **#2457** (ebafe074) into the
`drawings` branch at 04:18-04:20 UTC on 2026-10-27, so `upstream/drawings` moved from f18616e1 to
**ebafe074**. Merging the branch brought all three in at once; the plan's "merge #5867 to get
#3877" no longer applies.

Every conflict had one cause: this branch carries **#4817's old head** (77e58dfa, pulled in with
#7316's old head 55389724), and the branch carries #4817 as merged. 24 regions in 7 files:

| File | Regions | Resolution |
|---|---|---|
| `crates/geom/src/profile.rs` | 3 | **Ours.** #7316's `ArcMap`, `Seg2::mapped`, `similar()` through it. |
| `crates/geom/src/tests.rs` | 2 | **Ours.** The `every_kind_of_segment` helper and its two ArcMap tests. |
| `crates/geom/src/lib.rs` | 1 | **Ours.** Keeps `ArcMap` in the `profile` re-export. |
| `crates/doc/src/eval_more.rs` | 1 | **Ours.** `mapped(f, ArcMap::turn(-0.0, true))` over the inline match. |
| `crates/doc/src/sheet.rs` | 3 | **Ours.** Same pattern in all three. |
| `crates/geom/src/drawing_view.rs` | 8 | **Theirs**, the merged #4817: 291 lines against our 269. |
| `crates/geom/src/drawing_view_tests.rs` | 6 | **Theirs**: 273 lines against our 203. |

`map_controls` stays private: everything here goes through `mapped`, and the only other hit is an
unrelated `ui-egui` function of the same name. Taking theirs for the two `drawing_view` files loses
nothing, because only 77e58dfa ever touched them on this side.

Auto-merged hunks were read for silent drops, the §8.3 lesson: `check_design_numbers` is still in
`doc/format.rs`, `geom/aabb2.rs` gains `intersection()`, `geom/paper.rs` exports
`MAX_COORDINATE_MM`, `geom/drawing_sheet.rs` only changes a doc comment.

### 6.1a Merge cost from the new base, measured 2026-10-27

From **b6312c4a** with `git merge-tree` (no build, no working tree). The earlier figures in the log
were taken from 88f65b4d and included the 24 regions the drawings merge has now absorbed.

| To merge | Conflicting files | Notes |
|---|---|---|
| **#7316** 1403e678 | **none** | The rebased ArcMap commit, the same work kept as "ours" in §6.1. Independent of the #3891 decision. |
| **#6482** 2c084c49 | **none** | `mesh_lines`, base `main`. Independent. |
| **#7430** 1aaff713 | 2: `Cargo.lock`, `crates/io/Cargo.toml` | A dependency addition. Stacked on `drawings`, not on #3891, so independent. |
| **#3891** 36cf48a3 | 9: doc/format.rs, engine cmd/{edit,file,inspect,metadata,mod}.rs, engine/lib.rs, io/design.rs, ui-egui/agent_chat/tests.rs | Needs the storage decision first. |
| **#5867** cce39b40 | 12 | #3891 plus 3. Needs the same decision. |

So three of the five can go in whatever the owner decides about #3891.

### 6.2 Decide #3891: the owner's call

**Upstream changed the storage model.** 472a3306 says a drawing now lives in the file of the design
it draws ("bherbruck's proposal, Cameron's choice, so the separate drawing file goes"), and it
deletes `DRAWING_EXT`, `read_drawing`, `read_drawing_file`, `write_drawing_container`, `file_kind`,
`FileKind` and `is_drawing_path`. The beta's **D1** and beta item 6 assume a separate `.scdwg`
linked to the design, and Phase 2's first task is to port #3891 *with* `.scdwg`. That plan is now
against an upstream decision by the maintainers.

**What taking #3891 (36cf48a3) would give**, as three commits on the new `drawings`:
- 472a3306: drawings saved inside the design, both container and plain JSON, with the refusal rules,
  the round-trip tests and the two pending format files;
- e0cb46c1: `crates/engine/src/cmd/drawing.rs`, ten commands (`drawing.create`, `.list`, `.open`,
  `.close`, `.delete`, `.inspect`, `.set`, `.properties`, `.properties.set`, `.properties.remove`),
  each drawing its own undo, plus 454 lines of tests and the en/ja/zh strings;
- 36cf48a3: unknown-command tests.

That is most of Phase 2's "drawing session" and "records" tasks, as merges rather than hand-written
code. The commands use `.params(…)`; check whether they carry **output** schemas before relying on
them, because AGENTS.md requires both and `schema/unconverted.txt` may only shrink.

**If the owner holds to `.scdwg`:** skip #3891 and #5867, write Phase 2 by hand as planned, and
accept that the beta diverges from upstream's file format, which makes it hard to give back (D5).

### 6.3 Merge #7430 (dashes and PNG, which includes #7255)
`git merge --no-ff prs/pr-7430`. It has not been tried yet. Then run the targeted geom and io tests (`cargo test -p solidcraft-io paper`).

### 6.4 Bring in #6482 (`mesh_lines`, the shared hidden-line module, into `main`)
- Cherry-pick or merge `prs/pr-6482`.
- The earlier research found it conflicts only in `crates/engine/src/cmd/view_dxf.rs`.
- Test with `cargo test -p solidcraft-engine view_dxf`.

### 6.5 Optionally merge #6788 (tolerance text, `format_dimension`, into `main`)
- Merge `prs/pr-6788`.
- Our review found two problems ([§8.5](#85-reviews-posted-upstream)). Check whether the author has fixed them before relying on it.

### 6.6 Close out Phase 1
1. `cargo build -p solidcraft`. A debug build takes about 15 minutes from clean on the laptop.
2. Targeted tests on geom, doc and io, plus the engine filters you touched.
3. Launch with `plan\run-dev.cmd`, or the Bash lines in Appendix A.
4. Take a screenshot with `control.py` (Appendix C) by calling `ui.screenshot` with `{"path": "<file>.png"}`. Then close the app.
5. Write `BETA-BASE.md`: base `upstream/main`, then each merged PR with its SHA, in order.
6. Tick Phase 1 here, log it, and hand off.

**If a test link fails with `LNK1201`** (Windows, writing the program database): delete that test binary's stale `.pdb` in `target/debug/deps` and rerun. It's not a full disk.

Then start **Phase 2** ([§8.2](#82-phases-26-full-task-lists)).

---

## 7. Event log

Add new entries at the end. Say what was done, what was decided and by whom, what broke and how it
was fixed, and the exact state you left.

### Session 1: laptop (machine A), 2026-10-26 evening to 2026-10-27 00:10 (UTC−4)

**About 20:50 to 21:00: research.**
- Read AGENTS.md, the skills, ROADMAP, CONTRIBUTING, RFC #860, milestone 8, and every open or closed drawing PR and issue.
- Wrote the upstream to-do list (`plan/drawings-todo.md`, summarised in §8.4). Checked against `upstream/main` 924724dc and `upstream/drawings` f18616e1.

**Side question: the Fusion 360 appearance library.**
- There's no direct route, and the clean-room rules forbid importing another product's material library.
- Instead, recreate the looks and share them with Appearance → Local library → Export design library (`library.export` / `library.apply`).
- STEP import brings per-model colours, but those don't go into the library.

**About 21:00 to 21:30: local reviews of #3877, #4817, #2457 and #6788.**
- Each was built and its tests run in a separate target folder.
- Findings were written to `plan/drawings-pr-reviews.md` (§8.5).

**About 21:30: GitHub CLI.**
- `gh` wasn't installed. Installed it with `winget`; the owner approved the UAC prompt.
- The owner signed in with `gh auth login --web`, as DevonHope.

**About 21:40 to 21:50: reviews posted.**
- Posted the four reviews as **COMMENT** reviews, one at a time; the owner approved each before it went up.
- Review ids: #2457 5661292983, #3877 5661295428, #4817 5661297624, #6788 5661302434.

**About 22:00: beta plan.**
- The owner asked for a local beta plan. Wrote `plan/drawings-beta-todo.md`, which holds decisions D1–D5 and phases 0–6. Its contents are in §8.1 and §8.2.

**About 22:05: running the dev build beside the installed release.**
- The installed 0.24.0 release is in `C:\Program Files\SolidCraft` and on PATH.
- Created `plan/run-dev.cmd`. It gives the dev build its own data folder (`%USERPROFILE%\SolidCraft-dev`), control port 27900 and no update check.
- Designs saved by the dev build use a newer format number that 0.24.0 can't open, so work on copies.

**About 22:05 to 22:25: Phase 0.**
- **Rust:** the dev build failed on Rust 1.89 (`emath` needs `vec_deque_pop_if`). `rustup update stable` gave 1.99.1; added the wasm32 target.
- **`main`:** fast-forwarded to `upstream/main` 84a3a0c4.
- **PowerShell:** the Restricted execution policy blocked a `.ps1` launcher. We used `.cmd` and left the policy alone.
- **Dev app:** built (about 15 minutes, debug) and launched. Took a screenshot over the control channel to confirm it works.
- **Test parts:** built with the CLI, then measured by appending `inspect.center_of_mass` to each script:
  - plate-two-bores: 156,881 mm³, against 156,858 by hand;
  - boss-plate: 61,747 mm³, against 61,801;
  - bracket: 22,388 mm³, against 22,385.
  - All are within 0.1%, the mesh volume's error.

**About 22:25 to 22:40: `cargo xtask ci` baseline.**
- **python3:** it resolved to the Microsoft Store stub. Added a venv shim through `plan/ci.cmd`.
- **Line endings:** the skills gate failed with "no front matter" because of the CRLF checkout. The owner approved switching to LF:
  - `git config core.autocrlf input` (repo-local);
  - `git rm --cached -r -q .`;
  - `git reset -q --hard HEAD`.
- **Results:** as in the Phase 0 checklist. Clippy fails on Windows-only dead code, which upstream `main` already has.

**About 23:00: the laptop was overloaded.**
- The owner said "solidcraft is eating up resources on this machine, limit your testing".
- Stopped the full `cargo xtask test` run and closed the dev app.
- Adopted the caps in §5.

**About 23:05: Phase 1 started.**
- The owner asked for the integration branch with the caps and targeted tests only.
- Fetched the PR heads. Created `beta/drawings` from `upstream/main` 23734142.
- Merged `upstream/drawings` f18616e1: **8 files conflicted** (doc/format.rs, doc/metadata.rs, doc/lib.rs, engine/cmd/bom.rs, engine/cmd/metadata.rs, io/design.rs, AGENTS.md, glossary).
- Resolved them as described in §8.3, taking `main`'s newer semantics and putting the drawings generalisations on top.
- **Caught a silent auto-merge regression:** design files had lost `main`'s engineering-card number bounds. Restored `check_design_numbers`.
- **Fixed during testing:**
  - an unclosed brace where the format tests from both sides were joined;
  - doc tests still calling the old 2-argument `metadata.set` and `WELL_KNOWN`;
  - an unused `PackageReader::part` (removed);
  - the bom test's expected refusal message (updated for `bom_type`);
  - `LNK1201` from a stale `.pdb` (deleted the file and reran);
  - a rustfmt diff in `bom.rs`.
- Results: doc 86, io 67 and engine bom/metadata 36 tests pass; fmt, lists and skills pass.

**2026-10-27 00:05: commits.**
- **c4434126:** the drawings merge.
- **eaac419a:** merge of #7316 (55389724, which includes #4817).

**About 00:06: #5867 tried, then paused.**
- Tried merging #5867 (5bb34f49). It conflicted in 15 regions across 10 files (§6.1).
- The owner said "lets pause here". Ran `git merge --abort`.
- **State left:** branch clean at eaac419a, no processes running, nothing pushed.

**Later on 2026-10-27: this file.**
- The owner asked for this progress file so another machine can continue.
- Wrote `PROGRESS.md` at the repo root. `cargo xtask lists` and `cargo xtask skills` pass.
- With the owner's OK, committed it on `beta/drawings` and pushed the branch to `origin` (DevonHope/solidcraft) as `beta/drawings`. This is the beta's first push; nothing went to upstream.
- **State left:** the branch is clean and matches `origin/beta/drawings`. Nothing is running.

### Session 2: Mac (machine B), 2026-10-27 09:59 to midday (UTC-4)

**The owner's ask:** read the synced folder, build last night's version, and carry on with the
to-do list.

**About 10:00: what machine B actually had.**
- The folder the owner pointed at, `~/Library/CloudStorage/SynologyDrive-Software/solidcraft`, is
  the laptop's working tree synced by Synology Drive. **It has no `.git`**: Synology filters
  dot-folders, so `.gitignore` and `.github` are missing too. Its `target/` holds the laptop's
  **Windows** binaries (`solidcraft.exe`, a 1.2 GB `.pdb`).
- No Rust at all: no rustup, no cargo, nothing from Homebrew.
- The owner said the work "is not published on the git here". `git ls-remote` showed the fork does
  have `beta/drawings`, at **88f65b4d**, so the branch pushed in session 1 was reachable.
- **Decision (owner, by prompt):** install rustup 1.99.1 with the wasm32 target, and clone the fork
  to `~/dev/solidcraft`, outside the synced folder, so that Mac build artifacts never reach the NAS
  and the laptop. The synced folder has been left untouched.

**About 10:30: the clone is last night's version.**
- History holds c4434126, eaac419a, 23734142, f18616e1 and 55389724, as §8.7 says.
- `diff -rq` of the clone against the synced folder, less `.git`, `target` and `plan`: **no source
  file differs.** Only 151 `.svg` files, and those only by CRLF, from the Windows checkout. Nothing
  exists only on the synced side, so the laptop had committed everything.

**About 10:45: the plan was out of date, because upstream moved overnight.**
- Re-fetching the PR heads showed nearly all of them moved. **#4817, #3877 and #2457 were merged
  into `upstream/drawings`** at 04:18-04:20 UTC; the rest were restacked on the result. **#5867 was
  force-pushed**: its old head 5bb34f49 no longer exists, so §6.1's 15-region analysis was written
  against a commit that is gone. `upstream/main` also moved 49 commits ahead. Recorded in §8.7.
- Trial merges from 88f65b4d: new `drawings` 24 regions in 7 files; #7316's new head 14 in 2;
  #7430 26 in 9; #3891 44 in 16; #5867 53 in 19. The last two include the first's 24.
- So the next step was **not** "merge #5867" but "merge the new `upstream/drawings`".

**About 10:48: the baseline build, and a mistake.**
- `cargo build -p solidcraft` at 88f65b4d: **clean in 3 m 55 s** with `CARGO_BUILD_JOBS=8`.
- **Mistake:** `git worktree add` was refused because `beta/drawings` was already checked out, the
  following `cd` failed, and a trial merge therefore ran **in the checkout that was building**. It
  was aborted at once; the tree went back to 88f65b4d with no changes, and the build finished
  clean. Later trial merges used `git worktree add --detach ~/dev/sc-trial`.

**About 11:00: the merge, and the tests that passed.**
- Merged the new `upstream/drawings` ebafe074: 24 regions in 7 files, resolved as §6.1 records.
- `solidcraft-geom` 276, `solidcraft-doc` 464 and `solidcraft-io` 179 tests pass, 0 failed.
  `cargo fmt --check`, `cargo xtask lists` and `cargo xtask skills` pass. Committed as **b6312c4a**.

**About 11:20: the caps were set too high, and the Mac crashed.**
- Machine B was run with `CARGO_BUILD_JOBS=8` and `RUST_TEST_THREADS=4`, raised from §5's 4 and 2
  on the reasoning that 14 cores and 48 GB beat the laptop. During the engine test build and clippy
  the **Mac rebooted**. The cause was not pinned down: the same caps had already built
  `-p solidcraft` in 3 m 55 s and passed the geom, doc and io tests. Afterwards the target dir was
  31 GB with Spotlight indexing it, free disk had gone 106 → 76 GB, and a busy-wait shell loop was
  spinning a core. See the machine B caps in §5.
- The owner: "whatever youre doing is crashing my mac, be conscience of this machines limits and
  work within them." All build processes were killed. §5 now carries the machine B caps:
  **2 jobs, 1 test thread**, `cargo check` over `cargo build`, one crate at a time.
- **Three gates are still owed on b6312c4a:** the engine targeted tests (drawing, bom, metadata),
  `clippy -D warnings`, and `cargo check -p solidcraft`. The commit message says so too.

**Still open at the end of the session:** the owner's decision on #3891 (§6.2) and, under §5,
whether to push. #5867 was last updated 10:42 UTC, so the author is restacking now and the heads in
§8.7 may move again.

---

---

## 8. Reference

### 8.1 What "beta" means, and the decisions

For **one part**, a user can:
1. choose **Create drawing**, which opens a drawing in its own tab, linked to the part;
2. get a standard sheet (ISO A-sizes and US sizes) with a border and an editable **title block**;
3. place **front, top, right** (first- or third-angle) and **isometric** views at a chosen or fitted scale, with visible lines and **dashed hidden lines**;
4. drag views, and add **dimensions** (horizontal, vertical, aligned, radius, diameter) and **notes**;
5. edit the part, see the drawing marked **out of date**, and press **Update**. A dimension whose edge disappeared shows as *dangling* instead of silently moving;
6. **save and reopen** it (`.scdwg`, linked to the `.solidcraft`);
7. **export** the sheet to **SVG, DXF, PDF and PNG**;
8. do all of this **from the CLI and over MCP** with the same commands.

**Not in the beta:** assemblies, parts lists, balloons, sections, detail and broken views, multiple
sheets, GD&T, and exact (B-rep) hidden lines. View lines come from the mesh, so the UI and exports
say "approximate (mesh)".

**Decisions** (change them before Phase 2 if you disagree):
- **D1. A drawing is its own document tab, linked to the part.** This is upstream's W11 model: separate linked files, with undo per file, compatible with #3891.
- **D2 (still the beta's choice; see the 2026-10-27 note below). View lines come from the existing mesh hidden-line code** (`view_dxf` / `mesh_lines` from #6482), plus a new `View::orthographic(&Plane)`. Exact hidden lines (#2457) come after the beta.
  - **2026-10-27 note:** #2457 was merged into `upstream/drawings`, so `crates/geom/src/hlr/` is now
    in this branch's base. D2 is unchanged: the beta still draws view lines from the mesh. The exact
    code is simply available early, behind the same interface, for after the beta.
- **D3. Text is drawn as glyph outlines in Liberation Sans**, which already ships under the OFL. DXF uses TEXT entities instead.
- **D4. PDF uses `pdf-writer` 0.15** (MIT OR Apache-2.0, pure Rust, builds for wasm), added to `io` only.
- **D5. The beta stays local or on a branch of the fork.** Upstream claims cover most pieces. Give work back through the owner's issue, not a competing PR (§8.4).

### 8.2 Phases 2–6: full task lists

**Phase 2: headless drawing core (about 2 weeks).**

**Done when:** a script creates a drawing from a plate, adds the standard views and exports it, and the SVG is correct: front, top, right and iso views inside a bordered sheet, with dashed hidden lines. Two runs give identical bytes.

- [ ] **Drawing session ("K4-lite").** Port the structure of #3891:
  - `Session.drawing: Option<Box<DrawingFile>>` and `Snapshot.drawing`, so undo works per file;
  - the per-command `Files` gate (Design, Drawing or Any);
  - `DrawingFile`, holding the source part as a nested session (Open, Missing or Unreadable);
  - `.scdwg` in `doc.open`, `file.save` and `file.save_as`.
  - Defer relink, locate and `source.execute`.
  - Keep #3891's ids: `drawing.create`, `drawing.inspect`, `drawing.set`, `drawing.properties.set`.
  - Add input and output schemas, which #3891 lacks. The pattern is `CommandSpec::new(...).contract(&C).output(output_schema_for::<T>)`; see `engine/src/cmd/features.rs:41`.
- [ ] **Records.**
  - `Sheet` gains `title_block`, `views`, `dimensions` and `notes`; the drawing gains `projection` (first or third).
  - The format change is **one file** in `crates/doc/src/drawing/format/pending/`.
  - Add a save → open round-trip test and a mutation fuzz case in `io/src/mutate.rs`.
- [ ] **Exact view basis.** Add `View::orthographic(&Plane)` to `mesh_lines` (right = x, up = y, no perspective), fed by #4817's `NamedView::basis()`.
  - Going through the camera tilts top and bottom views by 1e-6 rad, which leaves stray stubs on vertical edges.
  - Test a view where faces are seen edge-on.
- [ ] **View lines (#5867's planned V3).**
  - `ViewLines` is backend-neutral: visible and hidden pieces, with an accuracy of "sampled ± chord".
  - Build one mesh `Scene` per source and chord, shared by all views.
  - Chain the pieces into polylines.
  - Add `lines` and `status: approximate` to `drawing.view.snapshot`.
  - Chord = 0.01 mm on paper ÷ scale, clamped to 1e-4..1 mm. Add an option to show tangent edges.
- [ ] **Standard layout.** A pure function in geom:
  - fit the scale with `Scale::fit`;
  - place the projected views first- or third-angle, aligned on shared axes;
  - put the iso view in the free corner, clear of the title block.
- [ ] **Paper scene builder.**
  - The sheet and border come from `SheetLayout`; nothing draws them yet.
  - The title block comes from #3877's `TitleBlock::layout`, plus **one built-in block of our own design**.
  - Add `ObjectKind::View`, `Dimension` and `Note`.
  - One group per view: visible strokes 0.5 mm, hidden strokes 0.25 mm dashed (#7430).
  - Mind the 100k-record cap per scene: simplify to paper tolerance.
- [ ] **View commands:**
  - `drawing.view.base` and `drawing.view.projected` (the names #5349 reserves);
  - `drawing.view.standard`;
  - `drawing.view.set` (move, scale, hidden-line style);
  - `drawing.delete` (a view takes its dimensions with it);
  - `drawing.sheet.set` and `drawing.title_block.set`;
  - `drawing.sheet.paper` (read-only scene JSON).
  - Stay under `drawing.`: `sheet.*` belongs to sheet metal and `view.*` to the camera.
- [ ] **Freshness and Update.**
  - Cache view lines keyed by the view record, the source's document id and revision (or file fingerprint), the build and the chord.
  - A changed source marks views out of date.
  - `drawing.update` recomputes on the file worker, like `Kind::ExportDxf`. Never on the window thread; wasm runs it serially.
- [ ] **SVG export.** `drawing.export {format: "svg", path, sheet?}` through the existing paper SVG writer.

**Phase 3: text, notes and dimensions (about 1–1.5 weeks).**

**Done when:** the title block shows the name, number, scale and date, and the sheet carries a note, two linear dimensions and a diameter. All of it survives save and reopen and appears in the SVG.

- [ ] **Text in the paper scene.**
  - Add `Item::Text {text, at, height_mm, angle, anchor}`.
  - Lay it out with `sketch::TextMetrics` (`sketch/src/text.rs:187`) and turn it into outlines with `text_geometry_styled` (`:272`), using Liberation Sans.
  - A missing glyph is an error, not a box.
- [ ] **Title-block fields.** Fill them from `drawing.properties` and the source's name and part number. Render them with #3877's cell rule.
- [ ] **Notes.** `drawing.note {sheet, text, at_mm, height_mm?, angle?, anchor?}` and `drawing.note.set`.
- [ ] **Dimensions.** `drawing.dimension {view, kind: horizontal|vertical|aligned|radius|diameter, refs, text_mm?}` and `drawing.dimension.set`.
  - **References** store `{view, body, edge_name, end?, at_model}`, with names from `doc::naming::edge_names`. View lines must carry their source edge; mesh edge indexes match `Body::edges`.
  - **On Update:**
    - resolve by name, then the nearest edge (`locate_named_edge`, `doc/src/project.rs:360`);
    - check the kind (line or circle, and the radius);
    - re-project the edge.
    - If that fails, mark it *dangling*, keep the last frame, and report it from `drawing.update`.
  - **Layout:** `sketch::dims::dim_layout_reaching` (`dims.rs:277`), scaled from px to mm (1 px = 1/3 mm). Arrowheads are filled triangles.
  - **Values:** `format_length` now, then `format_dimension` with Ø and R once #6788 is in.
  - **Exact radius:** `kernel::edge_circle` (`surface_kind.rs:80`).

**Phase 4: the other exports (3–5 days).**

**Done when:** one sheet exports to SVG, DXF, PDF and PNG, and each opens at true scale in a common viewer.

- [ ] **DXF:** a new `io/src/paper/dxf.rs` over `dxf_out::write_dxf_drawing`:
  - a layer per `ObjectKind::layer()`;
  - the HIDDEN linetype;
  - lineweights (code 370);
  - TEXT with codes 72/11 and a Liberation Sans STYLE;
  - `$INSUNITS` 4.
  - It doesn't wait for upstream's DXF chain (#6184 → #5321).
- [ ] **PDF:**
  - add `pdf-writer` 0.15 to `io`, then run `cargo xtask licences --check`. It pulls in `ryu` (Apache-2.0 OR BSL-1.0, which is accepted).
  - Write `io/src/paper/pdf.rs` with outline text.
  - **Guard every number:** pdf-writer panics on NaN or a negative width.
- [ ] **PNG:** #7430's `io/paper/png.rs`, wired to `drawing.export`.
- [ ] **Writing files:** writes are atomic through the safe-write path, and the browser build uses the download service. Never overwrite an existing file without an explicit flag.

**Phase 5: Drawing workspace UI (about 2 weeks).**

**Done when:** the whole beta flow can be done by clicking, proven by a scenario test and control-channel screenshots.

- [ ] **Workspace wiring.** Mirror the Render workspace:
  - **Switcher:** `context_menu.rs:377-390` and `:1348`, `ui_kit/menu.rs:266`.
  - **Ribbon:** `toolbar.rs:1080-1106`.
  - **Tabs:** `workspace.rs` (`visible_tabs` :556, `sync_context` :575, `select_command_tab` :626, `SECTIONS` :281).
  - **Per-document state:** `documents.rs:38-106`.
  - **Workspace host in `lib.rs`:** `show_workspace` :2244, side panel :2192, timeline hiding :2132.
  - **Control channel:** `control.rs:433` `ui.workspace` accepts `drawing`; also `ui.inspect`.
  - Update the test at `context_menu.rs:2485` that asserts Drawing is disabled.
  - Borrow #5349's `workspace::enter(app, "design"|"render"|"drawing")` dispatcher and its tab guards, not its placeholder commands.
- [ ] **Sheet canvas.** Follow the flat-pattern page (`dialogs_sheet.rs:1053-1229`):
  - a top-down orthographic camera through `navigation::handle` with `NavCaps { orbit: false, pick: true }`;
  - paper mm with y up;
  - paint `paper::Prepared`: lines, arcs (`arc_pieces`), cubics, `Shape::dashed_line`, fills as convex pieces, and text as filled outlines;
  - tessellate once per scene revision;
  - Fit and Home frame the sheet.
- [ ] **Selecting and dragging.**
  - Hit-test views, dimensions and notes by extending `empty_space` (`navigation.rs:505`).
  - A drag sends `drawing.view.set` on release.
  - The selection colour comes from theme tokens.
- [ ] **Ribbon, properties and Browser.**
  - **Ribbon groups:** SHEET, VIEWS, ANNOTATE, OUTPUT.
  - **Properties panel:** on `Panel` and `Container`, like `render_workspace_inspector.rs:225`.
  - **Browser card:** `browser.rs:974`.
- [ ] **Interactive tools.**
  - Place a view by clicking.
  - Dimension with two picks, snapping to vertices and edges, or one pick on a circle for Ø or R.
  - Click to place a note.
  - Hooks: `lib.rs:1174` (`start_interactive`).
- [ ] **Out-of-date badge and Update button.**
- [ ] **Theme.**
  - Add `paper` and `paper_ink` tokens, so the sheet stays white with black ink in dark themes.
  - Reuse the default tab accent: a new tab colour touches 24 theme files.
- [ ] **Text and icons.**
  - Strings go in `assets/locales/pending/drawing-beta.json`.
  - Reuse existing icons. New icons follow `assets/icons/ui/STYLE_GUIDE.md` and need the owner's review.
  - ATTRIBUTION rows go in `attribution/pending/`.

**Phase 6: hardening (3–5 days).**

**Done when:** `cargo xtask ci` is green, wasm builds, and the docs say what the beta does and doesn't do.

- [ ] **Engine tests:**
  - every W23a plate (84–95) in 4 views, with hand-derived line counts and lengths;
  - shared-axis alignment;
  - the same bytes from two runs;
  - edit → Update → dangling dimension;
  - save, reopen, then export.
- [ ] Add hostile-parameter cases for every new command to `hostile_params_never_panic`.
- [ ] **Scenario** `scenarios/s1xx_drawing_beta.json`. It needs a `{"paper":[x,y]}` click target in `scenario.rs:11-28`.
- [ ] **Screenshots** for each checkpoint, from the control channel (port 27900).
- [ ] **Performance:** a typical part's 4 views update in well under 1 s on desktop. Check the browser build too.
- [ ] **Docs:**
  - `docs/drawing-beta.md`;
  - the new commands in `docs/control-protocol.md`;
  - glossary terms.
- [ ] Put a **Beta** label on the workspace card.

**After the beta:**
- exact hidden lines (#2457 + #3013 + the W02 kernel adapter);
- true arcs recovered from mesh chains;
- assemblies, parts lists and balloons (#7305 → #7312);
- section, detail and broken views;
- multiple sheets and templates;
- searchable PDF text;
- tolerances and GD&T.

**Main risks:**
1. **Base drift.** `drawings` is about 615 commits behind `main`. Keep the base reproducible (`BETA-BASE.md`).
2. **Undo across two files.**
3. **Dimension references.** Without W13, prefer *dangling* to wrong.
4. **Faceted silhouettes.** Label them.
5. **The 100k-record scene cap.**
6. **Beta `.scdwg` files are disposable.** The pending format may change.

### 8.3 How the drawings merge was resolved (c4434126)

The principle: keep `main`'s newer code and semantics, and layer the drawings branch's generalisations on top.

- **`crates/doc/src/format.rs`** (the large one):
  - Kept `main`'s `knows_part_name` and `canonical_applied_name` (the format-29 alias).
  - Inside the drawings branch's `Family` (DESIGN/DRAWING), the merged `check(&self, noun, version, applied)` says:
    - version 0 → "not a SolidCraft {noun} (format 0)";
    - version > current → "this {noun} was saved by a newer SolidCraft (file format {v}; this version reads formats up to {current}): update SolidCraft to open it";
    - an unknown applied change → "... (file format {v} with `{unknown}`, which this version doesn't know) ...".
  - `upgrade(&self, noun, v, version, applied)` calls `check`, then `main`'s `already_applied` filtering.
  - Kept `main`'s `check_design_numbers` / `check_typed_numbers` (engineering-card bounds), calling `check_numbers(x, "design", depth)`.
  - **The auto-merge had silently replaced** the design `read()` call with `DESIGN.check_numbers(&v, "metadata")?`, dropping the card bounds. Restored `check_design_numbers(&v)?`.
  - Kept both sides' tests, with `main`'s adapted to `formats.upgrade("design", ...)`. Added a missing closing brace in `pending_concepts_moved_to_later_formats_are_not_reapplied`.
- **`crates/doc/src/metadata.rs`:**
  - Kept `main`'s `KEYS` table (#4786: `Key{name, scope, label, check}`, `Owner::{Design, Component}`, `set(owner, key, value)`).
  - `bom_type` stays outside the table as a component-only key:
    - `is_well_known` is `key(name).is_some() || name == BOM_TYPE`;
    - `check()` first routes `BOM_TYPE` to a new `bom_type(owner, value)`, which refuses on a design and validates with `BomType::parse`.
  - Tests use `m.set(Component, BOM_TYPE, ...)`.
- **`crates/engine/src/cmd/bom.rs`:**
  - `read()` adds `bom_type` for components (id ≠ 0).
  - The parameter loop is `BOM_KEYS.iter().copied().chain([BOM_TYPE])`.
  - The refusal message lists `bom_type`. The test in `bom_tests.rs` was updated to match.
- **`crates/engine/src/cmd/metadata.rs`:** `main`'s version, `next.set(Owner::Design, key, ...)`.
- **`crates/io/src/design.rs`:**
  - Kept `main`'s `doc.render.validate()` together with the branch's `PackageWriter::new()`.
  - Removed `PackageReader::part`, now unused (`container_kind` uses `limited_part`).
- **Smaller files:**
  - `crates/doc/src/drawing_tests.rs` uses `set(Owner::Design, "author", ...)`.
  - `crates/doc/src/lib.rs` keeps both `pub mod hole_reference;` and `pub mod ids;`.
  - `docs/book/src/reference/glossary.md` keeps both sides' rows, sorted.
- **AGENTS.md:** `main`'s router version, including the stray `>>>>>>> 4a3e0772a` line that `main` also has.
- **`skills/solidcraft-file-formats/SKILL.md`:** the drawing pending-folder note moved here: "Drawings work the same way with their own numbers: one file in `crates/doc/src/drawing/format/pending/`."

### 8.4 Upstream picture as of 2026-10-27

This changes daily. Re-check claims before acting.

- **The plan:** 48 work packages (W00–W47) in milestones M0–M6. M2, "first drawing end to end", is the likely bar for merging `drawings` into `main`.
- **On `main` today:**
  - File → Export Current View as DXF: mesh hidden lines, `crates/engine/src/cmd/view_dxf.rs`;
  - a disabled "Drawing" workspace entry;
  - the section type, number formatters, DXF writer and BOM.
- **On the `drawings` branch:**
  - sheet sizes and the ISO 5457 border;
  - the paper scene and its SVG writer;
  - the `.scdwg` file and its record;
  - BOM types and the W23a fixtures.
  - No commands, no UI.
- **Claims and blockers:**
  - CamWight holds most first-ship packages (about 20 PRs; 7 claims are empty).
  - The 0xlillith team paused on 26 Oct (#6184, #7064, #6163 and #7082 are parked).
  - The kernel pause blocks W02, so exact hidden lines on real bodies wait.
  - W13/W14 (persistent references, claim #4893, no code) is the biggest gap.
- **Giving the beta back upstream:**

  | Beta piece | Upstream package | Who holds it | How to offer it |
  |---|---|---|---|
  | Drawing session | W11/W12/W15 | CamWight #3891 | Offer schemas and tests on #3890 |
  | View lines and layout | W07 | CamWight #5867 | Offer them on #5866 |
  | Paper text, title block | W10, W18 | #4891, #7208 | Comment on #4890 and #3875 |
  | Dimensions, notes | W19/W20 | #4897 | Ask on #4896 |
  | DXF and PDF | W21/W22 | #4891; PDF P1 assigned to 0xlillith, who is paused | Ask on #4890 |
  | Workspace UI | W16 | #4895, Nathan-Dane #5349 | Ask on #4894 |

- **Suggested upstream contributions:**
  - review the BOM stack #7305 → #7312;
  - a tiny PR removing the AGENTS.md conflict marker (check for an open PR first);
  - the Windows clippy fix (§8.6);
  - take over the DXF chain #6184 after about 29 Oct, commenting first;
  - ask on #4892 about a slice of W13/W14.

### 8.5 Reviews posted upstream

Posted 2026-10-27, all as COMMENT reviews, after the owner approved each one. Watch for replies.

| PR | What it is | Finding | Review link |
|---|---|---|---|
| #2457 | W06a, exact orthographic hidden lines | The plane face's bounding box isn't padded, so slab culling can drop hits on a 1-ulp mismatch. Also, `Trim::margin` is computed per hit. | [5661292983](https://github.com/tangency-labs/solidcraft/pull/2457#pullrequestreview-5661292983) |
| #3877 | W18 S2, title blocks | `inside_lines` ignores `drawing_border_line_mm`. | [5661295428](https://github.com/tangency-labs/solidcraft/pull/3877#pullrequestreview-5661295428) |
| #4817 | W17a, view definitions | The Bottom view's orientation is untested. | [5661297624](https://github.com/tangency-labs/solidcraft/pull/4817#pullrequestreview-5661297624) |
| #6788 | W09c, tolerance text | `RoundingOutsideTolerance` fires for same-sign deviations, and Limits output can't be told apart from Stacked. | [5661302434](https://github.com/tangency-labs/solidcraft/pull/6788#pullrequestreview-5661302434) |

### 8.6 Known problems and fixes

| Problem | Fix |
|---|---|
| The dev build fails on Rust 1.89 (`vec_deque_pop_if`). | `rustup update stable`. The workspace needs ≥ 1.95. |
| Skills gate: "no front matter" (Windows). | LF checkout: `git config core.autocrlf input`, then re-checkout (§4). |
| `python3` opens the Microsoft Store (Windows). | Venv shim through `plan\ci.cmd` (Appendix A). |
| `.ps1` scripts are blocked (Windows, Restricted policy). | Use `.cmd` launchers. Don't change the policy. |
| Clippy fails on Windows only, also on upstream `main`. | `Bus::Names`/`Slow` are never built at `crates/engine/src/slicers.rs:987`, and `CHOSEN_PROGRAMS` is unused at `crates/engine/src/tests/mesh_convert.rs:234`. Both are Linux-only code. A good small upstream PR. |
| `LNK1201` when linking a test (Windows). | Delete the stale `.pdb` in `target/debug/deps` and rerun. |
| An auto-merge silently drops `main` code. | Review the auto-merged hunks in each conflicted file, not only the conflict markers. |
| The web offline test is skipped. | Chrome isn't installed on the laptop. |
| The dev window title says "v0.24.0". | Expected: the version number isn't bumped on `main`. Tell the windows apart by the port (27900) and the data folder. |

### 8.7 SHAs

**The branch:**

| What | SHA |
|---|---|
| `upstream/main`, the base of `beta/drawings` | 237341423ce7d76d484a92ea1339b8a3f6ce1525 |
| `upstream/drawings`, merged | f18616e19c8a16dd1575c522474ed54de34b247d |
| **c4434126**, the drawings merge | c44341263cc87099ea1be2ae3b8b0d66fa0b2ba0 |
| **eaac419a**, the #7316 merge (the last code commit; the commit adding this file sits on top) | eaac419ac6d241cb9a854fcc1d08fbad2e43bd93 |
| Local `main`, fast-forwarded in Phase 0 | 84a3a0c4 |
| Fork `origin/main`, not updated | 56bcd872cb57aabcfdedb0767f1fbc198f923991 |

**PR heads, first fetched 2026-10-27 morning (machine A), re-fetched 2026-10-27 10:45 EDT (machine B).**
Nearly every head moved overnight: #4817, #3877 and #2457 were **merged into `upstream/drawings`**
at 04:18-04:20 UTC, and the rest were restacked on top of the result. The old #5867 head
(5bb34f49) no longer exists, so it was force-pushed.

| PR | Head on machine A | Head now | State |
|---|---|---|---|
| #2457 | 0a84ccab | 7ae1dea8 | **MERGED into `drawings`** as ebafe074 |
| #3013 | 498dc545 | c68e49c0 | open, stacked on #2457 |
| #3877 | 1cfdcc95 | 11dc5af9 | **MERGED into `drawings`** as 3575c997 |
| #3891 | 72dfdb53 | **36cf48a3** | open; now the base of #5867 |
| #4817 | 77e58dfa | 9bac2e44 | **MERGED into `drawings`** as de7bf4fc |
| #5349 | e999ee5a | e999ee5a | unchanged; UI scaffold |
| #5867 | 5bb34f49 (**gone**) | **cce39b40** | open, stacked on #3891 |
| #6482 | 2c084c49 | 2c084c49 | unchanged; base `main` |
| #6788 | c70ec84e | c70ec84e | unchanged |
| #7255 | 8f500113 | d2fcae51 | open, inside #7430 |
| #7316 | 55389724 (merged here) | 1403e678 | open; restacked on new `drawings` |
| #7430 | 4063a536 | 1aaff713 | open, restacked |

**Branches now:** `upstream/drawings` is **ebafe074** (3 commits past the f18616e1 we merged).
`upstream/main` is **53cba0cf**, 49 commits past the 23734142 this branch is based on.

**The upstream stack today:**

```
upstream/drawings ebafe074        <- #4817, #3877, #2457 all merged in
  |- #7316  1403e678   drawings + 1
  |- #3891  36cf48a3   drawings + 3  (drawing session, storage, commands)
  |    \- #5867  cce39b40   #3891 + 4  (extents, projected views, dimensions, snapshot)
  \- #7430  1aaff713   drawings + n  (dashes, PNG)
```

The laptop also has a local branch `pr-6163` (5d658c30, the parked ledger draft), kept for reading only.

---

## Appendix A: launchers

They live in `plan/` on the laptop, which is gitignored. Copy them from here on a new machine.

**`plan/run-dev.cmd`** (Windows) runs the dev build beside an installed release:

```bat
@echo off
rem Run a development build of SolidCraft beside the installed release (C:\Program Files\SolidCraft)
rem without sharing its settings, autosaves or control tokens, and with the auto-updater off.
rem
rem   plan\run-dev.cmd [design.solidcraft]              debug build (faster to compile)
rem   plan\run-dev.cmd --release [design.solidcraft]    release build (faster to run)
rem   plan\run-dev.cmd --cli run x.json --save out.solidcraft    the dev command-line tool instead
rem
rem Dev data lives in %USERPROFILE%\SolidCraft-dev. The dev window listens for agents on port 27900,
rem token in SolidCraft-dev\control\27900.token; the installed app keeps its own port (27878 up).
rem Designs saved by a dev build use a format number 0.24.0 can't open: work on copies in
rem SolidCraft-dev\designs.

setlocal
rem Keep the laptop usable while it compiles: 4 compiler jobs unless you set CARGO_BUILD_JOBS.
if not defined CARGO_BUILD_JOBS set "CARGO_BUILD_JOBS=4"
set "ROOT=%USERPROFILE%\SolidCraft-dev"
for %%d in (settings recovery control designs) do if not exist "%ROOT%\%%d" mkdir "%ROOT%\%%d"
set "SOLIDCRAFT_SETTINGS_DIR=%ROOT%\settings"
set "SOLIDCRAFT_RECOVERY_DIR=%ROOT%\recovery"
set "SOLIDCRAFT_CONTROL_DIR=%ROOT%\control"
set "SOLIDCRAFT_UPDATE_CHECK=0"
set "MANIFEST=%~dp0..\Cargo.toml"

set "BUILD="
set "CLI="
:flags
if /i "%~1"=="--release" (set "BUILD=--release" & shift & goto flags)
if /i "%~1"=="--cli" (set "CLI=1" & shift & goto flags)

set "ARGS="
:collect
if "%~1"=="" goto run
set ARGS=%ARGS% "%~1"
shift
goto collect

:run
if defined CLI goto runcli
cargo run --manifest-path "%MANIFEST%" -p solidcraft %BUILD% -- --control 27900 %ARGS%
goto :eof

:runcli
cargo run --manifest-path "%MANIFEST%" -p solidcraft-cli %BUILD% -- %ARGS%
```

**`plan/ci.cmd`** (Windows) runs xtask with the python3 shim and the caps:

```bat
@echo off
rem Run the repository's full gate (`cargo xtask ci`) on this Windows machine.
rem The icon checks call `python3`, which Windows maps to the Microsoft Store stub. This puts a
rem private venv of your Python 3.12 (SolidCraft-dev\tools\py, with a python3.exe copy) first on
rem PATH for this run only. Chrome isn't installed, so the web offline test is skipped.
rem
rem   plan\ci.cmd             the full gate (takes a long time: it runs the whole test suite)
rem   plan\ci.cmd test ...    any other xtask command with the same PATH

setlocal
rem Keep the laptop usable: 4 compiler jobs and 2 test threads unless you set them yourself.
if not defined CARGO_BUILD_JOBS set "CARGO_BUILD_JOBS=4"
if not defined RUST_TEST_THREADS set "RUST_TEST_THREADS=2"
set "PY=%USERPROFILE%\SolidCraft-dev\tools\py"
if not exist "%PY%\Scripts\python3.exe" (
  python -m venv "%PY%" || exit /b 1
  copy /y "%PY%\Scripts\python.exe" "%PY%\Scripts\python3.exe" >nul || exit /b 1
)
set "PATH=%PY%\Scripts;%PATH%"
cd /d "%~dp0.."
if "%~1"=="" (cargo xtask ci) else (cargo xtask %*)
```

**Linux or macOS** (Bash), run from the repo root:

```bash
export CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=2
export SOLIDCRAFT_SETTINGS_DIR="$HOME/SolidCraft-dev/settings" \
       SOLIDCRAFT_RECOVERY_DIR="$HOME/SolidCraft-dev/recovery" \
       SOLIDCRAFT_CONTROL_DIR="$HOME/SolidCraft-dev/control" \
       SOLIDCRAFT_UPDATE_CHECK=0
mkdir -p "$HOME"/SolidCraft-dev/{settings,recovery,control,designs/scripts}
cargo run -p solidcraft -- --control 27900                                   # the app
cargo run -p solidcraft-cli -- run x.json --save out.solidcraft             # the CLI
```

---

## Appendix B: test parts

On the laptop they're in `%USERPROFILE%\SolidCraft-dev\designs`, with their scripts in `designs\scripts`.

- **Build one:** `plan\run-dev.cmd --cli run scripts\<name>.json --save <name>.solidcraft`, or `cargo run -p solidcraft-cli -- run ...`.
- **Check its volume:** append `{"command": "inspect.center_of_mass", "params": {}}` to a copy of the script and run it.

| Part | What it is | Measured (mm³) | By hand (mm³) |
|---|---|---|---|
| plate-two-bores | The W23a #84 plate: 100×80×20 with Ø10 bores at (20,20) and (60,50) | 156,881 | 156,858 |
| boss-plate | An 80×60×10 plate with a Ø30 boss 35 mm high, an R3 fillet at its base, a 1.5 mm chamfer at its top and a Ø12 bore | 61,747 | 61,801 |
| bracket | An L-bracket, the same as `examples/bracket.json`, with an R2 fillet and two Ø8 holes | 22,388 | 22,385 |

**`plate-two-bores.json`**
```json
{"commands": [
  {"command": "sketch.create", "params": {"plane": "XY", "name": "Plate"}},
  {"command": "sketch.line", "params": {"points": [[0, 0], [100, 0], [100, 80], [0, 80]], "closed": true}},
  {"command": "sketch.finish", "params": {}},
  {"command": "solid.extrude", "params": {"distance": 20, "body_name": "Plate"}},
  {"command": "sketch.create", "params": {"plane": "XY", "name": "Bores"}},
  {"command": "sketch.circle.center", "params": {"center": [20, 20], "diameter": 10}},
  {"command": "sketch.circle.center", "params": {"center": [60, 50], "diameter": 10}},
  {"command": "sketch.finish", "params": {}},
  {"command": "solid.extrude", "params": {"distance": 20, "operation": "cut"}}
]}
```

**`boss-plate.json`**
```json
{"commands": [
  {"command": "sketch.create", "params": {"plane": "XY", "name": "Base"}},
  {"command": "sketch.line", "params": {"points": [[0, 0], [80, 0], [80, 60], [0, 60]], "closed": true}},
  {"command": "sketch.finish", "params": {}},
  {"command": "solid.extrude", "params": {"distance": 10, "body_name": "Boss plate"}},
  {"command": "sketch.create", "params": {"plane": "XY", "name": "Boss"}},
  {"command": "sketch.circle.center", "params": {"center": [40, 30], "diameter": 30}},
  {"command": "sketch.finish", "params": {}},
  {"command": "solid.extrude", "params": {"distance": 35, "operation": "join"}},
  {"command": "solid.fillet", "params": {"edges": [[55, 30, 10]], "radius": 3}},
  {"command": "solid.chamfer", "params": {"edges": [[55, 30, 35]], "distance": 1.5}},
  {"command": "sketch.create", "params": {"plane": "XY", "name": "Bore"}},
  {"command": "sketch.circle.center", "params": {"center": [40, 30], "diameter": 12}},
  {"command": "sketch.finish", "params": {}},
  {"command": "solid.extrude", "params": {"distance": 35, "operation": "cut"}}
]}
```

**`bracket.json`**
```json
{"commands": [
  {"command": "parameters.change", "params": {"name": "width", "expression": "60 mm"}},
  {"command": "parameters.change", "params": {"name": "thickness", "expression": "6 mm"}},
  {"command": "sketch.create", "params": {"plane": "XZ", "name": "Profile"}},
  {"command": "sketch.line", "params": {"points": [[0, 0], [40, 0], [40, 6], [6, 6], [6, 30], [0, 30]], "closed": true, "ids": ["a", "b", "c", "d", "e", "f"]}},
  {"command": "sketch.finish", "params": {}},
  {"command": "solid.extrude", "params": {"distance": "width", "direction": "negative", "body_name": "Bracket"}},
  {"command": "solid.fillet", "params": {"edges": [[40, -30, -6]], "radius": 2}},
  {"command": "sketch.create", "params": {"plane": "XY", "name": "Holes"}},
  {"command": "sketch.circle.center", "params": {"center": [25, -15], "diameter": 8}},
  {"command": "sketch.circle.center", "params": {"center": [25, -45], "diameter": 8}},
  {"command": "sketch.finish", "params": {}},
  {"command": "solid.extrude", "params": {"distance": "thickness", "direction": "negative", "operation": "cut"}}
]}
```

---

## Appendix C: control-channel helper

`control.py` is a small client for the dev app's control channel: JSON lines on `127.0.0.1:PORT`,
with the token sent on the first request. On the laptop it was a scratch file, so keep a copy
wherever is convenient. It isn't part of the app.

```
python control.py 27900 %USERPROFILE%\SolidCraft-dev\control\27900.token ui.screenshot "{\"path\": \"C:/Users/D/SolidCraft-dev/shot.png\"}"
```

```python
"""Tiny client for SolidCraft's control channel: send JSON-line requests, print replies.

usage: python control.py PORT TOKEN_FILE METHOD [PARAMS_JSON] [METHOD PARAMS_JSON ...]
"""
import json
import socket
import sys


def main() -> None:
    port, token_file = int(sys.argv[1]), sys.argv[2]
    calls = sys.argv[3:]
    token = open(token_file, encoding="utf-8").read().strip()
    sock = socket.create_connection(("127.0.0.1", port), timeout=120)
    reader = sock.makefile("r", encoding="utf-8")
    i = 0
    rid = 1
    while i < len(calls):
        method = calls[i]
        params = json.loads(calls[i + 1]) if i + 1 < len(calls) and calls[i + 1].startswith("{") else {}
        i += 2 if i + 1 < len(calls) and calls[i + 1].startswith("{") else 1
        req = {"id": rid, "method": method, "params": params}
        if rid == 1:
            req["token"] = token
        sock.sendall((json.dumps(req) + "\n").encode())
        reply = reader.readline()
        print(reply[:1500].rstrip())
        rid += 1
    sock.close()


if __name__ == "__main__":
    main()
```
