# op-broker

Rust で実装された 1Password broker です。macOS 上の `op` CLI に安全にアクセスできるようにし、dev server やコンテナから Unix ドメインソケット経由で secret を取得します。詳細な仕様は `docs/INSTRUCTIONS.md`（元の指示書）にまとめています。

## リポジトリ構成

- `broker/` – macOS 側で常駐する daemon（`op-brokerd`）。Unix ソケットで tonic gRPC を提供し、allowlist に従って `op read` を実行します。
- `ctl/` – dev server / コンテナから利用する CLI。`--socket`, `--nonce`, `--json`, `--quiet` などのフラグを備えた `read` サブコマンドを提供します。
- `crates/protocol/` – 共有 proto・型定義。`proto/opbroker.proto` を `tonic_prost_build` で自動生成します。
- `configs/config.example.json` – allowlist やソケットパスのサンプル設定。
- `docs/INSTRUCTIONS.md` – 旧 README の完全版。脅威モデルや実装タスクなどすべての指示をここに保存しています。

## クイックスタート

1. **依存の準備**: macOS に 1Password アプリと `op` CLI をインストールし、`cargo` (Rust 1.92+) を用意します。
2. **設定の配置**:
   ```bash
   mkdir -p ~/.op-broker
   cp configs/config.example.json ~/.op-broker/config.json
   # ~/.op-broker/config.json を自分の op:// パスに書き換え
   ```
3. **broker の起動**:
   ```bash
   cargo run -p broker -- --config ~/.op-broker/config.json
   ```
4. **CLI からの取得**:
   ```bash
   cargo run -p ctl -- --socket ~/.op-broker/op-broker.sock read github_token
   cargo run -p ctl -- --socket ~/.op-broker/op-broker.sock read github_token --json
   ```
5. **SSH トンネルを利用する場合**: macOS から dev server へ `/tmp/op-broker.sock` をリバース転送し、dev server/コンテナ内で `--socket /tmp/op-broker.sock` を指定します。

## ドキュメント

- `docs/INSTRUCTIONS.md` – 元 README。脅威モデルや開発フローなど詳細な情報をすべて掲載。
- `AGENTS.md` – このリポジトリで作業する際のガイドライン（コードスタイル、セキュリティ、レビュー方針など）。

## ライセンス

MIT License
