# Repository Guidelines

## プロジェクト構成とモジュール方針
- `broker/` は macOS 上で動作する daemon（`op-brokerd`）を収める Rust crate。`broker/src/` 配下は機能単位で細分化し、`config.rs`（allowlist 読み込み）、`server.rs`（Unix ソケット + RPC）、`op_client.rs`（`op read` 呼び出し）などに分割します。
- `ctl/` はコンテナ内で使う CLI crate。`read` / `export` / `init` をサブコマンドとして実装し、RPC の構造体は共有 crate（`crates/protocol/` など）に切り出して broker と共通化してください。
- `configs/` には `config.example.json` など長期保存できるサンプル設定のみを置き、個人 vault の値は絶対に commit しないでください。
- 運用補助は `scripts/`（SSH トンネル, launchd, compose）と `docs/`（threat-model, usage）に整理し、README と重複する内容はリンクで誘導します。

## ビルド・テスト・開発コマンド
- `cargo build` / `cargo build -p broker` / `cargo build -p ctl` で全体または対象 crate をビルド。push 前の必須チェックです。
- `cargo fmt --all` で rustfmt を走らせ、フォーマット差分をゼロにします。
- `cargo clippy --all-targets -- -D warnings` を lint の基準とし、警告を残さない方針です。
- `cargo test --all -- --nocapture` が公式テストコマンド。Unix ソケットの権限チェックや allowlist の振る舞いは integration test（`broker/tests/`）で確認します。
- `just run-broker` / `just run-ctl github_token`（`just` が入っている場合）でローカル起動とサンプル secret の取得を再現できます。

## コーディング規約と命名
- Rust 標準の 4 スペースインデント、`snake_case`（関数・モジュール）、`PascalCase`（型）、`SCREAMING_SNAKE_CASE`（定数/環境変数）を徹底します。
- ファイルは 300 行程度を上限の目安とし、拡張時は `mod security` などに分割してテストを同じファイル内に `mod tests` で配置します。
- `op` の実行は常に `Command::new("op")` + 明示的引数。`sh -c` や文字列連結は禁止で、標準出力末尾の改行は `trim_end_matches('\n')` などで除去してください。

## テスト方針
- モジュール単位のテストは各ファイル内に書き、Unix ソケットやファイル権限を伴う振る舞いは `tempfile::TempDir` を使った integration test で検証します。
- テスト名は `should_*` よりもビジネスルールを示す名前（例: `denies_unknown_id`、`trims_op_output`）。`ctl/tests/` では CLI フラグの組み合わせを網羅してください。
- セキュリティや IPC に関する不具合を修正した際は必ず回帰テストを追加し、`cargo test --all` が CI の gating 条件です。

## コミットとプルリクエスト
- コミットサマリは命令形・現在形（例: `feat: add nonce validation`）。本文は 72 文字で改行し、背景やセキュリティ上の配慮を説明します。
- PR では関連 Issue をリンクし、`cargo fmt`, `cargo clippy`, `cargo test`, `just run-broker` など実施した確認手順を checklist で明記。CLI の UX を変えた場合は実行ログやスクリーンショットを添付してください。
- 秘匿情報は diff・ログ・スクリーンショットに残さず、サンプル値は `op://DevVault/...` のようなダミーを使います。broker/ctl 両方へ影響する修正はそれぞれのオーナーにレビューを依頼してください。

## セキュリティと構成のヒント
- `~/.op-broker` は `0700`、ソケットと設定ファイルは `0600` を徹底し、Makefile やスクリプトでも明示的に `chmod` します。
- ログには secret の断片すら書き出さず、必要なら `<redacted>` 表記を使用します。
- サンプル設定やドキュメントでは Touch ID, SSH トンネル, allowlist 更新手順など実運用で迷いやすい点を要約し、詳細は `docs/` へ誘導してください。
