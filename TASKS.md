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

- [ ] **Remove stale README TODO** (`cli/README.md:12`,
  `- TODO: Update known_hosts files`). Known-hosts read and write are both
  implemented and tested. Keep the filter-by-user/tag TODO — that's still real.
- [ ] **Drop the unmaintained `atty` crate** (RUSTSEC-2021-0145). Replace the
  three TTY checks (`ssh_keys.rs:136`, `pgp_keys.rs:106`, `known_hosts.rs:189`)
  with `std::io::IsTerminal` from std (available on `edition = "2024"`) and
  remove the dependency from `Cargo.toml`.
- [ ] **Inject the release version.** `Cargo.toml` is pinned at `0.0.0` and the
  `rust-cli-build` job in `publish.yml` never stamps the git tag, so the released
  binary reports `keys --version 0.0.0`. Add a step that sets the version from
  the tag before `cargo build --release`.
- [ ] **Make `known-hosts --write` consistent with `ssh --write`.** Currently it
  always replaces the whole file; change it to additive-by-default and require
  `--force` to fully replace, matching the SSH command's safety model.
  - Add a `--force` flag to the `KnownHosts` subcommand in `main.rs` and thread
    it into `write_known_hosts`.
  - Default (no `--force`): read existing entries, add server entries not already
    present, preserve local-only entries. Dedup by the `hosts key_type key`
    identity (ignoring trailing comment / leading flags), mirroring how
    `ssh_keys::extract_key_part` compares on type+key. Refresh flags/comment when
    an existing entry matches a server entry.
  - `--force`: keep today's behaviour (replace the file entirely with server
    entries).
  - Update help text, `cli/README.md`, and add tests for both modes (additive
    preserves local entries; `--force` replaces) alongside the existing ones.

## Lower priority — deliberate call

- [ ] **CI action-version consistency.** New Rust jobs use `actions/checkout@v4`
  and `check-version-format-action@v3`; existing jobs use `@v6` / `@v5.0.1`.
  Also `actions-rs/toolchain@v1` is archived — consider `dtolnay/rust-toolchain`.

## Done

- CLI implemented: `ssh` (fetch + safe additive/`--force` write), `pgp`
  (fetch + `--import` to GnuPG), `known-hosts` (fetch + write), `init`, config
  file support with `--server`/`--config` overrides.
- Test suite (28 tests) + integration tests; coverage wired into CI.
- Nix flake package + dev shell matching CI checks (fmt, clippy, llvm-cov).
- Devcontainer rust feature + extensions.
- Root README points at the CLI; CLI README documents install/usage.
