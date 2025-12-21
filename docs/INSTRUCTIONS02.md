
# 指示書：FUSE + op-broker-client によるオンデマンド Secret FS

## 0. プロジェクト概要

### 目的

* Linux（dev server または host）上で **FUSE ファイルシステム**を実装する
* アプリケーションは **ファイルを read() するだけ**で secret を取得できる
* ファイルの read() をトリガーにして：

  * `op-broker-client` が Unix Domain Socket 経由で
  * Mac 上の `op-broker` に secret をリクエスト
  * 返ってきた secret を **read() の戻り値として返す**
* secret は **永続保存しない**
* Mac 側で Touch ID / 1Password アンロックが発生する

---

## 1. 全体アーキテクチャ

```
[ App (container / host) ]
   |
   | read("/run/op-secrets/<id>")
   |
[ FUSE FS (this project) ]
   |
   | Unix Domain Socket (client)
   |
[ op-broker (Mac) ]
   |
   | op read (Touch ID)
```

---

## 2. 非ゴール（やらないこと）

* 1Password Connect を使う
* コンテナ内で `op` を直接実行する
* secret をファイルや env に永続保存する
* 任意の `op://...` パスを指定できる API
* write / create / unlink を許可する

---

## 3. マウント仕様

### マウントポイント

```
/run/op-secrets
```

* tmpfs 前提
* read-only FS として扱う

### ファイル構成

```
/run/op-secrets/
├── github_token
├── aws_access_key
└── aws_secret_key
```

* **ファイル名 = secret ID**
* ID は allowlist で事前定義されているもののみ

---

## 4. FUSE FS の仕様（厳守）

### 4.1 open()

* 読み取り専用 (`O_RDONLY`) のみ許可
* `O_WRONLY`, `O_RDWR` は `EACCES`
* open 時点では secret を取得しない

---

### 4.2 read()（最重要）

* 初回 read 時にのみ：

  1. secret ID を特定
  2. op-broker-client 経由で secret を取得
  3. メモリ上に保持
* offset を正しく処理する
* EOF 到達後は 0 バイトを返す
* **1 open = 1 secret**
* close 時に secret は即破棄する

---

### 4.3 getattr()

* 仮の stat を返す
* mode: `0400`
* size:

  * 0 または secret 未確定サイズとして扱う
* uid/gid: 実行ユーザー

---

### 4.4 readdir()

* allowlist に含まれる ID のみを返す
* `.` と `..` は含める

---

### 4.5 禁止操作

* write / create / unlink / rename / chmod → `EROFS` or `EACCES`

---

## 5. Secret ID の制約

* 正規表現：`^[a-zA-Z0-9_-]{1,64}$`
* `/`, `..`, NULL byte を含むものは即拒否
* ID → op パスの対応は **Mac 側 broker が決定**

---

## 6. op-broker-client の仕様

### 通信

* Unix Domain Socket（パスは起動時引数）
* 1 行 JSON リクエスト / レスポンス

#### リクエスト

```json
{"op":"read","id":"github_token"}
```

#### レスポンス（成功）

```json
{"ok":true,"value":"SECRET"}
```

#### レスポンス（失敗）

```json
{"ok":false,"error":"DENIED"}
```

---

### エラー変換（FUSE 側）

| broker error | FUSE error |
| ------------ | ---------- |
| DENIED       | EACCES     |
| TIMEOUT      | EIO        |
| UNAVAILABLE  | EHOSTDOWN  |

---

## 7. 実装要件（Rust）

### 使用クレート

* `fuser`（FUSE 実装）
* `serde`, `serde_json`
* `libc`
* `zeroize`（secret 破棄用）
* 標準ライブラリの `UnixStream`

※ async 不要（同期でよい）

---

### 内部構造（例）

```rust
struct OpSecretsFS {
    socket_path: PathBuf,
    allowed_ids: HashSet<String>,
    open_files: Mutex<HashMap<u64, SecretBuffer>>,
}
```

```rust
struct SecretBuffer {
    data: Vec<u8>,
}
```

* `fh`（file handle）単位で secret を管理
* close 時に `zeroize()` して drop

---

### read() の実装要件

* offset を `usize` に変換する前に範囲チェック
* size 分だけ slice して返す
* secret 全体を一度に返さない前提

---

## 8. セキュリティ要件（必須）

* secret 値を **ログに一切出さない**
* エラーメッセージに ID 以外の情報を含めない
* request サイズ上限（例：4KB）
* 同時 open 数の上限（例：16）
* broker RPC に timeout（例：30 秒）
* core dump 無効化（可能なら）

---

## 9. Docker / 運用前提

### 推奨構成

* FUSE は **dev server（ホスト）で mount**
* `/run/op-secrets` を container に bind mount（read-only）

### container 側

* 単なる read-only FS として扱う
* FUSE や broker-client を意識しない

---

## 10. 開発フェーズ分解（Codex 向け）

### Phase 1: FUSE PoC

* 固定文字列を返す FS
* read / offset / EOF の正しさを確認

### Phase 2: broker-client 統合

* Unix socket 経由で JSON RPC
* エラーハンドリング

### Phase 3: hardening

* zeroize
* 制限（open 数 / timeout）
* readdir 実装

---

## 11. 受け入れ条件（テスト）

* `cat /run/op-secrets/github_token` で secret が出る
* 2 回目の read で同じ値が返る
* close 後に再 open すると再取得される
* `ls /run/op-secrets` に allowlist のみ表示される
* write 系操作がすべて失敗する
* broker がロックされている場合、read() がブロックする

---

## 12. 命名

* プロジェクト名：`op-broker`
* FUSE バイナリ：`op-secrets-fs`
* client 内部名：`op-broker-client`

---

## 13. Codex への注意事項（重要）

* **絶対に LD_PRELOAD や ptrace を使わない**
* **shell 経由で `op` を呼ばない**
* **secret をファイルに書かない**
* **設計を勝手に簡略化しない**
