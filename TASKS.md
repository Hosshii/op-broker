# TASKS

## 完了済み ✅
- `proto` と `tonic_prost_build` を `crates/protocol` に集約し、gRPC 用のコード生成を構築。
- broker / ctl / protocol の各 crate を Rust 2024 + Tokio + tonic ベースに初期実装し、Unix ドメインソケット越しの gRPC 骨組みを整備。
- `cargo build --offline` と `cargo test --offline` が通る状態で共有ライブラリや依存を整理。
- broker サービスで `op read` を実行し、allowlist 済み secret を安全に返す処理を実装。
- gRPC レベルのエラー設計（invalid_argument / not_found / internal など）とログ方針を暫定整備。
- ctl CLI に `--nonce` や `--json` などのフラグを追加し、エラー表示や出力形式を整備。
- README / AGENTS / config サンプルを最新仕様（gRPC+UDS 運用、セキュリティ要求）に合わせて更新。

## 進行中 / 今後着手予定 ⏳
- （次のタスクをここに追記）
