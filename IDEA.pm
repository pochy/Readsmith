あなたはシニア Rust / Web アプリケーションエンジニアです。

これから、Go 製 RSS リーダーである [Fusion](https://github.com/0x2E/fusion) を参考にしながら、Rust で軽量なセルフホスト型 RSS リーダーを設計・実装してください。

これは Fusion の完全移植ではありません。
Fusion の設計思想・機能構成・DB 設計・feed pull worker・UI/UX を参考にしつつ、Rust 学習と軽量セルフホストを目的に、まずは小さな MVP を作ることを重視します。

# 背景

参考にする Fusion は、軽量なセルフホスト型 RSS リーダーです。

Fusion が持っている主な特徴は以下です。

* RSS / Atom feed の購読
* feed auto-discovery
* グループ管理
* 未読管理
* ブックマーク / starred
* 検索
* Google Reader 風キーボードショートカット
* レスポンシブ Web UI
* PWA 対応
* Fever API 互換
* OPML import / export
* SQLite による永続化
* 単一バイナリ / Docker 配布
* Password auth
* Optional OIDC
* feed pull worker による定期更新
* ETag / Last-Modified / cache header を考慮した fetch state 管理

ただし、今回最初に作るものは Fusion の完全互換ではありません。

目的は、Rust で以下を学びながら、実用可能な最小 RSS リーダーを作ることです。

* Rust Web backend
* axum
* SQLite
* sqlx
* background worker
* feed parsing
* search / FTS
* SSRF 対策
* HTML sanitizer
* server-rendered HTML
* htmx
* 軽量セルフホストアプリ設計

# プロジェクトの目的

Rust で軽量な RSS リーダーを作成してください。

重視することは以下です。

1. 小規模な個人サーバーで常時稼働できること
2. Docker で簡単に起動できること
3. SQLite だけで運用できること
4. まずは MVP を作ること
5. Fusion のような RSS リーダーの構造を学べること
6. feed pull worker をきちんと分離して設計すること
7. HTML sanitization と SSRF 対策を最初から意識すること
8. React SPA ではなく、最初は server-rendered HTML + htmx で作ること
9. 読書体験が htmx で苦しくなった場合だけ、将来的に Preact / Solid への置き換えを検討できる設計にすること
10. 不要に大きなアーキテクチャにしないこと

# プロジェクト名

仮のプロジェクト名は `Readsmith` としてください。

# 参考対象

Fusion を参考にしてください。

ただし、以下はコピーではなく、Rust 版として再設計してください。

* backend architecture
* DB schema
* feed pull worker
* feed fetch state
* read/unread model
* bookmarks model
* OPML import/export
* search
* Fever API compatibility
* frontend UX

Fusion の完全互換を最初から目指さないでください。

# 推奨技術構成

## Backend

以下を使ってください。

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
* async-trait

## HTML rendering

以下を第一候補にしてください。

* Askama

理由:

* Rust から type-safe に HTML template を扱える
* server-rendered HTML と相性が良い
* htmx の HTML fragment response を返しやすい
* React SPA より軽く始められる

代替案として Maud も検討可能ですが、画面数が増えそうなので Askama を優先してください。

## Frontend

MVP では以下を使ってください。

* server-rendered HTML
* htmx
* 少量の Vanilla JavaScript
* 必要なら Alpine.js を最小限

MVP では以下を使わないでください。

* React
* Vue
* Svelte
* Solid
* Preact
* TanStack Query
* Zustand
* 大きな UI component library

ただし、将来的に読書体験を強化する場合、Preact または Solid に置き換えやすいように、backend API は HTML response だけに閉じすぎないでください。
HTML page / partial と JSON API を分離しやすい構成にしてください。

## CSS

第一候補:

* 素の CSS
* class-based の軽量設計

第二候補:

* Tailwind CSS CLI

ただし、Node.js 依存や巨大な frontend build 環境は MVP では避けてください。

## Deployment

* Docker
* Docker Compose
* SQLite volume mount
* Rust アプリ単体で static assets を配信
* nginx / Caddy は必須にしない
* 外部公開する場合のみ reverse proxy を使う

# 全体アーキテクチャ

Fusion と同じく、1 プロセス内で以下を動かしてください。

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

# MVP 機能

最初の MVP では以下を実装してください。

1. password login
2. group 一覧
3. group 作成 / 編集 / 削除
4. feed 一覧
5. feed 追加 / 編集 / 削除
6. feed URL validation
7. feed auto-discovery
8. 手動 refresh
9. background worker による定期 refresh
10. article 一覧
11. article 詳細表示
12. article を read / unread にする
13. bookmark / unbookmark
14. unread / all / starred の filter
15. SQLite FTS5 による簡易検索
16. OPML import
17. OPML export
18. Docker Compose 起動
19. README
20. 基本的な unit test

# 後回しにする機能

以下は MVP では実装しなくてよいです。

* Fever API
* OIDC
* PWA
* i18n
* 完全な keyboard shortcuts
* optimistic update
* infinite scroll
* 高度な mobile drawer UI
* offline support
* full OpenAPI
* 複数ユーザー対応
* 高度な権限管理
* PostgreSQL 対応
* Kubernetes 対応

ただし、後で追加できるように設計上は邪魔しないでください。

# 重要な設計方針

## 1. Feed 設定と fetch state を分ける

Feed の静的情報と、fetch 実行時の状態は別 table にしてください。

`feeds` には以下のような静的情報を置きます。

* id
* group_id
* title
* feed_url
* site_url
* description
* suspended
* created_at
* updated_at

`feed_fetch_state` には以下のような実行時情報を置きます。

* feed_id
* etag
* last_modified
* cache_control
* expires_at
* retry_after_until
* last_checked_at
* next_check_at
* last_http_status
* last_success_at
* last_error_at
* last_error
* consecutive_failures

## 2. Pull policy は pure function にする

feed の次回取得時刻や backoff の計算は pure function に切り出してください。

例:

```rust
pub struct PullInput {
    pub now: OffsetDateTime,
    pub last_success_at: Option<OffsetDateTime>,
    pub last_error_at: Option<OffsetDateTime>,
    pub consecutive_failures: u32,
    pub cache_control: Option<String>,
    pub retry_after_until: Option<OffsetDateTime>,
}

pub struct PullDecision {
    pub should_fetch: bool,
    pub next_check_at: OffsetDateTime,
    pub reason: String,
}
```

この部分は unit test を書きやすくしてください。

## 3. HTML は必ず sanitize する

外部 feed の content / summary / description をそのまま表示しないでください。

記事本文を表示する前に `ammonia` などで sanitize してください。

最低限、以下の危険を避けてください。

* script injection
* event handler attributes
* unsafe iframe
* javascript: URL
* style injection
* clickjacking 的な危険

MVP では完璧でなくてもよいですが、外部 HTML を無加工で表示する実装は禁止です。

## 4. SSRF 対策を入れる

feed URL を fetch するため、SSRF 対策を必ず入れてください。

最低限、以下を実装してください。

1. URL scheme は http / https のみに制限
2. file://, ftp://, gopher:// などは禁止
3. localhost / 127.0.0.1 / ::1 へのアクセスは禁止
4. private IP range へのアクセスは禁止
5. link-local address へのアクセスは禁止
6. redirect 回数を制限
7. redirect 先も validation
8. request timeout
9. response body size limit
10. User-Agent の明示
11. Content-Type の確認
12. 必要なら `ALLOW_PRIVATE_FEEDS` のような env で private network feed を明示許可

## 5. Bookmark は snapshot として保存する

bookmark は source item に依存しすぎないようにしてください。

source item が削除されても bookmark が残るように、以下のような snapshot fields を持たせてください。

* item_id nullable
* link
* title
* content
* feed_title
* pub_date
* created_at

# DB schema 案

SQLite migration として以下をベースにしてください。

必要に応じて改善して構いません。

```sql
PRAGMA foreign_keys = ON;

CREATE TABLE groups (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

INSERT INTO groups (id, name, created_at, updated_at)
VALUES (1, 'Default', strftime('%s','now'), strftime('%s','now'));

CREATE TABLE feeds (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  group_id INTEGER NOT NULL,
  title TEXT NOT NULL,
  feed_url TEXT NOT NULL UNIQUE,
  site_url TEXT NOT NULL DEFAULT '',
  description TEXT NOT NULL DEFAULT '',
  suspended INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY(group_id) REFERENCES groups(id)
);

CREATE TABLE feed_fetch_state (
  feed_id INTEGER PRIMARY KEY,
  etag TEXT NOT NULL DEFAULT '',
  last_modified TEXT NOT NULL DEFAULT '',
  cache_control TEXT NOT NULL DEFAULT '',
  expires_at INTEGER NOT NULL DEFAULT 0,
  retry_after_until INTEGER NOT NULL DEFAULT 0,
  last_checked_at INTEGER NOT NULL DEFAULT 0,
  next_check_at INTEGER NOT NULL DEFAULT 0,
  last_http_status INTEGER NOT NULL DEFAULT 0,
  last_success_at INTEGER NOT NULL DEFAULT 0,
  last_error_at INTEGER NOT NULL DEFAULT 0,
  last_error TEXT NOT NULL DEFAULT '',
  consecutive_failures INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY(feed_id) REFERENCES feeds(id) ON DELETE CASCADE
);

CREATE TABLE items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  feed_id INTEGER NOT NULL,
  guid TEXT NOT NULL,
  title TEXT NOT NULL DEFAULT '',
  link TEXT NOT NULL DEFAULT '',
  content TEXT NOT NULL DEFAULT '',
  author TEXT NOT NULL DEFAULT '',
  pub_date INTEGER NOT NULL DEFAULT 0,
  unread INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  UNIQUE(feed_id, guid),
  FOREIGN KEY(feed_id) REFERENCES feeds(id) ON DELETE CASCADE
);

CREATE INDEX idx_items_unread ON items(unread) WHERE unread = 1;
CREATE INDEX idx_items_pub_date ON items(pub_date);
CREATE INDEX idx_items_feed_unread ON items(feed_id, unread);

CREATE VIRTUAL TABLE items_fts USING fts5(
  title,
  content,
  content='items',
  content_rowid='id',
  tokenize='unicode61'
);

CREATE TRIGGER items_ai AFTER INSERT ON items BEGIN
  INSERT INTO items_fts(rowid, title, content)
  VALUES (new.id, new.title, new.content);
END;

CREATE TRIGGER items_ad AFTER DELETE ON items BEGIN
  INSERT INTO items_fts(items_fts, rowid, title, content)
  VALUES ('delete', old.id, old.title, old.content);
END;

CREATE TRIGGER items_au AFTER UPDATE ON items BEGIN
  INSERT INTO items_fts(items_fts, rowid, title, content)
  VALUES ('delete', old.id, old.title, old.content);
  INSERT INTO items_fts(rowid, title, content)
  VALUES (new.id, new.title, new.content);
END;

CREATE TABLE bookmarks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  item_id INTEGER,
  link TEXT NOT NULL UNIQUE,
  title TEXT NOT NULL DEFAULT '',
  content TEXT NOT NULL DEFAULT '',
  feed_title TEXT NOT NULL DEFAULT '',
  pub_date INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  FOREIGN KEY(item_id) REFERENCES items(id) ON DELETE SET NULL
);
```

# 推奨ディレクトリ構成

以下をベースにしてください。

```txt
rss-reader-rs/
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

# ルーティング案

MVP では以下を実装してください。

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

htmx の partial response に対応してください。

例えば:

* article list の部分更新
* article detail の差し替え
* feed tree の部分更新
* read/unread toggle 後の row 更新
* bookmark toggle 後の button 更新

# UI 方針

MVP では、Fusion の UI を完全再現しなくてよいです。

ただし、以下の構造を参考にしてください。

```txt
Desktop:
  left sidebar:
    - app name
    - search
    - All / Unread / Starred
    - groups
    - feeds
    - settings / feed management

  main:
    - article list
    - article detail
```

スマホでは最初から完璧な drawer UI は不要です。
CSS で最低限読みやすくしてください。

# htmx で実現したい操作

以下を htmx で実装してください。

* read/unread toggle
* bookmark toggle
* article detail load
* feed refresh
* feed create/edit/delete 後の list 更新
* group create/edit/delete 後の sidebar 更新
* search result の表示
* mark all as read

# Feed fetcher 仕様

Feed fetcher は以下を実装してください。

1. feed URL を validation
2. HTTP GET
3. ETag があれば `If-None-Match`
4. Last-Modified があれば `If-Modified-Since`
5. 304 の場合は parse せず fetch state 更新
6. 200 の場合は body を取得
7. response body size limit を適用
8. feed-rs で parse
9. item を upsert
10. fetch state を更新
11. エラー時は `consecutive_failures` を増やす
12. exponential backoff で `next_check_at` を設定

# Feed parser 仕様

RSS / Atom / JSON Feed を可能な範囲で parse してください。

MVP では `feed-rs` を使ってください。

item の guid は以下の優先順位で決めてください。

1. feed item の id
2. link
3. title + pub_date の hash

item の content は以下の優先順位で決めてください。

1. content
2. summary
3. title

content は保存前または表示前に sanitize してください。

# Pull worker 仕様

Pull worker は tokio task として起動してください。

環境変数で以下を設定できるようにしてください。

* `APP_PULL_INTERVAL_SECONDS`
* `APP_PULL_TIMEOUT_SECONDS`
* `APP_PULL_CONCURRENCY`
* `APP_PULL_MAX_BACKOFF_SECONDS`
* `APP_ALLOW_PRIVATE_FEEDS`

worker loop は以下のようにしてください。

```txt
loop:
  1. now を取得
  2. next_check_at <= now かつ suspended = false の feeds を取得
  3. concurrency limit を守って fetch
  4. 成功 / 失敗に応じて feed_fetch_state を更新
  5. 一定時間 sleep
```

# 認証方針

MVP では single-user password auth でよいです。

環境変数:

* `APP_PASSWORD`
* `APP_ALLOW_EMPTY_PASSWORD`

要件:

* password が設定されていれば login 必須
* cookie は HttpOnly
* SameSite=Lax
* production では Secure cookie を有効化できる設計
* 空パスワードは local trusted environment のみ許可
* session secret は env から指定可能にする

OIDC は後回しにしてください。

# OPML 仕様

MVP では以下を実装してください。

## Import

* OPML ファイルを upload
* outline から feed URL を抽出
* group 情報があれば group に反映
* 重複 feed URL は skip
* import 結果を表示

## Export

* 現在の group / feeds を OPML として出力
* `Content-Type: text/xml`

# Search 仕様

SQLite FTS5 を使って検索してください。

対象:

* item title
* item content

MVP では feed search と item search を分けてもよいです。
将来的に unified search にできます。

# Fever API について

Fever API は MVP では実装しないでください。

ただし、将来的に `/fever`, `/fever/`, `/fever.php` を追加できるよう、routes の構成を邪魔しないでください。

Fever API を追加すると、Reeder / Unread / FeedMe のような外部 RSS client から利用できるようになります。

# 実装ステップ

以下の順番で進めてください。

## Step 1: Project scaffold

* Cargo project 作成
* axum server 起動
* config 読み込み
* tracing 設定
* healthz endpoint
* static file serving

## Step 2: SQLite setup

* sqlx setup
* migrations
* repository layer
* DB connection pool
* groups / feeds / items の基本 CRUD

## Step 3: Auth

* password login
* session cookie
* auth middleware
* login page

## Step 4: Feed fetch / parse

* url_guard
* reqwest client
* feed-rs parser
* sanitizer
* item upsert

## Step 5: Pull worker

* pull_policy
* background worker
* manual refresh
* fetch state update

## Step 6: Reader UI

* layout
* sidebar
* unread/all/starred
* article list
* article detail
* read/unread
* bookmark

## Step 7: Search

* FTS5
* search page
* search result partial

## Step 8: OPML

* import
* export

## Step 9: Docker

* Dockerfile
* docker-compose.yml
* `.env.example`
* README

## Step 10: Tests

* unit tests
* integration tests where practical

# テスト方針

最低限、以下の unit test を書いてください。

* URL validation
* private IP 判定
* redirect URL validation
* pull policy
* exponential backoff
* feed parser
* guid fallback
* sanitizer
* item upsert logic
* OPML import parser
* OPML export generator

可能であれば、以下の integration test も追加してください。

* feed 登録
* manual refresh
* item 一覧表示
* read/unread toggle
* bookmark toggle
* search
* OPML import/export

# コーディング方針

以下を守ってください。

1. まず設計を簡潔に説明する
2. その後、ファイル構成を作る
3. MVP が動くところまで実装する
4. 不要に抽象化しすぎない
5. ただし、fetch / parse / pull policy / repository / sanitizer は分離する
6. エラーは AppError に集約する
7. tracing で基本ログを出す
8. SQL は明示的に書く
9. ORM 的な過剰抽象化は避ける
10. README に起動方法を書く
11. Docker Compose で `docker compose up --build` できるようにする
12. `.env.example` を用意する
13. テストしやすい pure function を増やす
14. まず動くものを作り、その後に改善する

# 非目標

MVP では以下はやらないでください。

* Fusion 完全互換
* Fever API
* OIDC
* PWA
* i18n
* React SPA
* TanStack Query
* Zustand
* shadcn/ui
* Tailwind 前提の大規模 UI
* 複数ユーザー
* PostgreSQL
* Kubernetes
* AI 機能

# 期待する最初の出力

まず、実装を始める前に以下を出してください。

1. この要件に対する実装計画
2. 採用技術の理由
3. Fusion から参考にする点
4. Fusion からあえて削る点
5. ディレクトリ構成
6. DB schema
7. 主要 module の役割
8. 実装ステップ
9. MVP 完了条件

その後、実際にコードを生成・編集してください。

作業中に判断が必要な場合は、軽量性・安全性・保守性を優先して、妥当なデフォルトで進めてください。

