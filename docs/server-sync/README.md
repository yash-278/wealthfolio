# Own-server sync, protocol 1

The iOS and desktop app share an authenticated client, durable offline queue,
connection settings and conflict handling without Wealthfolio Connect. The app
builds and launches on the iOS 27 Simulator and a physical iPhone. Server
pairing and end-to-end sync on the physical phone still require user acceptance
testing. iOS background scheduling and Quick Add receipt/review synchronization
are not implemented. See [iOS testing](IOS-TESTING.md) for the disposable test
setup.

## Authority and scope

The server accepts changes into its existing application database. The normal
web app reads the same records. The phone retains a local copy and uploads
queued edits when connected and foregrounded. Each accepted edit must name the
revision it was based on; a stale revision produces a conflict. The server never
uses a phone's clock to decide which edit wins.

This stage reuses the existing user-managed sync dataset and replay adapters.
That includes manual/CSV financial activities, accounts, assets, goals, budgets,
spending configuration and the other entities in `APP_SYNC_TABLES`, subject to
its existing export filters. It is not a complete database mirror. Broker-owned
rows, provider-fetched quotes and derived calculations retain the upstream sync
filters. Do not advertise full brokerage parity until those paths are covered.

Uploads currently support entities with a single-column storage mapping. These
four special entities are rejected before a mutation:
`broker_activity_user_patch`, `custom_taxonomy`,
`spending_preset_rule_deletion`, and `addon_storage`. They may appear in
downloads because web-side edits use the existing outbox. Their upload adapters
need a later implementation.

Quick Add's posted manual activities participate through the existing activity
outbox. Capture receipts, unresolved candidates, source text, configuration and
correction rules need separate sync coverage. AI provider credentials and owner
sessions are not part of the downloaded snapshot.

## Authentication and opt-in

All routes are under `/api/v1` and use the existing owner session middleware.
Log in through the existing password or OIDC flow. There is no new
unauthenticated pairing endpoint and no shared device secret embedded in the
app. A server with no configured owner authentication rejects sync requests.
Clients must use HTTPS outside local development and keep session material in
the OS credential store.

`POST /server-sync` explicitly enables journaling. Enabling again is idempotent.
New installations leave it disabled. Existing records are supplied by the
initial snapshot; enabling does not backfill old outbox events into the change
feed.

Wealthfolio Connect device sync and own-server sync cannot operate
simultaneously on this server. Enabling rejects an existing Connect device
configuration, and Connect device-sync/crypto routes reject requests after
own-server sync is enabled. Broker connections are separate from device sync.

## Endpoints

| Method | Path                                                 | Result                                                      |
| ------ | ---------------------------------------------------- | ----------------------------------------------------------- |
| GET    | `/server-sync`                                       | `serverId`, `cursor`, `enabled` where 1 means enabled       |
| POST   | `/server-sync`                                       | Enable and return the current head                          |
| GET    | `/server-sync/snapshot`                              | Filtered SQLite snapshot with revision metadata             |
| GET    | `/server-sync/changes?serverId=…&cursor=…&limit=100` | Ordered page after the supplied cursor                      |
| POST   | `/server-sync/changes`                               | Apply one mutation, acknowledge a retry, or return conflict |

Snapshot response headers are `X-Sync-Server-Id`, `X-Sync-Cursor` and
`X-Sync-Protocol-Version: 1`. Snapshot rows and the cursor are read in the same
SQLite transaction. The SQLite file also contains `server_sync_revisions` with
`entity`, `entity_id` and `last_event_id`. This metadata is separate from
Connect's revision state. The file uses the existing snapshot table schema; it
is not a full application database and must not replace the phone's entire
database file.

The change page contains `serverId`, `cursor`, `hasMore`, and `changes`. Each
change has `seq`, `eventId`, `entity`, `entityId`, `op`, `timestamp`, and
`payload`. Payloads use the existing storage sync format, not the web CRUD
request format. The cursor advances only to the last returned event. Page sizes
must be between 1 and 500. The caller supplies the pinned server ID, so a
different server cannot silently reuse the old cursor. Cursors ahead of the
server are rejected.

Example upload with synthetic data:

