# localtools

自作のコマンドラインツールを一つの実行ファイルにまとめたものです。

## インストール

```bash
nix build
./result/bin/localtools --help
nix run github:Hosshii/op-broker -- --help
```

Nix flake は `packages.${system}.localtools` を公開しています。GitHub Releases には `localtools-{version}-{target}.tar.gz` と `SHA256SUMS.txt` が含まれます。対応するリリースターゲットは `aarch64-apple-darwin` と `x86_64-unknown-linux-gnu` です。

## SSH

```bash
localtools ssh nixos
localtools ssh -p 2222 user@host
```

OpenSSH に引数を渡します。tmux ペインのフォアグラウンドでは、SSH の引数から対話シェルと判断した場合に前景色を `#d8d8d8`、背景色を `#202020` に変更します。SSH の終了や一時停止で元の設定に戻し、`fg` で再開すると再び着色します。通信断の検出用に、着色時はコマンドラインで値を指定していない keepalive 項目に `ServerAliveInterval=15` と `ServerAliveCountMax=3` を補います。

SSH 設定ファイルでトンネル専用やリモートコマンド実行を指定しているホストでは、接続時に `-N` や `-T` を付けると着色を省けます。実行時は `ssh` と、着色する場合は `tmux` が `PATH` に必要です。

macOS の dotfiles では zsh 関数を通じて、tmux 内でも普段どおり `ssh nixos` と入力できます。

## op-broker

リモートクライアントから macOS 上の通知、1Password シークレット取得、サーバーで指定したコマンドの実行を gRPC 経由で利用できます。

```bash
# macOS 側のサーバー
localtools op-broker serve
localtools op-broker serve --port 9000 --exec-path /path/to/command

# リモートクライアント
localtools op-broker notify "Title" "Message" --sound Ping
localtools op-broker op-read "op://vault/item/field"
localtools op-broker exec arg1 arg2 --flag
localtools op-broker --addr http://127.0.0.1:9000 notify "Test" "Hello"
```

macOS 側では通知用に `terminal-notifier`、シークレット取得用に 1Password CLI の `op` が必要です。サーバーは既定で `127.0.0.1:50052` を待ち受けます。リモート環境から利用する場合は、macOS からリバーストンネルを開きます。

```bash
ssh -R 50052:127.0.0.1:50052 user@remote-server
```

以前の `macos-remote <command>` は `localtools op-broker <command>` に、`macos-remote-server` は `localtools op-broker serve` に置き換えてください。既存の gRPC クライアントとサーバーも引き続き通信できます。

## 開発

| クレート | 役割 |
| --- | --- |
| `crates/localtools` | CLI の入口とコマンドの振り分け |
| `crates/op-broker` | gRPC クライアントと macOS サーバー |
| `crates/ssh-pane` | OpenSSH の起動と tmux ペインの着色・復元 |
| `crates/macos-remote-protocol` | Proto 定義と Rust バインディング |

```bash
nix develop                # Rust 1.92 と protoc
cargo build --release
cargo test --workspace
nix flake check
```

## License

MIT
