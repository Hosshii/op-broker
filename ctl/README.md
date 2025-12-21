# op-brokerctl

`op-brokerctl` は dev server / コンテナから Unix ソケット経由で `op-brokerd` にアクセスする CLI です。`read` サブコマンドのみを提供し、allowlist 上の ID から secret を取得します。

## サブコマンド

```
op-brokerctl --socket /run/op-broker.sock read <id> [--nonce <text>] [--json] [--quiet]
```

- `--socket` (必須): `op-brokerd` が listen している Unix ソケットパス。
- `--nonce`: 任意の nonce 文字列。未指定の場合は空文字を送信します。
- `--json`: `{ "ok": true/false, ... }` 形式で出力します。エラー時も JSON を返します。
- `--quiet`: 成功時に値を標準出力へ書き出さず、終了コードのみで成功/失敗を判定したい場合に使用します（JSON モードでは無視）。

エラーは stderr へ `error: ...` を出しつつ exit code 1 を返します。JSON モードでは `{ "ok": false, "code": ..., "message": ... }` を stdout に出力します。

## サンプル

```bash
# Unix ソケットを直接指定
cargo run -p ctl -- --socket ~/.op-broker/op-broker.sock read github_token

# JSON 形式で取得
cargo run -p ctl -- --socket ~/.op-broker/op-broker.sock read github_token --json

# dev server 側 (例: /tmp/op-broker.sock) で利用
cargo run -p ctl -- --socket /tmp/op-broker.sock read github_token
```

## ビルド / テスト

```bash
cargo fmt
cargo build -p ctl --offline
cargo test -p ctl --offline
```

## 補足

- CLI の仕様は root README と `docs/INSTRUCTIONS.md` にも記載されています。
- gRPC proto (`crates/protocol/proto/opbroker.proto`) を変更した場合、`cargo build` で再生成されるコードに合わせて CLI を更新してください。
