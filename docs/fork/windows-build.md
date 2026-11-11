# Windows build and test handoff

See [fork policy](README.md) and [space pop-outs](space-popouts.md). These are desktop-development instructions, not host alignment. The app checkout is self-contained; no sibling configuration repository is required for building.

## Verified checkpoint

- Owner fork: <https://github.com/mbundgaard/herdr-app>; upstream: <https://github.com/herdrdev/herdr-app>.
- Baseline: `bd2d5cca47af94ad049c931e4125849f84829b0c`, package **0.1.15**, recorded on `build/windows-baseline`.
- Pop-outs: **`8c2c08e`**, committed and pushed to `origin/feature/space-popout`.
- Ghostty submodule: `291fd050b9e9c293c8adb6a140742649b7e42d3a`.
- Unchanged cold baseline build: **5m 30s**; selected Rust suites: **364 passed, 0 failed, 2 ignored**; disposable runtime smoke passed, installed binary hashes unchanged.
- Latest feature build: `cargo build --locked --timings`, exit **0**, **1m 41s**; reports `herdr-app 0.1.15`. Three existing vendor warnings. Feature tests and acceptance limitations are recorded in the linked feature note.
- Owner subsequently reported that pop-outs work very well and confirmed that selecting a detached space in main activates its window. This is basic hands-on acceptance, not complete interaction/platform coverage. Exact current runtime/profile state was not inspected.

These are historical results, not claims about a newly checked-out revision or currently running processes. An earlier low-resource host did not complete its build; that attempt is superseded by the successful baseline above.

## Prerequisites and build

Read every Markdown file in `docs/fork/`, including [our agent instructions](AGENTS.md), then root `AGENTS.md`, optional `AGENTS.local.md`, `CONTRIBUTING.md`, `README.md`, `justfile`, CI, and relevant scripts before building. Root documents are upstream guidance; fork authority and release overrides live here. Root installation commands install upstream builds. Inspect the checkout and installed tools before changing anything. For a new checkout:

```text
git clone --recurse-submodules https://github.com/mbundgaard/herdr-app.git HerdrApp
```

Inspect remotes and add the official repository as `upstream` if absent. Do not clone over an existing checkout. Select the intended feature/release revision explicitly; do not assume the fork's default branch includes it.

Verified Windows toolchain (recheck requirements at the selected revision):

| Tool | Recorded version |
| --- | --- |
| VS 2022 Build Tools | 17.14.45 |
| MSVC | 14.44.35207 |
| Windows SDK | 10.0.26100.0 |
| LLVM/libclang | 23.1.5 |
| Rust/Cargo | 1.95.0, Windows MSVC target |
| Zig | 0.16.0 |
| Bun, for JavaScript/docs checks | 1.4.4 |

1. Locate a Visual Studio installation with `Microsoft.VisualStudio.Component.VC.Tools.x86.x64` using `vswhere`; an installed IDE alone does not prove the tools exist. Standalone Build Tools can coexist with the IDE. Do not close unrelated IDE sessions to install prerequisites.
2. Initialize its `VC\Auxiliary\Build\vcvars64.bat` in the same command environment as Cargo. A child CMD cannot alter its parent PowerShell environment.
3. Add Rust/Zig to that process's PATH and set `LIBCLANG_PATH` to the actual LLVM `bin`. Check `where.exe link` to avoid Git's unrelated linker. Verify tool versions.
4. Select a separate `CARGO_TARGET_DIR` when preserving a working executable. Recorded local builds used `CARGO_PROFILE_DEV_DEBUG=1`, `CARGO_PROFILE_TEST_DEBUG=1`, and `CARGO_INCREMENTAL=0`; changing profiles can invalidate caches.
5. Run `cargo build --locked --timings`. Record logs, final exit status, and executable `--version`; a background job starting is not build success. Preserve caches after interruption.
6. Run the selected Windows checks:

   ```text
   cargo test --locked --bins --test colrv1 --test commands --test notices --test skill
   bun test scripts/popout-menu.test.ts scripts/docs.test.ts
   ```

   Use isolated test state as below. `shot_pixels` was excluded as in the inspected Windows CI. Passing these checks does not mean the broader Bun suite is green.

Do not use `just install` as a build test: it replaces installed software. Dependency optimization makes cold debug builds substantial; do not promise a short duration. When using Bash to invoke CMD, avoid MSYS argument rewriting of `/d` and `/c`. On Windows use system tar if Git's tar treats drive letters as remote hosts. Proper CPython is needed for ctypes-based native test helpers; an embedded Python may fail.

## Runtime isolation and daily use

Disposable tests must use:

- Fresh `HERDR_APP_HOME`, `XDG_STATE_HOME`, `XDG_CONFIG_HOME`, and unique `HC_SESSION`; optionally a unique `HERDR_APP_ID`.
- No inherited `HC_SOCKET`, `HC_TERMINAL_ID`, `HC_TERMINAL_SOCKET`, or `HC_SCOPE`. Clear `HERDR_RELEASES_URL` to avoid unintended update configuration.
- The existing unchanged `hc` executable, with the same isolated environment as the test app. A separate app home alone does not isolate the core.
- Explicit app executable paths; scratch spaces/terminals only. Inspect `ls`, `status`, terminal output, and screenshots. Upstream `scripts/bench.py` provides first-run fixture guidance.
- Cleanup scoped to the owned test app, terminals, and core session. Never kill processes globally by name or simulate input on the real display.

Separate build output protects baseline/installed executables and avoids Windows file locks; it does not require separate daily data. The owner wants one desktop at a time using the normal settings, core, and workspaces. An earlier authorized baseline switch preserved all **49 live terminal addresses**, settings, Focus selection, and the existing core; this was address continuity, not comprehensive agent-session protocol validation. Do not interrupt an active desktop merely to demonstrate a feature, and never stop core terminals to switch front ends.

Local development helpers and smoke artifacts historically live under `%LOCALAPPDATA%\HerdrAppBuild`, including `popout-build.cmd`, `popout-tests.cmd`, `run.py`, and `popout-smoke.py` (optional `--focus`). Feature output was `popout-target\debug\herdr-app.exe`; baseline output was the checkout's `target\debug\herdr-app.exe`. These are local conveniences, not shipped automation or prerequisites on another host. The portable procedure above and upstream scripts are the starting point elsewhere.
