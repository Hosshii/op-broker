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

## Usage

### Server (macOS)

```bash
macos-remote-server
macos-remote-server --port 9000
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

# サーバーアドレスを指定
macos-remote --addr http://127.0.0.1:9000 notify "Test" "Hello"
```

**利用可能なサウンド:** `Ping`, `Pop`, `Glass`, `default`, またはサウンドファイルパス

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
