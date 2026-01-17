# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

コンテナ/リモートクライアントからmacOSを遠隔操作するRust製gRPCアプリケーション。サーバーはmacOS上で動作し、`terminal-notifier`（通知）と`op read`（1Passwordシークレット取得）をgRPC経由で公開する。クライアントはSSHリバーストンネル経由で接続。

```
┌─────────────────┐       SSH -R 50052       ┌─────────────────┐
│   Container     │ ◄──────────────────────► │     macOS       │
│  macos-remote   │     localhost:50052      │ macos-remote-   │
│  (client CLI)   │ ──────────────────────►  │ server          │
└─────────────────┘                          └─────────────────┘
```

## ビルドコマンド

```bash
cargo build --release              # 全クレートをビルド
cargo build -p macos-remote-server # サーバーのみ
cargo build -p macos-remote-client # クライアントのみ
cargo build -p macos-remote-protocol # protoバインディング再生成
```

## テストコマンド

```bash
cargo test --all                   # 全テスト実行
cargo test --all -- --nocapture    # 出力付き
cargo test -p <crate-name>         # 特定クレートのテスト
```

## リント・フォーマット

```bash
cargo fmt --all                              # コードフォーマット
cargo fmt --all -- --check                   # フォーマットチェック
cargo clippy --all-targets -- -D warnings    # 警告をエラーとしてリント
```

## 実行方法

```bash
# サーバー (macOS)
cargo run -p macos-remote-server -- --port 50052

# クライアント (Container)
cargo run -p macos-remote-client -- notify "タイトル" "メッセージ"
cargo run -p macos-remote-client -- op-read "op://vault/item/field"
cargo run -p macos-remote-client -- --addr http://127.0.0.1:9000 notify "Test" "Hello"
```

## アーキテクチャ

**ワークスペースクレート:**
- `macos-remote-protocol` (`crates/macos-remote-protocol/`) - Proto定義、tonic/prostバインディング
- `macos-remote-server` (`macos-remote-server/`) - macOS用gRPCサーバー、`terminal-notifier`と`op` CLIをラップ
- `macos-remote-client` (`macos-remote-client/`) - コンテナ用CLIクライアント

**Proto定義:** `crates/macos-remote-protocol/proto/macos_remote.proto`
- `MacOsRemoteService`に`Notify`と`OpRead` RPCを定義
- protoを編集後、`cargo build -p macos-remote-protocol`でバインディング再生成

**サーバーモジュール:**
- `main.rs` - エントリポイント、CLIパース（clap）
- `service.rs` - gRPCサービス実装
- `notify.rs` - `terminal-notifier`ラッパー
- `op_client.rs` - 1Password CLIラッパー（10秒タイムアウト）

## コードスタイル

- 4スペースインデント、`snake_case`関数/モジュール、`PascalCase`型
- モジュールは約300行以内、`mod tests`は同一ファイルに埋め込み
- `op`実行は`Command::new("op")`で明示的な引数を渡す、`sh -c`は使わない
- 標準出力の末尾改行は`trim_end_matches('\n')`で除去
- シークレットの内容は絶対にログに出力しない
- テスト名はビジネスルールを記述（例: `denies_unknown_id`）、`should_*`形式は使わない
- コミットメッセージ: 命令形現在時制（例: `feat: add nonce validation`）

## 外部依存（実行時）

**macOSサーバーに必要:**
- `terminal-notifier`: `brew install terminal-notifier`
- `op` (1Password CLI): `brew install 1password-cli`

## SSHトンネル設定

```bash
# macOSからコンテナへのリバーストンネル
ssh -R 50052:127.0.0.1:50052 user@remote-server

# ~/.ssh/config に設定する場合
Host devserver
    RemoteForward 50052 127.0.0.1:50052
```
