# Herdr desktop — owner fork

This repository, <https://github.com/mbundgaard/herdr-app>, is the authoritative home for the owner's Herdr desktop source, tests, build/release automation, and development notes. Herdr is a native GPUI desktop client for terminals and coding agents, backed by the separate upstream herdr-core project.

## Start here after cloning

Tell the coding agent: **Read every Markdown file in `docs/fork/` before working.** This folder contains the fork-specific instructions and handoff; no root-level pointer is required.

- [Agent instructions and upstream authority overrides](AGENTS.md)
- [Windows build and isolation](windows-build.md)
- [Native space pop-outs: implementation and evidence](space-popouts.md)

[HerdrConfig](https://github.com/mbundgaard/herdr-config) separately owns Focus, host configuration, and Pi/router policy. It is not required to build or maintain the app. Optional Focus integration testing needs its package.

## What differs from upstream

Native terminal-space pop-outs are implemented in the recorded `feature/space-popout` checkpoint. They preserve existing terminals while displaying a space in its own native window. See the feature note for supported actions, limitations and historical Windows test evidence; inspect the current checkout rather than assuming a branch contains that checkpoint.

Owner release/update distribution is not implemented yet. No packaged owner release is claimed. Use the build guidance in this folder for this fork; the unchanged root README's installation commands install **upstream**, not our builds.

For general Herdr usage, see the [upstream README](https://github.com/herdrdev/herdr-app/blob/master/README.md) and [upstream project](https://github.com/herdrdev/herdr-app). The root README and AGENTS files are retained from the selected upstream baseline, not maintained as fork introductions.

## Documentation boundary

All fork-specific narrative documentation and instructions belong in `docs/fork/`. Keep root documentation and other non-functional files clean against the selected upstream baseline, without fork notices or pointers. Necessary source, test and shipped/generated API documentation changes stay with the functionality they describe; see [agent instructions](AGENTS.md) for the precise boundary. Keep this folder complete enough to hand off a fresh clone without a sibling repository or private notes.

## Maintenance policy

- Maintain one continuing desktop fork: selected upstream code plus our features, not multiple product variants. Minimize divergence; reuse upstream packaging, tests, and release conventions.
- `origin` is the owner's fork; `upstream` is <https://github.com/herdrdev/herdr-app>. Integrate selected upstream releases when useful, not every intermediate release. Integration includes cumulative upstream changes; retain and regression-test our features.
- Keep the integrated upstream version number unchanged. No `-comput` or other fork suffix, independent version sequence, or custom build-number system is currently requested.
- Initially publish at most one immutable owner release per upstream version. The existing version-based updater cannot distinguish different builds with the same version. If another distributed update is needed before integrating a new upstream version, resolve that requirement explicitly rather than overwriting release assets or inventing numbering.
- Herdr core is a separate upstream project. Do not fork or modify it for desktop work. Preserve official core-update support; that policy does not authorize updating or restarting an active core during development.
- Owner authorization governs fork work. Do not submit upstream issues/PRs or push upstream unless requested. Merging selected upstream changes into our fork is distinct from submitting our changes upstream.
- Keep host-specific helpers, logs, test profiles, backups, credentials, and private workspace state outside Git. Use disposable app/core state for experiments; normal daily use shares the existing profile/core with one desktop at a time.

## Release direction — not implemented yet

Desktop releases should be published to **this repository's GitHub Releases**. Desktops should detect those releases and install/restart only after the owner's explicit Update action. Core update checks should continue using the official Herdr source. A Git push or local build is not a published update.

Adapt the existing `.github/workflows/release.yml`, `.github/workflows/ci.yml`, and `scripts/package` rather than introduce a parallel build system. Keep packaging checks, checksums, license notices, and test gates; replace upstream-specific publishing/signing dependencies. Start with Windows x64, the platform validated so far. Verify the pipeline's exact current prerequisites before changing it.

Current `src/update.rs` still defaults to `https://releases.herdr.dev`: app metadata is `app/latest.json`, core metadata is `core/latest.json`. Ordinary source builds do not normally self-update; release builds or an explicit `HERDR_RELEASES_URL` override enable checking, and app replacement also depends on a recognized installation. Existing staged-update application on quit and post-swap core-update behavior must be reviewed against the desired explicit desktop-update action and independent core lifecycle. Simply changing the shared URL is not sufficient to separate app and core feeds.

Before production publication, test two distinct versions in a disposable installation: owner-only desktop discovery, explicit installation approval, checksum/download failure safety, profile/terminal preservation, and independent official core-update detection. Fork-specific updater/publishing changes and this end-to-end validation remain pending; there is no owner release pipeline or update-enabled release claimed here.

The owner app repository was confirmed public during this documentation migration. Public releases allow unauthenticated downloads; private releases would require secure client authentication. Repository visibility alone does not configure the updater. Do not change visibility or publish a release implicitly.
