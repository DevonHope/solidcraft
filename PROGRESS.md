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
| **Last updated** | 2026-10-28, on the Mac (machine B, see [Machines](#machines)) |
| **Branch** | `beta/drawings` at **62eab387**, two commits on top of 3bf352b7. Every merged SHA is in [BETA-BASE.md](BETA-BASE.md). |
| **Working tree** | Clean, with no merge in progress. |
| **Pushed?** | **Yes**, through **62eab387**, to the fork as `origin/beta/drawings` on 2026-10-28 with the owner's OK ("Push to the fork"). Another machine gets everything with `git pull --ff-only` (§3). **Agents: confirm with the owner before each later push.** |
| **Phase** | **Phase 2 of 6 (headless drawing core): in progress.** Its first two items — Exact view basis and View lines — are done, committed and pushed ([§8.2](#82-phases-26-full-task-lists)). |
| **Running** | Nothing. |
| **Next action** | Phase 2's remaining items, in the plan's order: **Standard layout**, then the **Paper scene builder**, the rest of the **view commands**, **Freshness and Update**, and the **SVG export** the phase's done-when is measured against. |
| **Blockers** | None. |

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

### Phase 1: integration base (done, 2026-10-27)
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
- [ ] **Decide the drawing storage model** before #3891 ([§6.2](#62-decide-3891-decided-on-2026-10-27--follow-upstream)). Upstream
  dropped the separate drawing file; the beta's D1 and item 6 assume `.scdwg`.
- [x] **The owner chose to follow upstream** on 2026-10-27, so the beta takes in-design storage and
  `.scdwg` is gone. D1 and beta item 6 are superseded ([§8.1](#81-what-beta-means-and-the-decisions)).
- [x] Merged **#7316** (1403e678, 909187f7): no net change; its ArcMap work was already kept.
- [x] Merged **#6482** (2c084c49, bca3611a), `mesh_lines`: clean.
- [x] Merged **#7430** (1aaff713, fe8dd804), dashes and PNG: only `Cargo.lock` and `crates/io/Cargo.toml`.
- [x] Merged **#3891** (36cf48a3, 69261367): 20 regions in 9 files. See the commit message.
- [x] A regression test for the `user_materials` cap (b0f2a86e), proved by mutation.
- [x] Merged **#5867** (cce39b40, unchanged since the Mac's re-fetch) as **09c3052f** on the
  laptop: 4 regions in 4 files, all additive. Its one 3-argument `checked!` call site now uses
  `main`'s 2-argument macro; the spec already had `.contract(&CONTRACT)`.
- [x] Wrote the schemas for the 10 `drawing.*` commands (**3232bbb0**, [§6.6](#66-contracts-owed--done-3232bbb0)).
  The coverage test's list of commands without schemas went from 30 to 20; the 20 left predate
  this branch.
- [x] Merged **#6788** at its new head **9e6b1bb2** as **4f18ddea**: clean. The author's update
  fixes both findings from our review (§8.5).
- [x] The checks the Mac left for the laptop (§8.6) all pass on Windows:
  - `cargo xtask wasm`: 12 crates build, with no warnings `main` doesn't already have;
  - the CLI test `run_results`: 18 passed;
  - clippy on geom, doc, render, io, engine and ui-egui with all targets: only the two Windows-only
    warnings `main` already has;
  - the three tests that fail on macOS pass here: prism `a_ring_cut…` (4 s), robustness seeds 36
    (23 s) and 71 (7 s);
  - `hostile_params_never_panic` passes (577 s, pinned to 4 cores).
- [x] `cargo build -p solidcraft`, then a launch check over the control channel on port 27900 with a
  copy of `plate-two-bores`. drawing.list, two drawing.view.snapshot calls (front fitted at 2:1,
  top at 1:1), drawing.create, drawing.properties.set, drawing.inspect, drawing.close and
  engine.describe (which serves the new schemas) all answer. Screenshot taken, then the app quit
  with `app.quit {force: true}`.
- [x] `BETA-BASE.md` lists every merged SHA in order.

### Phases 2–6 (not started)
- [ ] **Phase 2:** headless drawing core: session, records, views, layout, paper scene, commands, Update, SVG.
- [ ] **Phase 3:** text, notes and dimensions.
- [ ] **Phase 4:** DXF, PDF and PNG export.
- [ ] **Phase 5:** Drawing workspace UI.
- [ ] **Phase 6:** hardening: tests, hostile parameters, a scenario, screenshots, performance, docs.

Full task lists are in [§8.2](#82-phases-26-full-task-lists).

### Phase 7: Arrange (after the drawings beta)
- [ ] **Arrange the cut pieces for making them.** Lay out every piece either flat on stock sheets
  for a CNC machine (2D: router, laser, plasma, waterjet), or by its footprint on a 3D printer's
  build plate. Asked for by the owner on 2026-10-27, to come **after drawings are in the
  package**. Plan, related upstream work and open questions: [§8.8](#88-arrange-planned-after-drawings).

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
| **B: Mac** | macOS 27.0.1 (Darwin 27.0.0), Apple silicon, 14 cores, 48 GB; repo at `~/dev/solidcraft`, trial worktree at `~/dev/sc-trial` | Added 2026-10-27. Rust 1.99.1 + wasm32, git 2.55, `gh` signed in as DevonHope. **The NAS copy at `~/Library/CloudStorage/SynologyDrive-Software/solidcraft` is not a git repo** and must not be built in: Synology filters dot-folders, so `.git` never arrives. Builds use `CARGO_BUILD_JOBS=8`. A clean `cargo build -p solidcraft` takes **3 m 55 s**; an incremental relink about 5 m 30 s at 2 jobs. A dev build of **26140353** is at `~/dev/solidcraft/target/debug/solidcraft`; launch it with the Bash lines in [Appendix A](#appendix-a-launchers), which read `SOLIDCRAFT_SETTINGS_DIR`, `SOLIDCRAFT_RECOVERY_DIR`, `SOLIDCRAFT_CONTROL_DIR` and `SOLIDCRAFT_UPDATE_CHECK`. |

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

- **After each phase, install the app again** (owner, 2026-10-28). Build with
  `packaging/macos/package.sh --arch aarch64`, then copy `SolidCraft.app` from the DMG into
  `/Applications` with `ditto`. Each round is about 16 minutes and about 7 GB of `target`.
  Note when telling the owner it is ready: **Phases 2, 3 and 4 are headless**, so the installed
  app looks identical after each of them. The Drawing tab lights up in Phase 5 only
  (§8.2, "Workspace wiring"). What is visible before then is Phase 2's SVG export and the
  `drawing.*` commands over the control channel.
- An installed beta claims `.sc`, `.solidcraft` and `.solvecraft` as **Editor**, and
  `.step/.stp/.igs/.iges/.3mf/.stl` as **Viewer**, so it becomes the default for all of them
  (§8.9). Turn its update check off: `check_updates` in
  `~/Library/Application Support/SolidCraft/app.ron`, read at `ui-egui/src/update.rs:593`.

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
- **Engine tests: always `--lib`.** `cargo test -p solidcraft-engine <filter>` builds the lib test
  binary *and* all ten `crates/engine/tests/*.rs` integration binaries, each a full debuginfo link.
  That is what was running when the Mac rebooted. `--lib` is one link and holds the drawing, bom,
  metadata and joints tests. Add `RAYON_NUM_THREADS=2`: `RUST_TEST_THREADS` does not bound rayon.
- **Never wait with `pgrep -f "cargo test"`.** The waiting shell's own command line contains that
  string, so the loop never ends and spins a core. Use `pgrep -f '[c]argo test'`, or wait on a
  marker the job itself prints.
- **`${PIPESTATUS[0]}` is a bash-ism** and is empty in zsh, so piped exit codes read as success.
  Use `setopt pipefail`, or judge a run by its output.
- **Disk.** A shared `CARGO_TARGET_DIR` across two checkouts reached 51 GB, 36 GB of it
  `target/debug/incremental`, which is regenerable cache.
- **Never filter on `robustness_goldens` alone.** It matches **104 tests**, one per robustness
  seed, and some seeds take minutes each; upstream CI spreads them over shards. On the laptop
  that filter ran for 40 minutes before it was stopped. Name the seeds:
  `robustness_goldens::seed_36`. Test filters are substrings, so check what a filter matches with
  `<test binary> --list | grep -c <filter>` before running it.
- **`hostile_params_never_panic` starts one thread per core** (16 on the laptop), whatever
  `RUST_TEST_THREADS` says. It takes about 10 minutes. On Windows, run the test binary directly
  with its process pinned to 4 cores at below-normal priority (PowerShell: `Start-Process …
  -PassThru`, then `.ProcessorAffinity = 0xF` and `.PriorityClass = "BelowNormal"`).
- **The test binary can be run directly** once built
  (`target/debug/deps/solidcraft_engine-<hash>.exe --test-threads=2 <filters…>`, from
  `crates/engine`): no cargo, no rebuild, several filters at once.
- **Laptop build times at 4 jobs:** the engine `--lib` test binary about 13 minutes from a change
  in engine (5 minutes for a small change), `cargo build -p solidcraft` about 10 minutes, the CLI
  test `run_results` about 17 minutes, `cargo xtask wasm` about 1 minute when warm.
- **PowerShell 5.1 strips the double quotes** from JSON passed to a native program, so
  `control.py` calls with JSON arguments go through Git Bash, not PowerShell.
- **Synology Drive syncs the laptop's repo folder, `target/` included** (the Mac saw the laptop's
  Windows binaries in the NAS copy). That copies tens of GB of build output to the NAS and likely
  explains cargo's "hard linking files in the incremental compilation cache failed" warnings.
  **Owner's call:** exclude `target` from the sync in Synology Drive's settings. Agents don't
  change it.
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

### 6.2 Decide #3891: DECIDED on 2026-10-27 — follow upstream

**The owner chose to follow upstream, and #3891 is merged (69261367).** What follows is the case
as it was put, kept because it records why.

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

### 6.3 Merge #7430 (dashes and PNG, which includes #7255) — done, fe8dd804
`git merge --no-ff prs/pr-7430`. It has not been tried yet. Then run the targeted geom and io tests (`cargo test -p solidcraft-io paper`).

### 6.4 Bring in #6482 (`mesh_lines`) — done, bca3611a
- Cherry-pick or merge `prs/pr-6482`.
- The earlier research found it conflicts only in `crates/engine/src/cmd/view_dxf.rs`.
- Test with `cargo test -p solidcraft-engine view_dxf`.

### 6.5 Merge #5867, then #6788 — done, 09c3052f and 4f18ddea
- #5867 (cce39b40): 4 additive regions, in `engine/src/lib.rs` and `render/src/lib.rs` (`mod`
  lines), `engine/src/schema/mod.rs` (exports) and `engine/src/tests.rs` (the hostile-parameter key
  list). The commit message records each.
- #6788 (9e6b1bb2): clean. The author answered our review with this head, and asked the owner on
  2026-10-27 to look again (§8.5).

### 6.6 Contracts owed — done, 3232bbb0

**Done on the laptop, 2026-10-27:** `crates/engine/src/schema/drawing.rs` (with
`drawing_tests.rs`) holds a parameters type and a reply type for each of the ten commands, and
`cmd/drawing.rs` registers them and builds its replies from the reply types. The JSON is unchanged,
and the existing drawing tests pass untouched. The coverage list is down to the 20 commands below,
none of them ours. What follows is the task as the Mac recorded it.

`every_command_has_input_and_output_schemas` (in `crates/engine/src/schema/coverage_tests.rs`)
**passes**: a missing schema is only a warning today, and becomes an error under
`SOLIDCRAFT_SCHEMA_RULE_STRICT=1`. It nevertheless names **30** commands with no schemas, and
**10 of them are #3891's**, which this branch now owns:

```
drawing.create  drawing.list  drawing.open  drawing.close  drawing.delete
drawing.inspect  drawing.set  drawing.properties  drawing.properties.set
drawing.properties.remove
```

The other 20 predate this work (`view.*`, `solid.cone`, `canvas.to_sketch`, `sketch.arc3d`,
`sketch.drawing_bounds`, `component.cycle_color`, `component.copy_bodies`, `occurrence.reparent`,
`file.send_to_slicer`, `file.open_in_easyprint`, `inspect.colored_components`,
`document.revision`). They are not ours to fix here.

**Do not add any of them to `crates/engine/src/schema/unconverted.txt`**: that file says "Never add
one: a new command comes with both schemas", and it only shrinks. Write the contracts instead. The
pattern is `.contract(&C)` plus `.output(output_schema_for::<T>)`, with `crate::checked!("id",
handler)`; `engine/src/cmd/inspect.rs` and `features.rs:41` are the examples, and
`docs/book/src/dev/schemas.md` is the long form. Take each command's output shape from the `→ …`
text already in its `.params()` string, so upstream's own contracts drop in later.

### 6.7 Two integration gaps, recorded not fixed

Neither blocks Phase 1: nothing opens a drawing yet.

1. **`Part::Model` commands are refused while a drawing is open.** #3891 adds
   `CommandSpec::allowed`, which refuses a `Part::Model` command whenever `open_drawing` is set.
   Commands `main` added after the drawings base default to `Model` and do not carry
   `.any_part()`, so global commands (much of `engine.*`, file and settings) will be blocked once
   a drawing can be opened. The merge added `.any_part()` only where #3891 itself did.
2. **A drawing's properties use the design's key table.** `engine/src/cmd/metadata.rs::reading`
   is shared with `drawing.properties`, and it lists `metadata::keys(Owner::Design)`. `Owner` has
   only `Design` and `Component`, so a drawing is offered a design's keys. Upstream's own version
   used a flat `WELL_KNOWN` list, which no longer exists on `main`, so this is the closest
   equivalent. A `Owner::Drawing` variant would be the real fix.

### 6.8 Close out Phase 1 — done on 2026-10-27

Every step below was done on the laptop (§2 and the session 4 log). Kept as the recipe for
re-checking a rebuilt base.

1. `cargo build -p solidcraft`. A debug build takes about 15 minutes from clean on the laptop.
2. Targeted tests on geom, doc and io, plus the engine filters you touched.
3. Launch with `plan\run-dev.cmd`, or the Bash lines in Appendix A.
4. Take a screenshot with `control.py` (Appendix C) by calling `ui.screenshot` with `{"path": "<file>.png"}`. Then close the app.
5. Write `BETA-BASE.md`: base `upstream/main`, then each merged PR with its SHA, in order.
6. Tick Phase 1 here, log it, and hand off.

**If a test link fails with `LNK1201`** (Windows, writing the program database): delete that test binary's stale `.pdb` in `target/debug/deps` and rerun. It's not a full disk.

Then start **Phase 2** ([§8.2](#82-phases-26-full-task-lists)).

### 6.9 Merge upstream `main` — done, 51f0c2d0

**Done on the laptop, 2026-10-27,** exactly as measured below, with one more change git couldn't
see: `main`'s new `crates/doc/src/placement_reuse.rs` names every `Document` field in
`same_geometry_inputs`, so `drawings` needed a decision. It is **compared**, like every other
non-placement field: never wrong, and at worst it skips a reuse when a drawing and a placement
change together. `cargo check --all-targets` on doc, io, engine, ui-egui, mcp, the app and the CLI
then passed. Tests: doc 96, io 71, engine `--lib` 135. Every line `main` added in the 22 files both
sides changed is in the result, apart from the two rewordings. The merge commit records it all.

The owner chose to finish Phase 1 first, then merge `main`. Measured with `git merge-tree` from
4f18ddea against `upstream/main` **9fb41f83**: **144 commits** since the branch's base (23734142),
**5 conflicting files, every one both sides adding code in the same place**:

| File | Ours (the drawings work) | `main` | Resolution |
|---|---|---|---|
| `crates/doc/src/document.rs` | `differs_only_in_drawings`, `drawing`, `put_drawing` | `differs_only_in_appearance` | keep both |
| `crates/doc/src/lib.rs` | re-exports `BomType` | re-exports `WorldNames` | keep both |
| `crates/engine/src/cmd/edit.rs` | `.any_part()` on undo, redo and checkpoint; `latest`, `nothing` and the per-part `go_to` docs | the new `edit.repeat` command and its contract | keep both |
| `crates/engine/src/lib.rs` | "drawings changed: don't evaluate" | "appearances changed: don't evaluate" | keep both conditions, and say both in the comment |
| `crates/engine/src/schema/mod.rs` | `pub mod drawing; pub mod drawing_view;` | `pub mod edit;` | all three, sorted |

**Checked in the dry run's tree:**
- **Nothing we did is lost.** `check_design_numbers` is still called (3 times) and `take_drawings`
  still runs first. `main`'s `checked!` macro is unchanged, so the call sites fixed here still fit.
- **No duplicated work.** None of #6482, #6788, #7316, #3891 or #5867 has landed on `main`, and
  nobody upstream has written the `drawing.*` schemas.
- **#7325: new designs save as `.sc`.** `.solidcraft` files still open and keep their name on Save,
  so the test parts and the one drawing test that names `plate.solidcraft` are unaffected.
- **From the follow-up list:** `main` has fixed the stray `>>>>>>>` marker in `AGENTS.md`. The two
  Windows-only clippy warnings are still there, so that small upstream PR is still open to take.
- **Gap 1 in §6.7 grows.** `main` added 14 command ids since the base, none with `.any_part()`. For
  the sketch and solid commands that's right while a drawing is open; `edit.repeat`,
  `view.render_stage` and `view.shots_strip` are app-wide and will be refused too.

**After merging:** `cargo check --all-targets` on doc, io and engine; the doc `drawing` and
`format` tests; engine `--lib` `drawing`, `schema::` and `edit`; then `cargo build -p solidcraft`
and a short launch.

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


### Session 3: Mac (machine B), 2026-10-27 afternoon (UTC-4)

Carrying on from session 2 after the owner answered three questions: run the owed gates gently,
**follow upstream on storage**, and push (that push covered 98bd4a46 only).

**The three gates owed on the drawings merge all pass**, run one at a time at 2 jobs under
`nice -n 19`: `cargo check -p solidcraft` clean in 1 m 16 s; `clippy -D warnings` on geom and doc
clean; engine `--lib` drawing 11, bom 19, metadata 17, all 0 failed. The machine stayed near load 5.

**Four merges landed**, each measured first with `git merge-tree`, which needs no working tree:
- **#7316** (909187f7): clean, and **no net change** — its ArcMap work was already kept as "ours".
- **#6482** (bca3611a): clean.
- **#7430** (fe8dd804): only `Cargo.lock` and `crates/io/Cargo.toml`, both dependency lists; kept
  both sides and let cargo regenerate the lock. `xtask licences --check` passes.
- **#3891** (69261367): 20 regions in 9 files, each combining both sides. The commit message
  records every file.

**The near-miss worth remembering.** In `doc/src/format.rs`, upstream replaced
`check_design_numbers` with `DESIGN.check_numbers(&v, "metadata")`, and it was first resolved as an
equivalent refactor. It is not: main's also caps `user_materials` and sends each `physical_card`
through `validate_card`. **This is the same check session 1 had to restore.** It is now kept, with
drawings keeping their own family rule, and **b0f2a86e adds the first test for it**, proved by
mutation: with `DESIGN.check_numbers` swapped back in the test fails with a serde type error,
because the design is decoded before the cap is applied.

**Other things the checks caught, not the author:**
- `cargo check` without `--all-targets` hid two errors in `io/design.rs`
  (`PackageWriter::new` and `PackageReader::part`, both removed by one side or the other).
- `same_but_features` destructures `Document` exhaustively, so `drawings` had to be decided on.
  It was first *compared*, which would have re-solved the joints after every drawing edit; its only
  caller is `solve_joints`. It is now ignored, and the joints tests pass 161 of 161.

**Three engine tests fail on macOS** and are platform divergence, not these merges: see §8.6. Seed
36 is proven by producing the Windows golden exactly.

**State left:** `beta/drawings` at **b0f2a86e**, tree clean, nothing running. Everything after
98bd4a46 is on this Mac only and needs the owner's OK to push. The full engine `--lib` suite is
2595 passed / 3 failed (the three above); doc 467, io 190, joints 161, all 0 failed; fmt, lists,
skills, format-pending and licences pass.

### Session 4: laptop (machine A), 2026-10-27 evening (UTC−4)

**The owner's ask:** "pull the latest updates from the fork and get to work on resolving phase 1",
then "get yourself up to speed with the progress updates first".

**About 20:30: caught up.**
- `git pull --ff-only` brought the Mac's sessions 2 and 3, 88f65b4d → a5be5764. Read this file,
  `BETA-BASE.md` and every commit message.
- Re-fetched the PR heads: #5867 still cce39b40, so §6 still applied; **#6788 had a new head**,
  9e6b1bb2; `upstream/drawings` still ebafe074.

**About 20:35: #5867.**
- `git merge-tree` showed 4 conflicting files, down from the 12 the Mac measured before #3891 was
  in. All 4 were additive (§6.5). Fixed the one 3-argument `checked!` call site.
- `cargo check --all-targets` on geom, doc, render and engine: clean apart from the known Windows
  warnings, 2 m 13 s.
- geom `drawing` 38, doc `drawing` 19 and render `visual_style` 1 passed.

**About 20:37 to 21:30: a mistake that cost 40 minutes.**
- The engine run's filters included `robustness_goldens`, which matches all 104 seed tests, and the
  output filter hid which test was running. The build took 13 minutes; the seeds then ran for 40.
- The owner asked why it was taking so long. The test process was stopped and nothing was left
  running. §5 now warns about it.
- Re-ran from the built test binary with exact filters: engine `drawing` + `schema::` 70 passed in
  0.3 s. **The three macOS failures pass on Windows** (prism 4 s, seed 36 23 s, seed 71 7 s).
  `hostile_params_never_panic` passed in 577 s, pinned to 4 cores.
- **21:41:** committed the merge as **09c3052f**.

**21:41 to 21:49: the contracts and #6788.**
- Wrote `schema/drawing.rs` and its tests, and switched `cmd/drawing.rs` to `checked!`, `.contract`
  and `.output`, with replies built from the reply types (§6.6). The first build failed: the
  existing drawing tests had their `json!` from `use super::*`, which the change removed. Fixed with
  their own import.
- engine `drawing` + `schema::` 74 passed. The coverage list went from 30 to 20. fmt and lists pass.
  Committed as **3232bbb0**.
- Read #6788's new head: both review findings addressed (§8.5). It merged cleanly as **4f18ddea**,
  and doc `tolerance` + `display_units` 35 passed.

**About 21:55: clippy.** On geom, doc, render, io, engine and ui-egui with all targets, 3 minutes:
only the two Windows-only warnings `main` already has.

**About 22:00: the owner asked how far behind `main` the fork is.**
- The fork's `main` was 157 commits behind; `beta/drawings` was 144 behind `upstream/main`
  9fb41f83.
- A dry-run merge found 5 additive conflicts, nothing lost and nothing duplicated (§6.9).
- **Decision (owner):** "Finish Phase 1 first", then merge `main`.

**About 22:00 to 22:30: the rest of Phase 1.**
- `cargo xtask wasm`: 12 crates in 1 minute, with no warnings `main` didn't already have.
- CLI `run_results`: 18 passed, 17 m 31 s, nearly all of it building the CLI.
- `cargo build -p solidcraft`: 10 m 30 s.
- **22:27 launch check:**
  - The dev build ran with the dev data folder on port 27900, with a copy of `plate-two-bores`
    (`designs\phase1-check.solidcraft`).
  - Over the control channel: `drawing.list` (none); `drawing.view.snapshot` front (fitted at 2:1,
    boundary 200 × 40 mm on A3) and top at 1:1 (100 × 80 mm); `drawing.create` (one A3 landscape
    sheet, units mm); `drawing.properties.set` part_number; `drawing.inspect`; `drawing.close`;
    `drawing.list` (one, closed); and `engine.describe drawing.create`, which serves the new schemas.
  - Screenshot: `%USERPROFILE%\SolidCraft-dev\screens\phase1-launch.png`.
  - Then `app.quit {force: true}`; the app exited 0. The copy was not saved.

**Side request (owner):** add an **Arrange** feature to the to-do list, after drawings. It is now
Phase 7, with its plan in §8.8. Found while writing it:
- RFC #3820 already proposes the 2D nesting;
- the ribbon already has an "Arrange" group.

**About 22:40: Phase 1 notes** committed as **f572c8c7**.

**About 22:45 to 23:30: `upstream/main` merged (51f0c2d0).**
- Same 5 conflicts as the dry run, resolved as §6.9 says.
- The first `cargo check` failed: `main`'s new `placement_reuse.rs` destructures `Document`
  exhaustively and didn't know `drawings`. Decided to compare them (§6.9).
- The second check passed on doc, io, engine, ui-egui, mcp, the app and the CLI, all targets.
- A line-by-line check of the 22 files both sides changed found nothing of `main`'s missing.
- Built the three test binaries (13 m 46 s), counted every filter's matches first, then ran them:
  doc 96, io 71, engine 135, all passing.
- `cargo build -p solidcraft` on the merge (11 m 54 s), then a short launch with a copy of the
  plate (`designs\main-merge-check.solidcraft`; `.solidcraft` files still open after #7325). The
  window now says **v0.25.1**, `main`'s version. Over the control channel:
  - `drawing.view.snapshot` right (fitted at 5:1);
  - `drawing.create` and `drawing.inspect`;
  - `edit.undo` inside the drawing answered "nothing to undo in this drawing (close it to undo in
    the model)";
  - `drawing.close`, then `edit.undo` undid "Create Drawing", and `drawing.list` was empty.

  So per-drawing undo, the merged `edit.rs`, works in the app. The screenshot is
  `%USERPROFILE%\SolidCraft-dev\screens\main-merge-launch.png`. Then `app.quit {force: true}`.
- **Leftover:** the first launch check's forced quit left an autosave of its throwaway copy in
  `SolidCraft-dev\recovery`, so the dev build now opens with "Recover unsaved design?" for
  `phase1-check.solidcraft`. **Press Discard.** It holds nothing of value, and only the dev data
  folder has it. Next time, end a launch check by undoing back to a clean design before
  `app.quit`, or discard it on the next launch.

**State left:** Phase 1 done and `main` merged. `beta/drawings` is the notes commit on top of
51f0c2d0, and **pushed to the fork** with the owner's OK (a5be5764 → 56861a46, then this note).
Nothing is running. Next is Phase 2.

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
- **D1 — SUPERSEDED on 2026-10-27 by the owner's decision to follow upstream.** A drawing now
  lives inside the design's file, and `.scdwg` is gone with `DRAWING_EXT`, `read_drawing_file`,
  `write_drawing_container`, `file_kind`, `FileKind` and `is_drawing_path` (#3891, 472a3306:
  "bherbruck's proposal, Cameron's choice"). **Beta item 6 goes with it**: "save and reopen the
  drawing (`.scdwg`)" is now "the drawing is saved in the design". The original D1 read:
- ~~**D1. A drawing is its own document tab, linked to the part.**~~ This is upstream's W11 model: separate linked files, with undo per file, compatible with #3891.
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
- [x] **Exact view basis.** Done 2026-10-28 (machine B), uncommitted. Add `View::orthographic(&Plane)` to `mesh_lines` (right = x, up = y, no perspective), fed by #4817's `NamedView::basis()`.
  - Going through the camera tilts top and bottom views by 1e-6 rad, which leaves stray stubs on vertical edges.
  - Test a view where faces are seen edge-on.
- [x] **View lines (#5867's planned V3).** Done 2026-10-28 (machine B), uncommitted.
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

**Phase 2, the first two items — done on machine B, 2026-10-28, not yet committed.**

- `View::orthographic(&Plane)` in `mesh_lines/projection.rs`, beside `from_camera`. It returns
  `Result` and **checks** the basis (finite, unit, square to 1e-9) rather than squaring it up: the
  basis comes from a saved record, and axes that are off would scale the depth `project` returns
  so `depth_eps` would stop meaning what it says. `Plane`'s own doc already states its axes are
  orthonormal, so this enforces an invariant the type claims.
- `ViewLines`, `chord_mm` and `chain` in `engine/src/drawing_view.rs`, beside `extents`;
  `drawing.view.snapshot` now returns `lines` with `status: approximate`.

Three decisions worth keeping:

1. **Every placed body**, `prepare(s, CMD, chord, |_| true)`, not `view_dxf`'s
   `|b| s.body_visible(&b.name)`. The module doc is explicit: hiding a part in the model never
   changes a drawing.
2. **Chaining was written fresh** rather than reusing `view_dxf::connect`, which does the same
   degree-two walk but is tied to `DxfEntity` and only chains hidden lines. Collapsing the two is
   worth doing later, from the DXF side, as its own change.
3. **`tangent_edges: show` is refused** (`not supported yet: … (#5866)`), following the
   shaded-views refusal above it. `prepare` has no switch — the classification is baked into the
   placed body's mesh — so the option would otherwise have silently done nothing. The command had
   been accepting the parameter while nothing computed lines at all.

Verified: 11 `drawing_view` tests and 8 `mesh_lines` tests pass, clippy `--all-targets` is silent,
and two mutation checks hold — negating the view's `back` fails the depth-sign assertion, and
returning loose segments from `chain` fails the polyline assertion. `cargo xtask ci` has **not**
been run.

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
- **Arrange** (Phase 7): cut pieces nested on stock sheets for CNC, or footprints on a printer's
  build plate ([§8.8](#88-arrange-planned-after-drawings));
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

**Replies so far:** all four developers answered, and all four were replied to on 2026-10-28
with the owner's explicit OK. Each claim was verified by reading the merged code on
`beta/drawings` 3bf352b7, not by re-running their benchmarks or fuzz runs.

- **#2457** (SychPL, merged): `Trim::margin` gone, `Trim::near` in its place, the rounding test
  present, `trim_samples` and the scene-extent cap bounded. Replied: issuecomment-6470833664.
- **#3877** (CamWight, merged): both named tests present, `place_cells` shared by `validate` and
  `layout`, `MAX_COORDINATE_MM` clamp in `cell.rs`. "Field" naming deferred until one becomes a
  file key, which we accepted. Replied: issuecomment-6470834223.
- **#4817** (CamWight, merged): `MIN_PART`/`MAX_PART` 1e-6/1e6, `boundary` returns `Result`,
  `basis_looking` re-orthogonalises with its test. Replied: issuecomment-6470834713.
- **#6788** (armin0411, OPEN): all five points addressed at `9e6b1bb2` — `ToleranceText::Limits`
  its own variant with `plain()` omitting the nominal, the warning gated on `moved`, stacked
  decimals padded, the sign test added, the repeat count through `format_number`. Re-review
  posted as COMMENT: pullrequestreview-5683740904. Not confirmed by us: the 100,000-case fuzz
  runs and the eight planted faults, and the branch is not rebased onto current `main`.

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
| **3 engine tests fail on macOS** (machine B) at b0f2a86e: `prism_local_tests::a_ring_cut_round_a_counterbored_hole_in_a_chamfered_disc`, and `robustness_goldens` seeds **36** and **71**. | **Platform divergence in the kernel, not the merges.** Seed 36 is proven: macOS produces the **Windows** golden to six decimals (area 9964.083286 against 9964.083285, faces 21, edges 45, the same lumps error), and `tests/robustness/goldens.json` carries `platforms` overrides for **windows only**, so macOS falls back to the Linux-recorded shared value. Seed 71 differs in area by 0.0002% with faces 20→19 and edges 52→50, a tolerance-driven topology flip; prism is 0.108% off a 0.1% tolerance. Neither test file was touched by any merge, `crates/kernel` and `crates/sketch` are byte-identical to the base, and #3891 changes no geom, kernel or sketch code. The one geometry change in the base is #4817's rewrite of `Seg2::polyline` through a new `chords()`, which is output-identical: the arc step's old `.max(1e-6)` could never bind, because `tol >= radius * 1e-6` forces the step above 2.8e-3, and the other branches apply the same `max(1e-9)` once instead of per branch. **Confirmed on the laptop, 2026-10-27, at 09c3052f: all three pass on Windows** (prism 4 s, seed 36 23 s, seed 71 7 s), so they are macOS-only. Recording macOS goldens is upstream's call, so none were blessed. |
| **Owed on a machine with the headroom** (the laptop): **all done 2026-10-27.** | `cargo xtask wasm`: 12 crates build, no new warnings. CLI `run_results`: 18 passed. Clippy on geom, doc, render, io, engine and ui-egui with all targets: only the two Windows-only warnings `main` has. The three tests above pass. |
| The dev window title says "v0.24.0" (before the `main` merge) or "v0.25.1" (after it, 51f0c2d0). | Expected: it is `main`'s version number, which can match an installed release. Tell the windows apart by the port (27900) and the data folder. |
| The dev build opens with "Recover unsaved design?" | A launch check closed with `app.quit {force: true}` left an autosave in `SolidCraft-dev\recovery`. Press Discard. Only the dev data folder is affected. |

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
| #5867 | 5bb34f49 (**gone**) | **cce39b40** | open, stacked on #3891; **merged here** as 09c3052f |
| #6482 | 2c084c49 | 2c084c49 | unchanged; base `main` |
| #6788 | c70ec84e | **9e6b1bb2** (pushed 2026-10-27 evening, after our review) | open, base `main`; **merged here** as 4f18ddea |
| #7255 | 8f500113 | d2fcae51 | open, inside #7430 |
| #7316 | 55389724 (merged here) | 1403e678 | open; restacked on new `drawings` |
| #7430 | 4063a536 | 1aaff713 | open, restacked |

**Re-checked 2026-10-27 evening (laptop):** `upstream/drawings` is still **ebafe074**;
`upstream/main` is **9fb41f83**, 144 commits past this branch's base; the fork's `origin/main` is
157 behind it.

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

### 8.8 Arrange: planned after drawings

**The owner's request (2026-10-27):** "an arrange feature that lays out all cut pieces for a cnc
machine into a 2d plane or a 3d footprint for cutting or printing", working much like the arrange
tools in other CAD programs, as **the step after drawings**. Nothing here starts before the drawings
beta is done.

**What it does:**
- **2D, for cutting.** Choose the pieces (bodies, or each sheet-metal part's flat pattern). Each is
  laid flat on its cutting face and packed onto one or more stock sheets of a given size.
  - Settings: spacing between pieces (the cutter's kerf or tool diameter), a margin at the sheet's
    edges, how pieces may turn (not at all, by 90°, or freely), grain direction kept, and how many
    of each piece.
  - Results: the sheets used, the yield (area used ÷ area bought), and a DXF or SVG per sheet for
    the CAM program, through the drawings' paper writers (Phase 4).
- **3D, for printing.** Choose the bodies, set each on a face (or keep its orientation), and pack
  their footprints onto the printer's build plate, onto more plates when they don't fit. The result
  is exported as 3MF with plates, for the slicers that Send to Slicer already hands designs to.
- **Like everything else:** each step is a command with input and output schemas (an `arrange.*`
  family; no command uses that prefix today), it works from the CLI and over MCP, and it is saved in
  the design the way drawings now are. It should update when the pieces change, as drawing views do.

**Related upstream work** (read before designing; claim through the owner's issue, not a
competing PR, D5):
- **[#3820](https://github.com/tangency-labs/solidcraft/issues/3820) [RFC] Cut lists from the
  model: stock sizes, nesting onto boards and sheets, and build plans** (emlama, open, unassigned,
  needs-triage). Its "nesting onto boards and sheets" **is** the 2D half of this feature, so the 2D
  arrange should build on it, or be offered to it. Its first slice is draft PR
  [#3995](https://github.com/tangency-labs/solidcraft/pull/3995), "Stock size: a body's true length
  × width × thickness, along the part (#137)".
- **[#1626](https://github.com/tangency-labs/solidcraft/issues/1626) Research Request for Plugin:
  3D Printing** (CamWight, research only, no code wanted). Its scope includes "orientation and plate
  layout": the 3D half.
- **Manufacturing / CAM** is milestone 9 in ROADMAP.md, with proposals
  [#1153](https://github.com/tangency-labs/solidcraft/issues/1153) (2.5D milling, toolpaths and
  G-code) and [#1621](https://github.com/tangency-labs/solidcraft/issues/1621) (CAM research).
  Arrange is the step before toolpaths, not CAM itself.
- **Already in SolidCraft:**
  - the sheet-metal flat pattern and its DXF;
  - section curves (`geom/section.rs`) for a planar piece's outline;
  - 3MF export (`io/src/threemf.rs`) and Send to Slicer (`file.send_to_slicer`);
  - after the beta, the paper scene and its SVG, DXF and PDF writers.

**How it might be built** (a sketch, to be designed properly when it starts):
1. Stock sizes and piece outlines first: #3995's true part size, then each piece's flat outline
   with its holes.
2. Packing by bounding rectangles (simple, fast and predictable) as the first version, then true
   shape nesting, which fits pieces into each other's gaps, as the second.
3. The 3D plate layout, reusing the same packing on footprints.

**Clean-room rules for this feature:**
- Design it from this description, in our own words. Never copy another program's arrange UI, its
  text, its icons or its layouts, and never run commercial CAD to produce reference layouts or
  numbers.
- Some well-known nesting code is GPL or LGPL, including what the copyleft slicers use. Check a
  library's licence before reading anything of it, and add only MIT or Apache licensed crates
  (`cargo xtask licences --check`).

**The name "Arrange" is taken in the UI.** The Model tab's ribbon already has a group called
**Arrange** (Move / Copy, Mirror, Pattern), as the laptop's screenshot of 2026-10-27 shows. The new
feature needs a name and place that don't clash with it, such as its own group or workspace for
making parts. Decide this with the owner; its command ids must not reuse an existing prefix either.

**Open questions for the owner, before starting:**
- 2D first, 3D first, or both together?
- Is an arrangement saved in the design and kept up to date as pieces change, or is it a one-off
  export?
- Offer to work with #3820's author, or keep it in the beta first (D5)?

---

### 8.9 A beta build upgrades a real design to a format older builds refuse

Seen on machine B, 2026-10-27 15:25: opening and saving `Fairphone6 ESR MagSafe Rear Cover.solidcraft`
in the beta build took it from `solidcraft/38` to `solidcraft/40` with
`format_pending: ["cone-primitive", "drawings", "feature-id-refs"]`.

**Those numbers are from before the main merge (51f0c2d0).** `main` has since folded
`cone-primitive` and `feature-id-refs` into `v40/` and added `v41/`, so `pending/` now holds
`drawings.rs` alone and this branch writes **format 42** with `format_pending: ["drawings"]`.
`main` itself is at 41. The conclusion is unchanged, and the one unknown name is still enough
for the refusal: nothing but this branch opens a design it has saved.

This is the format policy working as designed, not a defect. `crates/doc/src/format/pending/README.md`:
while `pending/` holds any change, the build writes the last folded number + 1, "which no release
reads". `read()` (`crates/doc/src/format.rs:797`) stamps `d.format` and `d.format_pending` on every
design it opens, unconditionally, and `to_json` writes them straight back out.

Who refuses this branch's output (format 40 with those three names):

- the build that wrote the `.bak`, which reads up to 38: refused on the number.
- `upstream/main`: its FORMAT is **also 40** — it has folded `cone-primitive` and `feature-id-refs`
  into `crates/doc/src/format/v40/` — so it passes the number check and refuses on the single name
  it does not know, `drawings`.

This branch is *behind* main on folding: those two changes are still in our `pending/`. Merging
current main would move them into `v40/` and push our FORMAT to 41, which main would then refuse on
the number instead. Catching up does not help compatibility.

Nothing in the tree writes an older format: no env var, flag, preference or command, and no warning
before a save raises a design's format.

The design needs none of the changes it is now tagged with. Ten features (import, move, sketch,
extrude, fillet, sketch, extrude, fillet, sketch, extrude): no cone, no drawings, no pattern, no
mirror, no construction-plane reference, and none of v39's four either (no revolve or circular
pattern, no thread, no 3D sketch arc, no mesh blob — the one blob is a 441 KB STEP). The strings
"cone" and "drawings" occur in the saved file only inside `format_pending` itself. Its content is
what a 38 build would have written; only the tag and the list lock it to this build.

The save writes a `.bak` holding the pre-save content, so format 38 survives exactly one save and a
second overwrites it. Restoring it is not free: the backup predates four real edits made in the same
sitting (6.1 → 6.4 mm, 28.5 → 28.75 mm, 7.1 → 7.25 mm, a fillet 8 → 10 mm, and the sketch re-solved).
Copies of both files are in `~/SolidCraft-dev/designs/rescue-2026-10-27/` on machine B.

So: don't open a design you need elsewhere (machine A, a release build, a synced folder) with a beta
build. Exports are unaffected — a `.step` the beta build writes is ordinary STEP.


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
