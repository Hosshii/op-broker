# op-secrets-fs 開発メモ

## 実装済みの内容

- `clients/op-secrets-fs` クレートを追加し、FUSE + broker client の土台を整備。
- CLI (`src/main.rs`) で `--config`（entries: `{ "name": "...", "path": "/abs/path", "id": "..." }`）/`--socket`/`--timeout` を検証し、Tokio runtime を初期化した上で `OpSecretsFs` をマウントしつつ bind mount を自動で張る。
- gRPC クライアント (`src/client.rs`) を tonic ベースの async 実装に刷新。Unix ソケット経由の接続、`ReadSecretRequest` の発行、`tonic::Status`→`ClientError` 変換を共通化。
- FUSE 実装 (`src/filesystem.rs`) で read 要求ごとに broker へ問い合わせ、`ClientError` に応じて errno を割り当て、write/rename 系は `EROFS` で拒否。レスポンス後はバッファを `zeroize` で破棄。
- `cargo fmt --all` と `cargo check -p op-secrets-fs` を実行済み。

## 現状の課題 / 次のステップ

1. **Linux での統合テスト**: `/run/op-secrets` を tmpfs 上で作成し、`ls`/`cat` で受け入れ条件 (INSTRUCTIONS02.md) を満たすか検証する。`--config` による bind mount・`--timeout`・エラー時の挙動も確認したい。
2. **接続の堅牢化**: `ClientError::Unavailable` 発生時に再接続する仕組みや、連続失敗時のリトライ方針を整備する。
3. **ドキュメント更新**: README もしくは docs 以下に op-secrets-fs のセットアップ手順・必要権限・サンプルコマンドを追記する。
4. **ハードニング**: open ハンドル上限の監視、RPC タイムアウト値のチューニング、core dump 無効化など INSTRUCTIONS02.md の Phase 3 項目を順次対応。
5. **CI への組み込み**: Linux ランナーで `cargo test -p op-secrets-fs` を回し、退行を検知できるようにする。

必要に応じて runtime と FUSE の境界を再検討し、非同期処理の待ち合わせがボトルネックにならないよう監視してください。
