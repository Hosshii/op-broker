# broker

`op-brokerd` は macOS で常駐する tonic gRPC サーバーです。`~/.op-broker/config.json` に定義した allowlist を読み込み、Unix ドメインソケット上で `ReadSecret` RPC を提供します。各リクエストは allowlist を通過した `op` パスに対して `op read` を実行し、応答を呼び出し元へ返します。

## 主な機能

- Unix ドメインソケット (`socket_path`) 上の tonic gRPC。
- `config.json` による ID → `op://` パスの allowlist。
- `tokio::process::Command` を通じた `op` 実行、タイムアウト・終了コード・stderr のハンドリング。
- gRPC `Status` へのエラーマッピング（`invalid_argument`, `not_found`, `failed_precondition`, `internal` など）。

## 使い方

```bash
cp configs/config.example.json ~/.op-broker/config.json
cargo run -p broker -- --config ~/.op-broker/config.json
```

config 例:

```json
{
  "socket_path": "/Users/you/.op-broker/op-broker.sock",
  "items": {
    "github_token": { "op_path": "op://DevVault/GitHub/token" }
  }
}
```

## テスト / ビルド

```bash
cargo fmt
cargo build -p broker --offline
cargo test -p broker --offline
```

## 備考

- 詳細な仕様や脅威モデルはリポジトリ直下の `docs/INSTRUCTIONS.md` を参照してください。
- `op` CLI が PATH に無い場合は `OpClient` が `failed_precondition` を返します。
