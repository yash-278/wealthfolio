CREATE TABLE server_sync_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    server_id TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 0
);
INSERT INTO server_sync_state (id, server_id) VALUES (1, lower(hex(randomblob(16))));
CREATE TABLE server_sync_events (
    seq INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT NOT NULL UNIQUE,
    entity TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    op TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    payload TEXT NOT NULL,
    request_json TEXT
);
CREATE TABLE server_sync_revisions (
    entity TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    last_event_id TEXT NOT NULL,
    PRIMARY KEY (entity, entity_id)
);
