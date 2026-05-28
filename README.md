# Readsmith

Rust で作る軽量なセルフホスト型 RSS リーダーです。

`Readsmith` は、既存の軽量セルフホスト型 RSS リーダーの設計を参考にしながら、個人利用・低リソース運用・Rust Web アプリケーション学習を目的として作る RSS リーダーです。

特定の既存サービスや大型 RSS リーダーの完全な代替を最初から目指すのではなく、まずは Rust、SQLite、バックグラウンドワーカー、サーバーサイド HTML、htmx を使って、実用可能な最小構成の RSS リーダーを作ることを目的とします。

---

## 目的

`Readsmith` は、以下を目指します。

* 小規模 VPS、自宅サーバー、Raspberry Pi、NAS、古い mini PC でも動かしやすいこと
* Docker Compose で簡単に起動できること
* SQLite だけで運用できること
* Rust らしい明確で保守しやすい構成にすること
* 外部 feed を取得する際のセキュリティを最初から考慮すること
* 重いフロントエンドフレームワークを使わずに始められること
* Rust Web 開発の学習題材として使えること

---

## 参考実装から取り入れる点

このプロジェクトは、軽量なセルフホスト型 RSS リーダーで一般的に採用されている構成を参考にしています。

参考にする機能領域は以下です。

* RSS / Atom feed の購読
* feed の自動検出
* グループ管理
* 既読 / 未読管理
* ブックマーク
* 検索
* キーボードショートカット
* レスポンシブ Web UI
* PWA 対応
* Fever API 互換
* OPML import / export
* SQLite による永続化
* Docker / 単一バイナリ配布

`Readsmith` では、特に以下の設計を参考にします。

* feed pull worker の構成
* SQLite による永続化
* feed fetch state の管理
* 既読 / 未読モデル
* ブックマークモデル
* OPML import / export
* 軽量なセルフホスト設計

ただし、最初から特定実装との完全互換を目指すわけではありません。

---

## MVP でやらないこと

最初の MVP では、以下は対象外です。

* 特定実装との完全互換
* Fever API
* OIDC
* PWA
* i18n
* React SPA
* TanStack Query
* Zustand
* shadcn/ui
* 複数ユーザー対応
* PostgreSQL 対応
* Kubernetes 対応
* オフライン対応
* 高度なモバイル UI

これらは将来的な拡張候補です。
まずは、小さく理解しやすい RSS リーダーを作ることを優先します。

---

## 技術スタック

### バックエンド

* Rust
* axum
* tokio
* tower-http
* tracing
* serde
* thiserror
* anyhow
* sqlx
* SQLite
* reqwest
* feed-rs
* ammonia
* time
* url

### フロントエンド

* サーバーサイド HTML レンダリング
* Askama テンプレート
* htmx
* 少量の Vanilla JavaScript
* 素の CSS

### デプロイ

* Docker
* Docker Compose
* SQLite volume mount
* 単一 Rust アプリケーションプロセス

---

## アーキテクチャ

このアプリケーションは、単一の Rust プロセスとして動作します。

```txt
Rust process
  ├─ axum HTTP server
  └─ tokio background feed pull worker

SQLite
  ├─ groups
  ├─ feeds
  ├─ feed_fetch_state
  ├─ items
  ├─ items_fts
  └─ bookmarks
```

HTTP server は以下を担当します。

* ログイン
* feed 管理
* group 管理
* 記事閲覧
* 既読 / 未読操作
* ブックマーク操作
* 検索
* OPML import / export

background feed pull worker は以下を担当します。

* feed の定期取得
* ETag / Last-Modified 対応
* feed 解析
* item の upsert
* fetch state の更新
* エラー時の exponential backoff

---

## MVP 機能

最初のバージョンでは、以下を実装します。

* パスワードログイン
* group 一覧
* group 作成 / 編集 / 削除
* feed 一覧
* feed 作成 / 編集 / 削除
* feed 自動検出
* 手動 feed refresh
* background worker による定期 refresh
* 記事一覧
* 記事詳細
* 記事の既読 / 未読切り替え
* bookmark / unbookmark
* unread / all / starred filter
* SQLite FTS5 による検索
* OPML import
* OPML export
* Docker Compose setup
* 基本的な unit test

---

## セキュリティ方針

このアプリケーションは外部 URL を取得するため、セキュリティを重視します。

