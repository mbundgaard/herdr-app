# Native space pop-out windows — owner fork

**Status:** Committed and pushed as **`8c2c08e`** on `origin/feature/space-popout`, based on `bd2d5cca47af94ad049c931e4125849f84829b0c` (package 0.1.15). Windows Rust tests and disposable native smoke tests pass, including with Focus as the main layout. The owner subsequently reported that the feature works very well and explicitly confirmed that selecting a detached space in main activates its pop-out. This is basic hands-on acceptance, not complete UI/platform validation. No packaged fork release is claimed.

Read [the Windows build handoff](windows-build.md) and [fork maintenance policy](README.md). App source and portable app notes belong in this repository; builds, test profiles, logs, and host-specific helpers stay outside Git. No upstream submission or core modification is involved. These are development instructions, not host alignment.

## User-facing behavior

- Right-click a space in the standard menu → **Open in new window**. Focus's expanded space rows already use that menu; no change to its unrelated local edits was needed.
- The native window contains the space's complete tab strip, activity dots, existing terminal panes/splits, add-tab control, and **Return to main window** action. It can be moved/resized independently.
- Selecting a detached space from main raises its existing window; repeated detach does not duplicate it. The standard sidebar adds a detached arrow; the custom Focus sidebar does not yet add that indicator.
- Closing the child or returning it docks the space without ending terminals. Explicit `space.close` remains destructive. Closing main removes its children but leaves core terminals running.
- Detachment is process-local. Reopening restores the spaces in main, not separate windows. Child placement is not persisted.

**Initial scope is terminal-only:** all tabs must contain terminal panes or row/column splits. Agents running inside those terminals are retained as the same processes, not re-created. Mod/layout panes, stacks, and floating blocks are rejected before detach. Adding an unsupported pane through main while detached safely returns the whole space to main. Custom chrome, Files/Changes panels, pane drag/reordering, destructive tab/pane shortcuts, and arbitrary main-window actions are not supported inside the child; unsupported commands report an error instead of falling through to main.

This needs the modified executable. It is not a stock-Herdr plugin. A layout/mod cannot create a native OS window through the original API. The owner authorized independent fork development without Can's approval; the fork's `AGENTS.md` records that exception. Do not submit upstream work unless asked.

## Ownership and implementation

**One authoritative `AppWindow` owns the model, Core connection, terminal entities, polling, documents, mod runtime, and request pump.** `SpaceWindow` is a lightweight native view of that owner, not a second ordinary `AppWindow` or competing desktop process.

| Area | Implementation |
| --- | --- |
| `src/window/popout.rs` | Registry/lifecycle, terminal-only eligibility, native tab strip, contextual command allowlist, target validation, close/dock behavior, pane focus, terminal context menu |
| `src/window.rs` | Child routing and status; main render/selection excludes detached content; child commands restore main focus/selection afterward; document continuations stay on main |
| `src/window/pane_size.rs` | Main hidden-tab fitting skips detached spaces, avoiding competing terminal resize calls |
| `src/mods.rs`, `src/window/floats.rs` | `WindowSnapshot` owns snapshots and revisions; loaded mods/callbacks capture their owner; terminal-address lookup remains app-wide |
| `defaults/mods/menu/main.js`, `defaults/mods/spaces/main.js` | Detach/return menu actions and standard sidebar indicator |
| `src/commands.rs`, `src/control.rs`, `src/main.rs` | Commands, explicit child automation, status inspection, child screenshot targeting |
| `src/SKILL.md`, `defaults/AGENTS.md`, `defaults/herdr.d.ts`, `docs/commands.md` | Updated guides, `SpaceInfo.detached`, generated command reference |

Important safeguards:

- GPUI draws synchronously during `open_window`; a readiness gate prevents reading the still-borrowed owner during that first draw. This fixed a reproduced Windows startup panic.
- Child actions normalize aliases/shorthand and validate arguments before checking space ownership. Foreign pane/tab IDs or terminal addresses are rejected.
- Main selection/focus is restored after child actions; pane navigation cannot enter main chrome. Main sync must not fit/blur terminals displayed by children.
- Child mutations suppress document flushing until execution returns to main, so asynchronous document continuations retain the main window context.
- Child rendering loads no additional mods. App-wide request/notification machinery is not duplicated. This does **not** make two independent ordinary `AppWindow` instances safe.

## Automation

These commands exist in the fork; substitute a real space ID:

```text
herdr-app do space.popout --id SPACE_ID
herdr-app do space.dock --id SPACE_ID
herdr-app do space.window '{"id":"SPACE_ID","command":"tab.next"}'
herdr-app shot child.png SPACE_ID
herdr-app status
```

JSON shell quoting depends on the caller; subprocess argument arrays avoid shell ambiguity. `space.window` returns **queued**, not completed; inspect `ls` and `status.popout_windows` afterward. Status includes detached IDs, child errors, active state, and viewport size. Ordinary CLI commands still target main unless they explicitly name a space/pane; do not infer a new implicit OS-focus routing rule.

Supported child actions include tab new/select/next/prev/rename, pane split/zoom/focus/go, terminal copy/paste/search/scroll/paste_image, child minimize/maximize, and non-destructive close/dock. Return to main for other actions.

