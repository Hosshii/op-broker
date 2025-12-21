# op-secrets-fs

`op-secrets-fs` は Linux 専用の補助バイナリで、稼働中の [`op-broker`](../../README.md) を
裏側に持つ読み取り専用 FUSE ファイルシステムをマウントします。許可されたシークレット参照
ごとに通常ファイルを公開することで、コンテナや CI から 1Password の認証情報を持ち出さずに
`cat` で参照できます。FUSE 内のファイル名は `op://` パスを URL-safe base64 へ変換して生成します。

## 前提条件

- FUSE を利用できる Linux ホスト（macOS ではビルドされません）
- `op-broker` が提供する Unix ドメインソケット（多くの場合 SSH トンネル越し）
- broker 側の allowlist と一致するシークレット参照の一覧

## 使い方

```bash
cargo run -p op-secrets-fs -- \
  --socket /tmp/op-broker.sock \
  --mountpoint /run/op-secrets \
  --config configs/op-secrets-fs.example.json \
  --timeout 30
```

- `--config` は JSON ファイルを指し、`entries` 配列に `path`（ホスト上で見せたい絶対パス）と `secret_reference` を列挙します。実行時に `/run/op-secrets/<base64>` を自動マウントし、指定パスへ bind mount します。
- `--socket` は SSH トンネルであっても broker の Unix ソケットを指す必要があります
- マウントポイントは事前に作成しておき、読み取り専用でマウントされます
- 回線が遅い場合は `--timeout` を延ばしてください

`configs/op-secrets-fs.example.json`

```json
{
  "entries": [
    {
      "path": "/path/to/hoge/.env",
      "secret_reference": "op://DevVault/GitHub/token"
    },
    {
      "path": "/path/to/fuga/.env",
      "secret_reference": "op://DevVault/Database/password"
    }
  ]
}
```

マウント後は `/run/op-secrets/<base64>` と指定した各 `path` が bind mount で結び付けられます。読み取り時に gRPC
で broker に問い合わせ、レスポンス送信後に内容をゼロクリアします。

## 開発

- `cargo fmt --all`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all -- --nocapture`

Linux でのみビルドできるため、クロスコンパイルや Linux VM/コンテナを用意してください。
リポジトリ全体のポリシーは `docs/INSTRUCTIONS.md` と `AGENTS.md` を参照してください。