MVP では、最低限以下の SSRF 対策を入れます。

* `http://` と `https://` のみ許可する
* `file://`、`ftp://`、`gopher://` などは禁止する
* localhost へのアクセスを禁止する
* private IP range へのアクセスを禁止する
* link-local address へのアクセスを禁止する
* redirect 先も検証する
* redirect 回数を制限する
* request timeout を設定する
* response body size limit を設定する
* User-Agent を明示する
* 外部 HTML を表示する前に sanitize する

外部 feed の HTML content は、そのまま表示してはいけません。

記事本文の HTML は、`ammonia` などで sanitize してから表示します。

これにより、以下のような危険を減らします。

* script injection
* event handler injection
* `javascript:` URL
* unsafe iframe
* layout-breaking markup

---

## ディレクトリ構成

```txt
Readsmith/
  Cargo.toml
  Cargo.lock
  Dockerfile
  docker-compose.yml
  README.md
  .env.example

  migrations/
    0001_init.sql
    0002_feed_fetch_state.sql
    0003_items_fts.sql

  src/
    main.rs
    app.rs
    config.rs
    error.rs
    state.rs

    routes/
      mod.rs
      auth.rs
      pages.rs
      groups.rs
      feeds.rs
      items.rs
      bookmarks.rs
      search.rs
      opml.rs
      static_files.rs

    domain/
      mod.rs
      group.rs
      feed.rs
      feed_fetch_state.rs
      item.rs
      bookmark.rs
      user_session.rs

    services/
      mod.rs
      feed_fetcher.rs
      feed_parser.rs
      feed_discovery.rs
      pull_worker.rs
      pull_policy.rs
      sanitizer.rs
      opml.rs
      url_guard.rs
      auth.rs

    db/
      mod.rs
      group_repository.rs
      feed_repository.rs
      item_repository.rs
      bookmark_repository.rs
      search_repository.rs
      migration.rs

    templates/
      layout.html
      login.html
      reader.html
      feeds.html
      groups.html
      search.html
      partials/
        sidebar.html
        article_list.html
        article_card.html
        article_detail.html
        feed_tree.html
        feed_form.html
        group_form.html

  static/
    app.css
    htmx.min.js
    app.js

  tests/
    url_guard_test.rs
    pull_policy_test.rs
    feed_parser_test.rs
```

---

## データベース設計

MVP では SQLite を使います。

主な table は以下です。

* `groups`
* `feeds`
* `feed_fetch_state`
* `items`
* `items_fts`
* `bookmarks`

### 設計メモ

feed の設定情報と fetch state は分離します。

`feeds` には、主に静的な情報を保存します。

* feed title
* feed URL
* site URL
* group
* description
* suspended flag

`feed_fetch_state` には、取得処理に関する実行時情報を保存します。

* ETag
* Last-Modified
* Cache-Control
* Retry-After
* last checked time
* next check time
* last HTTP status
* last error
* consecutive failure count

bookmark は snapshot として保存します。

これにより、元の feed item が削除されても、bookmark 済みの記事を残せるようにします。

---

## ルーティング

MVP で想定する route は以下です。

```txt
GET  /login
POST /login
POST /logout

GET  /
GET  /unread
GET  /all
GET  /starred

GET  /feeds
POST /feeds
GET  /feeds/new
GET  /feeds/:id/edit
POST /feeds/:id
POST /feeds/:id/delete
POST /feeds/:id/refresh

GET  /groups
POST /groups
POST /groups/:id
POST /groups/:id/delete

GET  /items/:id
POST /items/:id/read
POST /items/:id/unread
POST /items/:id/bookmark
POST /items/:id/unbookmark
POST /items/mark-all-read

GET  /search?q=...

GET  /opml/export
POST /opml/import

GET  /healthz
```

---

## 環境変数

`.env.example` の例です。

```env
APP_HOST=0.0.0.0
APP_PORT=8080

DATABASE_URL=sqlite:/data/rss-reader.db

APP_PASSWORD=change-me
APP_ALLOW_EMPTY_PASSWORD=false
APP_SESSION_SECRET=change-this-to-a-long-random-secret

APP_PULL_INTERVAL_SECONDS=300
APP_PULL_TIMEOUT_SECONDS=20
APP_PULL_CONCURRENCY=4
APP_PULL_MAX_BACKOFF_SECONDS=86400

APP_ALLOW_PRIVATE_FEEDS=false
APP_MAX_FEED_BODY_BYTES=5242880

RUST_LOG=info
```

