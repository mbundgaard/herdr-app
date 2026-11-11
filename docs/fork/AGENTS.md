# Agent instructions for the owner fork

These instructions apply to work throughout this checkout when the owner directs
an agent to read `docs/fork/`. This nested file is not assumed to be automatically
loaded for source files outside this folder.

## Start here

Read every Markdown file in `docs/fork/` before working:

- [README.md](README.md): fork identity, maintenance and release policy.
- [windows-build.md](windows-build.md): build prerequisites and runtime isolation.
- [space-popouts.md](space-popouts.md): implemented behavior, architecture, test
  evidence and outstanding acceptance work.

Then read the root [AGENTS.md](../../AGENTS.md) and optional `AGENTS.local.md`
for upstream engineering guidance. Inspect the current branch, working tree and
relevant source before changing anything. Recorded checkpoints are historical,
not verification of the current checkout or running processes.

## Authority and upstream relationship

This is the owner's `mbundgaard/herdr-app` fork. Owner approval governs fork-only
architecture, development, contributions and releases. Root instructions to ask
Can, require his approval, or reserve releases to him apply to upstream work,
not independent maintenance of this fork. These fork-specific rules take
precedence where the upstream documents conflict.

Retain upstream's small-core architectural questions: does the app act on it,
can only the app see it, and can mods not implement it correctly? If all three
are no, put it in mods. Discuss core growth and compatibility-breaking changes
with the owner before implementing them. Independent desktop development does
not authorize modifying herdr-core.

Do not submit upstream issues or pull requests, push upstream, publish releases,
or change repository visibility unless asked. Integrating selected upstream
releases into our fork is distinct from submitting changes upstream. Follow the
release policy in this folder; do not invent a fork version suffix or overwrite
an existing release.

## Documentation boundary

Keep fork-specific READMEs, agent instructions, policies, plans, build guidance
and implementation/test notes in `docs/fork/`. This folder is the complete fork
handoff: the owner will tell an agent to read it after cloning.

Keep root `README.md`, root `AGENTS.md`, and other non-functional upstream files
aligned with the selected upstream baseline. Do not add fork notices, pointers,
or local instruction files at the root. The root install instructions install
upstream builds, not this fork.

Source changes, tests and necessary shipped/generated documentation remain in
their proper locations. In particular, keep `src/SKILL.md`, `defaults/AGENTS.md`,
`defaults/herdr.d.ts` and `docs/commands.md` accurate for implemented commands and
APIs; these are product contracts, not places for fork maintenance policy.
Preserve upstream documentation checks and generation rather than duplicating
or relocating these contracts. Otherwise avoid changes outside `docs/fork/`
unless required by functionality. Follow upstream engineering and commit
conventions where they do not conflict with the owner's policy.

## Safety and validation

Leave herdr-core source, installed binaries and existing core sessions alone.
Never stop, signal or restart someone else's app or core. Keep the working
desktop running; use separate build output and disposable app/core state for
experiments. A separate app home alone does not isolate the core: follow the
full environment procedure in [windows-build.md](windows-build.md). Never fake
input on the real display or use an install command as a build test.

Daily use may share the normal profile/core with only one desktop at a time.
Keep host-specific state, credentials, helpers, logs and test artifacts outside
Git. HerdrConfig separately owns Focus and host/Pi configuration; it is not a
required sibling checkout for desktop development.

Preserve released-data compatibility and upstream platform/testing safeguards.
Keep agent-facing product documentation current with behavior changes. Record
what was actually tested and distinguish historical evidence, owner feedback,
and unverified behavior. A passing build or smoke test is not a release,
installation, visual acceptance or upstream approval.
