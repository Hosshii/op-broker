# 指示書: op-broker（Mac 1Password Broker + SSH reverse tunnel）

## 0. ゴール

* **Mac 上で動く broker** が `op`（1Password CLI）を呼び出して secret を取得する
* **dev server 上のコンテナ**は broker に対して **ローカル Unix socket 経由で RPC** し、secret を受け取る
* **Touch ID は Mac の 1Password アプリ側**で行われる（`op` がロックされていれば解除要求が出る）
* **サーバー/コンテナに 1Password の資格情報を置かない**
* **許可した secret 以外は取れない**（allowlist）

---

## 1. 非ゴール（やらない）

* 1Password Connect を立てる
* コンテナ内で `op` を直接動かす
* secrets をファイルや compose に永続保存する（`.env` 生成など）
* 任意の `op read` パスを自由に指定できる API（危険なので禁止）

---

## 2. 脅威モデル（最低限）

### 守るもの

* 1Password の secret 値（トークン、鍵、APIキーなど）
* どの secret を取得できるか（取得対象の情報自体も機密）

### 想定する攻撃者

* dev server 上の別ユーザー
* コンテナ内のプロセス（同一コンテナ内に悪性コードが混入）
* 誤操作（ログや履歴に secret を出してしまう）

### 主要リスクと対策

1. **任意パス読み出し** → allowlist + 固定ID方式
2. **コマンドインジェクション** → `op` 実行は引数配列、シェル不使用
3. **ソケット奪取** → Unix socket 600 + 所有者限定
4. **再利用/リプレイ** → 1回限りトークン（nonce）/ TTL（オプション）
5. **ログ漏洩** → 値をログに出さない（長さ/ハッシュすら慎重に）

---

## 3. アーキテクチャ

### 3.1 ローカル（Mac）

* broker: `op-brokerd`（daemon）
* listen: Unix domain socket `~/.op-broker/op-broker.sock`
* RPC: tonic gRPC（proto は `crates/protocol/proto/opbroker.proto`、`ReadSecret` サービスのみ）
* 実行: `op` バイナリを子プロセス起動して `op read ...` する（タイムアウト/エラーを gRPC `Status` に変換）

### 3.2 トンネル（Mac → dev server）

* SSH reverse tunnel で dev server 上に socket を公開

推奨コマンド（例）：

* dev server 側ソケット: `/tmp/op-broker.sock`

```
ssh -N \
  -R /tmp/op-broker.sock:/Users/<you>/.op-broker/op-broker.sock \
  <devserver>
```

> 注意：`-R` の “remote unix socket” が使える条件（SSH のバージョン/設定）に依存する。
> 代替として TCP localhost ポート転送も用意（後述）。

### 3.3 dev server → container

* コンテナに `/tmp/op-broker.sock` を bind mount で渡す

  * `-v /tmp/op-broker.sock:/run/op-broker.sock`

---

## 4. API 仕様（固定ID方式）

### 4.1 リクエスト（`ReadSecretRequest`）

```
message ReadSecretRequest {
  string id = 1;
  string nonce = 2; // 任意。未使用時は空文字。
}
```

* `id`: allowlist に定義された ID 文字列（例 `github_token`）
* `nonce`: 1回限り token（Phase 1 では空文字でも可）

### 4.2 レスポンス（`ReadSecretResponse`）

```
message ReadSecretResponse {
  string value = 1; // secret 本体
}
```

gRPC `Status` を `invalid_argument`（ID フォーマットエラー）、`not_found`（allowlist 非許可）、`internal`（`op` 実行失敗）で返す。クライアントや CLI はこれをユーザー向けに整形する。

### 4.3 allowlist の定義

`configs/config.example.json` をコピーして `~/.op-broker/config.json` に配置する。

```json
{
  "socket_path": "/Users/you/.op-broker/op-broker.sock",
  "items": {
    "github_token": { "op_path": "op://DevVault/GitHub/token" },
    "aws_access_key": { "op_path": "op://DevVault/AWS/access_key_id" }
  }
}
```

**コンテナからは `op://...` を指定できない**。必ず `id` のみ。

---

## 5. 実装要件（必須）

### 5.1 broker（Mac）

* 言語: **Rust 固定**（`tokio` + `serde_json` + `anyhow` + `clap` を前提に記述）
* Unix socket サーバ
* ファイル権限:

  * ディレクトリ `~/.op-broker` を `0700`
  * ソケットを `0600` 相当で作成（umask/作成後 chmod）
* request サイズ制限（例 4KB）
* JSON パースは厳密に（不明フィールド拒否）
* allowlist にない `id` は `DENIED`
* `op` 起動は **shell を介さない**

  * Rust: `std::process::Command` で `["read", path]`
* タイムアウト（例 5〜10 秒）
* `op` の STDERR はログに出してよいが **secret 値は絶対ログに出さない**
* ログはデフォルト INFO、secret 値を含む可能性のある出力は禁止
* 詳細なコーディング規約やテストフローはリポジトリ直下の `AGENTS.md` に最新版をまとめるので、実装前に必ず参照
* 同時処理:

  * まずは直列でもOK（簡単）
  * 可能なら 1 リクエスト 1 `tokio::task` で処理する並列版を追加

