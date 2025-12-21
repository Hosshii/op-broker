# protocol crate

共有 proto とユーティリティ型をまとめる crate です。

## 内容

- `proto/opbroker.proto`: tonic / prost 用の gRPC 定義。`ReadSecret` RPC のみを扱います。
- `build.rs`: `tonic_prost_build` を使って proto をコンパイルし、descriptor を `OUT_DIR` に出力します。
- `src/lib.rs`: 共有ロジック (`SecretId` のバリデーション、`include_proto!` で生成した型) を提供します。

## 使い方

Broker と CLI は双方とも `protocol` crate を依存関係に追加し、`protocol::pb::*` を通じて gRPC 型を共有します。

proto を編集したら `cargo build` で自動的に再生成され、`*_descriptor.bin` も更新されます。

## ビルド / テスト

```bash
cargo fmt
cargo build -p protocol --offline
cargo test -p protocol --offline
```

## 補足

- 追加の RPC が必要な場合は `proto/opbroker.proto` を編集し、`docs/INSTRUCTIONS.md` と README を更新してください。