```json
{
  "serverId": "SERVER_ID_FROM_BOOTSTRAP",
  "eventId": "019950ab-38b3-7000-8000-000000000001",
  "entity": "goal",
  "entityId": "example-goal",
  "op": "create",
  "baseEventId": null,
  "payload": {
    "id": "example-goal",
    "title": "Example goal",
    "target_amount": 100
  }
}
```

A successful new write returns `{"status":"applied","seq":1}`. Repeating the
same event returns `{"status":"duplicate","seq":1}`, even after later edits.
Reusing an accepted event ID with different content is rejected. Accepted events
and their receipts persist in the database. Request bodies are limited to 1 MB.

Updates and deletes require `baseEventId` to match the server's current
revision. A mismatch returns HTTP 409 with `status: "conflict"` and
`currentEventId`. Creating over an existing record, updating a missing record,
or recreating a deleted ID also conflicts. A deleted entity keeps a revision
tombstone. Explicit recreation requires a new entity ID. A failed replay rolls
back the record, revision and journal entry together. Invalid protocol inputs
return 400; storage failures return a generic 500 without exposing database
values.

## Native client behavior

Settings → Your server accepts an HTTPS origin and the server owner password.
The password is used for login only; the owner session is kept in the existing
OS credential store (Keychain on Apple platforms). The server supports OIDC, but
this client currently supports password login only. Redirects are refused. Debug
builds also accept loopback HTTP for the disposable Simulator server.

Initial pairing requires a fresh installation with no user records or Connect
pairing. Import and revision metadata are committed together. Existing local
records are never silently replaced. A paired app can sign in again to the same
server; switching servers or disconnecting is not implemented in this version.
Pause keeps pairing and queued edits, including edits made while paused.

Sync runs on startup, foreground, network reconnection and every 30 seconds
while visible; Sync now is also available. iOS may suspend the app in the
background, so this does not promise background delivery. Uploads preserve event
IDs across restarts and timeouts. Downloads advance the cursor transactionally.
A stale edit pauses the upload queue, preserves the local record, and offers an
explicit confirmation to discard that record's local edits in favor of the
server version. Keeping/merging the local version is a follow-up workflow.
Unsupported upload entities also stop the queue and surface a sync error.

## Client implementation contract

1. Preserve local unsent work before pairing. Do not silently overwrite an
   existing local database during bootstrap.
2. Enable sync, download a snapshot, validate the protocol/schema, and import
   the allowed tables plus revision metadata atomically. Pin the server ID and
   cursor.
3. Keep the base server revision and a stable event ID with every queued edit.
   Upload dependent records in order, one at a time. Stop dependent uploads
   after a conflict. A network timeout must retry the same event, not generate
   another.
4. Pull every page after the saved cursor. Apply changes, update server
   revisions and advance the cursor in one local transaction. Never discard
   pending local edits when applying a remote change; preserve both sides for
   conflict review.
5. Acknowledge accepted outbox records without skipping unrelated server events.
   The sequence returned by an upload is a receipt, not permission to advance
   the download cursor. Echoed events identify the already accepted local
   change.
6. Recalculate derived views after accepted downloads, using the existing domain
   event flow. Background iOS execution remains follow-up work; sync status is
   available in Settings → Your server.

## Retention and recovery

The first version retains its journal and accepted request identities
indefinitely. It does not prune records before devices have acknowledged them.
Retention and a rebootstrap policy must be designed before enabling unattended
long-term use.

A database restore must invalidate existing device pairings and rotate the
server identity before clients reconnect. Automatic restore integration is not
included in this stage. A cursor-ahead check detects some restores, but cannot
detect every rollback of the same database identity. Do not reconnect old
clients to a restored copy without that reset procedure.

## Verification

Run the focused storage tests, including the existing replay and snapshot tests:

```sh
cargo test -p wealthfolio-storage-sqlite sync::app_sync --lib
cargo test -p wealthfolio-server --test server_sync_api
cargo test -p wealthfolio-server-sync --test round_trip
```

These tests use temporary databases. They cover retries, changed-content reuse,
concurrent edits, deletion tombstones, web-side change tracking, pagination,
invalid payload rollback, snapshot data/revision consistency, authentication,
body limits, opt-in persistence and separation from Connect revision resets.
