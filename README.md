# macOS Remote Control

コンテナからmacOSを遠隔操作するgRPCアプリケーション。

## Architecture

```
┌─────────────────┐       SSH -R 50052       ┌─────────────────┐
│   Container     │ ◄──────────────────────► │     macOS       │
│  macos-remote   │     localhost:50052      │ macos-remote-   │
│  (client CLI)   │ ──────────────────────►  │ server          │
└─────────────────┘                          └─────────────────┘
```

## Features

- **Notify** - `terminal-notifier`で通知表示（サウンド指定可）
- **OpRead** - `op read`で1Passwordシークレット取得
- **Exec** - サーバー指定のコマンドを引数付きで実行

## Requirements

### macOS (Server)
- Rust 1.92+
- [terminal-notifier](https://github.com/julienXX/terminal-notifier): `brew install terminal-notifier`
- [1Password CLI](https://developer.1password.com/docs/cli/): `brew install 1password-cli`

### Container (Client)
- Rust 1.92+

## Build

```bash
cargo build --release
```

## Release

`v*` タグをpushするとGitHub Actionsがビルドを実行し、GitHub Releasesにバイナリを公開する。

```bash
git tag v1.0.0
git push origin v1.0.0
```

リリースには以下が含まれる:
- `macos-remote-server-{version}-{target}.tar.gz`
- `macos-remote-{version}-{target}.tar.gz`
- `SHA256SUMS.txt`

対応ターゲット: `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`

## Build with Nix

```bash
# client + server
nix build

# client only
nix build .#client

# server only
nix build .#server
```

```bash
# open a development shell with Rust/protoc toolchain
nix develop

# run all nix checks (build, clippy, fmt, test)
nix flake check
```

`nix develop` はビルド必須ツールのみを提供します。`macos-remote-server` 実行時に必要な `terminal-notifier` と `op` は別途インストールが必要です。

## Usage

### Server (macOS)

```bash
macos-remote-server
macos-remote-server --port 9000

# カスタムコマンド実行を有効化
macos-remote-server --exec-path /path/to/command
```

### Client (Container)

```bash
# 通知を送信
macos-remote notify "Title" "Message"

# サウンド付き通知
macos-remote notify "Title" "Message" --sound Ping
macos-remote notify "Title" "Message" -s default

# 1Passwordシークレットを取得
macos-remote op-read "op://vault/item/field"

# サーバー指定のコマンドを実行
macos-remote exec arg1 arg2 --flag

# サーバーアドレスを指定
macos-remote --addr http://127.0.0.1:9000 notify "Test" "Hello"
```

## SSH Tunnel Setup

```bash
# macOS側でSSH接続時にリモートフォワード
ssh -R 50052:127.0.0.1:50052 user@remote-server

# ~/.ssh/config に設定する場合
Host devserver
    HostName remote-server
    User user
    RemoteForward 50052 127.0.0.1:50052
```

## Project Structure

```
├── crates/macos-remote-protocol/   # Proto定義
├── macos-remote-server/            # gRPCサーバー (macOS)
└── macos-remote-client/            # CLIクライアント (Container)
```

## License

MIT
