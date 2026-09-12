-- Local intake evidence is not replicated to other devices.
CREATE TABLE captures (
    id TEXT PRIMARY KEY NOT NULL,
    owner TEXT NOT NULL,
    request_id TEXT NOT NULL,
    payload_hash TEXT NOT NULL,
    body TEXT NOT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    UNIQUE(owner, request_id)
);

CREATE TABLE capture_ai_calls (
    id TEXT PRIMARY KEY NOT NULL,
    month TEXT NOT NULL,
    cost_micros BIGINT NOT NULL CHECK(cost_micros >= 0)
);
CREATE INDEX capture_ai_calls_month ON capture_ai_calls(month);

CREATE INDEX captures_processing ON captures(json_extract(body,'$.status'),json_extract(body,'$.leaseUntil'));

CREATE TABLE activity_source_evidence (activity_id TEXT NOT NULL REFERENCES activities(id) ON DELETE CASCADE, source_id TEXT NOT NULL, body TEXT NOT NULL, PRIMARY KEY(activity_id,source_id));
