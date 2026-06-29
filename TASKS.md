# TASKS — `24-add-basic-cli-application`

Branch-scoped working status for landing the Rust `keys` CLI. Delete when the
branch merges.

Current state: branch is functionally complete and current with `main`. All CI
passes (`test`, `test-rust-cli`, codecov, CodeRabbit); PR #77 is `MERGEABLE` /
`CLEAN` but still a **Draft**. Remaining work is polish before marking ready.

## Blockers — mechanical

- [x] **Commit the devcontainer lock drift.** Done in `44b2c34` (Update
  devcontainer lockfile) — the `rust` feature lock is now tracked.
- [ ] **Mark PR #77 ready for review** and give it a body that summarises the CLI
  and links issue #24. (Operator owns GitHub state.)

## Should fix before merge

- [x] **Remove stale README TODO** (`- TODO: Update known_hosts files`).
  Replaced with a real feature bullet (safe `known_hosts` updates); kept the
  filter-by-user/tag TODO and fixed its "exlusions" typo.
- [x] **Drop the unmaintained `atty` crate** (RUSTSEC-2021-0145). Replaced the
  three TTY checks with `std::io::stdout().is_terminal()` and removed the
  dependency from `Cargo.toml` / `Cargo.lock`.
- [x] **Inject the release version.** Added a "Set CLI version from tag" step to
  the `rust-cli-build` job in `publish.yml` that `sed`s the tag version
  (`steps.version.outputs.full`) into `cli/Cargo.toml` before `cargo build
  --release`, so the released binary's `--version` reports the real version
  instead of the `0.0.0` placeholder.
- [x] **Make `known-hosts --write` consistent with `ssh --write`.** Now
  additive-by-default with `--force` to fully replace, matching the SSH model.
  Added the `--force` flag, dedup by the `hosts key_type key` identity (ignoring
  flags/comment), local-entry preservation + warning, comment/flag refresh on
  match, and updated help text + `cli/README.md`. Added unit tests (preserve,
  no-duplicate, refresh-comment, force-overwrite) and integration tests
  (additive + `--force`).

## Lower priority — deliberate call

- [x] **CI action-version consistency.** Bumped the Rust jobs to
  `actions/checkout@v6` and `check-version-format-action@v5.0.1` to match the
  existing jobs, and replaced the archived `actions-rs/toolchain@v1` with
  `dtolnay/rust-toolchain@master` pinned to an explicit `toolchain: 1.89.0` in
  both `test.yml` and `publish.yml`. Added a Renovate `customManager` (mirroring
  the existing `deno-version` one) so the pinned Rust version is bumped via PR
  against the `rust` Docker datasource.

## Done

- CLI implemented: `ssh` (fetch + safe additive/`--force` write), `pgp`
  (fetch + `--import` to GnuPG), `known-hosts` (fetch + write), `init`, config
  file support with `--server`/`--config` overrides.
- Test suite (28 tests) + integration tests; coverage wired into CI.
- Nix flake package + dev shell matching CI checks (fmt, clippy, llvm-cov).
- Devcontainer rust feature + extensions.
- Root README points at the CLI; CLI README documents install/usage.
