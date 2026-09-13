CREATE TABLE server_sync_client (
 id INTEGER PRIMARY KEY CHECK(id = 1), endpoint TEXT NOT NULL, server_id TEXT NOT NULL,
 cursor INTEGER NOT NULL, paused INTEGER NOT NULL DEFAULT 0, last_sync TEXT
);
CREATE TABLE server_sync_client_queue (
 seq INTEGER PRIMARY KEY AUTOINCREMENT, event_id TEXT NOT NULL UNIQUE,
 entity TEXT NOT NULL, entity_id TEXT NOT NULL, request_json TEXT NOT NULL,
 conflict_json TEXT
);