## Verified on Windows

Latest feature executable: `%LOCALAPPDATA%\HerdrAppBuild\popout-target\debug\herdr-app.exe`. It is separate from the daily-use baseline executable in `HerdrApp/target/debug` and the installed executable.

- `cargo test --locked --bins --test colrv1 --test commands --test notices --test skill`: **376 passed, 0 failed, 2 ignored** (372 binary tests, four integration tests; no Windows `colrv1` cases). Includes mod snapshot/callback isolation, eligibility/target checks, and a GPUI multi-window lifecycle test retaining the same terminal entity IDs. The GPUI subprocess test also verifies child targeting and non-destructive close.
- `bun test scripts/popout-menu.test.ts scripts/docs.test.ts`: **6 passed, 0 failed**, 34 assertions. Menu-specific portion: two tests, five assertions.
- `bun scripts/docs.ts generate <feature-executable>`: references generated and CLI examples checked in five pages.
- Native standard-layout smoke: `popout-smoke-1bd24e33`, **22 checks passed**. Native Focus smoke: `popout-smoke-3e49c6cd`, **23 checks passed**, including Focus mod validation and active-layout inspection. Artifacts are local under `%LOCALAPPDATA%\HerdrAppBuild`.
- Both smokes cover full tab retention, two simultaneous pop-outs, repeated raise, isolated select/new-tab/split/rename, foreign-target rejection, native resize, main-size isolation for visible and hidden detached terminals, docking, OS window close, repeated cycles, unsupported-pane rejection/automatic docking, explicit space deletion, main close, and reopen.
- **Process evidence:** smokes compare each terminal's address, `pid`, and `created_ms` obtained from the isolated local core via `hc get --json`. The earlier `ecb65987` smoke checked addresses only and must not be cited as PID evidence. New tests establish disposable terminal process continuity, not a separate end-to-end Pi/Codex session-protocol test.
- No app/mod errors; both disposable cores stopped; installed app/core executable hashes unchanged. Focus files were copied only into the disposable home. Existing normal-profile state and unrelated Focus source changes were not replaced.

### Broader JavaScript suite limitation

The entire Bun suite is **not green on this Windows host**. Unmodified baseline reproduces the existing CRLF-sensitive import-stripping failures (baseline: 18 pass, 63 fail, two errors; feature: 20 pass, 63 fail, two errors). In separate LF-normalized scratch copies, baseline gives **87 pass / 2 fail**, feature **89 pass / 2 fail**. Both retain the same directory-symlink permission and Git/worktree grouping failures. No source normalization, privilege change, or broad unrelated test repair was applied. Logs: `bun-baseline*.log`, `bun-feature-lf.log` outside Git.

## Remaining acceptance and limitations

- Captures exist (`main-before.png`, `alpha-native.png`, `alpha-split.png`, `beta-native.png`); the image reader previously could not display them. Owner feedback supplies basic workflow acceptance, but automated visual review and two-monitor/DPI acceptance remain unverified.
- Actual typing, clipboard/search interaction, right-click hit testing, keyboard shortcuts, pane focus/navigation, notifications/sounds, and seen/unread behavior need further UI acceptance. No simulated input was sent to the real display.
- Remote reconnect/disconnect, monitor removal, external terminal exits, macOS/Linux behavior, and `shot_pixels` remain unverified.
- Child close/dock semantics and CLI routing are tested, but there is no persisted child layout/bounds or custom child mod runtime.
- The owner used the feature successfully; exact launch/profile/current process state was not inspected. Only one front end may use the normal profile at a time; do not interrupt a working desktop merely to demonstrate the feature. Daily use shares normal settings/core/workspaces, not a separate permanent daily profile.
- The implementation is pushed to the owner's feature branch. This does not imply a default-branch merge, packaged release, or updater deployment.

## Continue safely

1. Inspect this repository's status and read fork instructions. Inspect HerdrConfig only when working on optional Focus integration, preserving unrelated edits. Do not reclone or rebuild over a working baseline.
2. Use separate target output and fresh `HERDR_APP_HOME`, `XDG_STATE_HOME`, `XDG_CONFIG_HOME`, `HC_SESSION`; remove inherited socket/terminal/scope selectors. A separate app home alone does not isolate the core.
3. Local helpers are `popout-tests.cmd`, `run.py`, and `popout-smoke.py` (`--focus` adds disposable Focus coverage). Use a proper CPython installation: PATH currently selects MySQL's embedded Python, which fails ctypes. Python 3.13 worked. These helpers are not shipped here.
4. Address remaining UI/remote/platform acceptance without faking input on the real display or touching production terminals. Preserve one model/document owner and terminal process identity.
5. Keep agent-facing docs and test evidence current. Do not claim install, release, visual acceptance, or upstream approval based on a passing smoke.

Continue from `8c2c08e` with the single authoritative owner and terminal identities intact. Prioritize remaining interaction/platform coverage and the separately documented release/updater work. Preserve existing daily sessions and distinguish owner feedback, automated evidence, and unverified behavior.
