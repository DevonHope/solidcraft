# Drawings beta: what this branch is built from

`beta/drawings`, in order. Every entry is a merge commit on this branch; the SHA on the right is
the head that was merged. Status and the task list are in [PROGRESS.md](PROGRESS.md).

| # | Merge | What it brings | Head merged |
|---|---|---|---|
| 0 | — | Base: `upstream/main` | 237341423ce7d76d484a92ea1339b8a3f6ce1525 |
| 1 | c4434126 | `upstream/drawings` as it was on 2026-10-26 | f18616e19c8a16dd1575c522474ed54de34b247d |
| 2 | eaac419a | PR #7316's first head, which carried #4817's first head | 5538972499178c03425dd66a7412fa7936e151b1 |
| 3 | b6312c4a | `upstream/drawings` again, now carrying **#4817, #3877 and #2457** as upstream merged them on 2026-10-27 | ebafe07487df5cdd108e091db939de765645e5eb |
| 4 | 909187f7 | PR #7316, rebased head. No net change: its ArcMap work was already kept as "ours" in 3 | 1403e67803c6f17e2175ade72d9762cb1ffd19dd |
| 5 | bca3611a | PR #6482, `mesh_lines`: the hidden-line module both view commands share | 2c084c49a9eceaf72e7639275aeaac79351d24be |
| 6 | fe8dd804 | PR #7430, dashed lines and PNG, which contains #7255 | 1aaff713aae1e3645752f9de313634cc4090f69e |
| 7 | 69261367 | PR #3891, the drawing session, ten commands and in-design storage | 36cf48a3f64c530a128f5d794f37b7cfa24cc3ab |
| 8 | 09c3052f | PR #5867, `drawing.view.snapshot` (W07 V2), with projected views, measurement on view frames and model extents on a plane | cce39b40d1cb177baad320ddede0e153452fce90 |
| 9 | 4f18ddea | PR #6788, tolerance and dimension text (W09c), the head that answers our review | 9e6b1bb2b5f3f21a6b730a07d8dc7d6d446fe1ed |

That completes the Phase 1 stack (2026-10-27). Next: `upstream/main` (9fb41f83 when measured,
144 commits past the base), as PROGRESS.md §6.9 describes.

**Rebuilding this base from scratch** is possible from the heads above, but the resolutions are
the work: see PROGRESS.md §6.1 for the drawings merge and the #3891 merge commit's own message,
which records every file. Moving the commits is much cheaper than redoing them.

## Not from upstream

Two commits on this branch are ours, not a merge:

- **98bd4a46** and the PROGRESS.md commits: notes only.
- **b0f2a86e**: a regression test that the `user_materials` cap runs before a design is decoded.
  Both the session-1 auto-merge and the 2026-10-27 drawings merge nearly dropped the check it
  guards.
- **3232bbb0**: input and output schemas for #3891's ten `drawing.*` commands
  (`engine/src/schema/drawing.rs`), which #3891 lacks. If upstream adds its own, take theirs.
