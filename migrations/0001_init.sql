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

CREATE INDEX idx_feeds_group_id ON feeds(group_id);

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

CREATE INDEX idx_bookmarks_created_at ON bookmarks(created_at);