---

## 起動方法

### Docker Compose を使う場合

`.env.example` から `.env` を作成します。

```bash
cp .env.example .env
```

アプリケーションを起動します。

```bash
docker compose up --build
```

ブラウザで開きます。

```txt
http://localhost:8080
```

---

## ローカル開発

Rust を更新します。

```bash
rustup update
```

DB migration を実行します。

```bash
cargo sqlx migrate run
```

server を起動します。

```bash
cargo run
```

test を実行します。

```bash
cargo test
```

---

## Docker Compose 例

```yaml
services:
  Readsmith:
    build: .
    container_name: Readsmith
    ports:
      - "8080:8080"
    env_file:
      - .env
    volumes:
      - ./data:/data
    restart: unless-stopped
```

---

## Feed Pull Worker

Feed pull worker は、HTTP server と同じ Rust process の中で動作します。

基本的な loop は以下です。

```txt
loop:
  1. 現在時刻を取得する
  2. next_check_at <= now の feeds を取得する
  3. suspended feeds を skip する
  4. concurrency limit を守って feeds を取得する
  5. ETag / Last-Modified があれば使う
  6. 更新された feeds を parse する
  7. items を upsert する
  8. feed_fetch_state を更新する
  9. APP_PULL_INTERVAL_SECONDS だけ sleep する
```

pull policy は test しやすい pure function として実装します。

例:

* 成功時の next check time を計算する
* 失敗時の next check time を計算する
* exponential backoff を適用する
* Retry-After を考慮する
* Cache-Control を可能な範囲で考慮する

---

## Feed Parsing

Feed の parse には `feed-rs` を使います。

対応予定の形式は以下です。

* RSS
* Atom
* JSON Feed

item の GUID は、以下の優先順位で決めます。

1. item ID
2. link
3. title と publication date の hash

content は、以下の優先順位で決めます。

1. content
2. summary
3. title

記事 HTML は、表示前に必ず sanitize します。

---

## OPML

MVP では OPML import / export に対応します。

### Import

* OPML file を upload する
* `outline` から feed を抽出する
* group 情報があれば反映する
* 重複 feed URL は skip する
* import 結果を表示する

### Export

* 現在の groups / feeds を OPML として出力する
* `Content-Type: text/xml` を返す

---

## Search

検索には SQLite FTS5 を使います。

検索対象は以下です。

* article title
* article content

MVP では simple search で十分です。

将来的には以下を追加できます。

* feed filter
* group filter
* date filter
* bookmark filter
* unread filter

---

## Testing

最低限、以下の unit test を書きます。

* URL validation
* private IP detection
* redirect URL validation
* pull policy
* exponential backoff
* feed parser
* GUID fallback
* HTML sanitizer
* item upsert logic
* OPML import parser
* OPML export generator

追加できると良い integration test は以下です。

* feed 登録
* manual refresh
* article list rendering
* read / unread toggle
* bookmark toggle
* search
* OPML import / export

---

## Roadmap

### Phase 1: Core MVP

* Login
* Feed CRUD
* Group CRUD
* Manual refresh
* Background refresh
* Article list
* Article detail
* Read / unread
* Bookmark
* Search
* OPML import / export

### Phase 2: Reader UX

* Keyboard shortcuts
* Better mobile layout
* Article drawer
* Better unread navigation
* Mark group as read
* Mark feed as read
* Better search filters

### Phase 3: Integrations

* Fever API
* PWA
* OIDC
* JSON API
* External client support

### Phase 4: Advanced Features

* Full-text ranking improvements
* Content extraction
* Image proxy
* Feed health dashboard
* Per-feed refresh interval
* Advanced pull policy

---

## 開発方針

このプロジェクトでは、以下を優先します。

1. シンプルさ
2. 低リソース運用
3. 明確な Rust アーキテクチャ
4. セルフホストしやすさ
5. 外部 content 取得時のセキュリティ
6. test しやすい business logic
7. MVP first の実装

過剰な抽象化は避けます。

ただし、以下は分離します。

* fetch
* parse
* pull policy
* repository
* sanitization
* URL security
* OPML handling

---

## License

TBD.