### 5.2 クライアント（コンテナ側）

* `op-brokerctl`（軽量 CLI）
* 基本構文: `op-brokerctl --socket /run/op-broker.sock read <id>`
* `--nonce <text>` で nonce を明示指定（未指定の場合は空文字）
* `--json` で `{ "ok": true, "value": "..." }` 形式を出力。エラー時は `{ "ok": false, ... }`
* `--quiet` で標準出力を抑制（JSON モードでは無視）し、環境変数などへ転送する用途を想定
* gRPC エラーは Exit code 1 & stderr で通知。JSON モードでは `ok:false` を返す

---

## 6. 追加の強化（オプションだが推奨）

### 6.1 1回限り nonce

* broker 起動時にランダム秘密鍵（session key）を生成
* `op-brokerctl init` で nonce を受け取り、以後リクエストに付与
* nonce は TTL 30 秒、使い捨て
* nonce は dev server/コンテナには “保持される”ので、必要性は好み（安全増）

### 6.2 TCP フォールバック

SSH が Unix socket の reverse を許さない環境向けに

* Mac: `127.0.0.1:NNNN` にバインド（localhost only）
* SSH: `-R 127.0.0.1:NNNN:127.0.0.1:MMMM`
* dev server: localhost の TCP をコンテナへ渡す（host network or socat bridge）

**ただし** TCP は誤公開リスクが増えるので、必ず localhost 限定。

### 6.3 “値を返さない”モード

より安全にするなら、secret を返す代わりに

* broker が **指定コマンドを実行**して STDIN で渡す（secret を外に出さない）
  ただし実装が重くなるので次フェーズ。

---

## 7. 開発タスク分解（Codex 向け）

### Phase 1: Mac broker MVP

1. `config.json` 読み込み（items: id -> op path）
2. Unix socket サーバ listen
3. tonic gRPC (`ReadSecretRequest`) を受ける
4. allowlist チェック
5. `op read <path>` 実行して stdout を取得
6. response JSON を返す
7. 権限設定（0700/0600、umask）

### Phase 2: container CLI

1. `--socket` 指定
2. tonic gRPC クライアントで Unix socket 接続
3. レスポンスの `value` を stdout/JSON で出力し、`--json` / `--quiet` 等の UX を追加

### Phase 3: 運用補助

1. `Makefile` / `justfile`
2. `launchd`（Mac 常駐）用 plist（任意）
3. `ssh` トンネル起動スクリプト（例 `bin/op-broker-tunnel`）
4. docker compose サンプル（socket bind mount）

### Phase 4: セキュリティ強化

1. サイズ制限、タイムアウト、エラーハンドリング
2. nonce/TTL（任意）
3. 監査ログ（id と時刻だけ、値は絶対出さない）

---

## 8. 受け入れ条件（テスト観点）

* [ ] Mac で `op-brokerd` 起動、`~/.op-broker/op-broker.sock` ができる
* [ ] allowlist にある `id` は secret が返る
* [ ] allowlist にない `id` は `DENIED`
* [ ] `op` がロックされているとき、Mac 側で通常の 1Password アンロックフローに入れる（Touch ID/パスワード）
* [ ] dev server へ SSH reverse tunnel して、server 上の `/tmp/op-broker.sock` から取得できる
* [ ] コンテナに socket を mount し、コンテナ内 `op-brokerctl` で取得できる
* [ ] broker のログに secret 値が出ない
* [ ] request が巨大/壊れていても broker が落ちない

---

## 9. リポジトリ構成（提案）

```
op-broker/
  broker/           # Mac daemon
  ctl/              # client CLI
  configs/
    config.example.json
  scripts/
    tunnel.sh
  docs/
    threat-model.md
    usage.md
```

---

## 10. ドキュメント（usage.md に書くこと）

* 前提：Mac に 1Password アプリと `op` がインストール済み
* broker 起動方法
* tunnel 起動方法
* dev server 上の socket の場所
* docker run / compose の mount 方法
* コンテナ内での利用例（env 注入例）
* トラブルシュート（SSH が unix socket reverse を許さない場合の TCP fallback）

---

## 11. 実装の注意（Codex に強く指示）

* **絶対に `sh -c` で `op` を呼ばない**
* request の `id` は **正規表現で制限**（例 `[a-zA-Z0-9_\-]{1,64}`）
* `op` の出力末尾改行は扱いを決める（通常は trim する）
* secret 値をメモリに長時間保持しない（返したら破棄）
* エラー文に path を出さない（vault 構造が漏れる）

---

必要なら、この指示書をあなたの好みに寄せて **Rust 実装前提で「crate選定」「モジュール構造」「launchd plist」「compose例」まで含めた完全版」**に整えます。
（たとえば Rust だと `tokio + serde_json + anyhow`、ソケットは `tokio::net::UnixListener` が素直です）
