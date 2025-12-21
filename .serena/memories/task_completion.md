# Task Completion Checklist
- **Format**: Run `cargo fmt --all` to enforce workspace-wide Rust formatting with zero diffs.
- **Lint**: Execute `cargo clippy --all-targets -- -D warnings` and resolve all warnings.
- **Build/Test**: Run `cargo build` (or targeted `cargo build -p broker` / `-p ctl` if relevant) followed by `cargo test --all -- --nocapture` to cover unit and integration suites (including socket/allowlist tests under `broker/tests/`).
- **Runtime Checks**: When applicable, validate manual flows via `just run-broker` (launch daemon) and `just run-ctl github_token` to ensure CLI interaction/regressions are covered.
- **Docs/Proto Updates**: If proto or docs change, regenerate via `tonic_prost_build` and refresh README/TASKS references before concluding.
- **Security Review**: Confirm file permissions (0700 dir / 0600 socket+config), ensure no secrets were logged or added to repo, and note any remaining security considerations in the final summary.