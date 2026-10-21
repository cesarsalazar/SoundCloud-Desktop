# Integration scenarios (1.T)

Real chat-core clients against a dev chat-api. Every test is `#[ignore]`, plain `cargo test` skips them.
The tests write sessions straight into the database, so point them only at a throwaway dev stack.

## Stack

- Postgres with the Backend migrations applied (the `scd-pg` podman container works) and Redis in podman.
- Keys dir: `ock-1` holds the 32 raw bytes of `ock_secret` from Backend `utils/chat-wire/vectors/request.json`,
  `kek-1` and `reports-kek-1` hold 32 random bytes, `chat-admin-token` any token.
- chat-api from Backend `api`:

```sh
API_ROLE=rt PORT=3417 CHAT_ACCESS=all CHAT_KEYS_DIR=<keys> \
DATABASE_URL=postgres://scd:dev@127.0.0.1:5432/<db> \
REDIS_URL=redis://127.0.0.1:<port> REDIS_RT_URL=redis://127.0.0.1:<port> \
target/debug/api
```

## Run

```sh
export CHAT_IT_ORIGIN=http://127.0.0.1:3417
export CHAT_IT_PSQL="podman exec -i scd-pg psql -U scd -d <db>"
export CHAT_IT_ADMIN_TOKEN_FILE=<keys>/chat-admin-token
export SC_CHAT_DEV_OCK=1:dde332903df0f9e9fbc91a814131b5bcea28b46f5728e2a7e28700ed4a6c2d4d
cargo test --test integration -- --ignored
```

`SC_CHAT_DEV_OCK` is read only by debug builds. Drop the database, the Redis container and the keys dir after.

## Scenarios

| File | Checks |
|---|---|
| `dm.rs` | DM both ways, `mine`, read marks |
| `group.rs` | group fan-out, a promotion arrives as a `role` row |
| `room.rs` | room chat opens from the seat, ends with a local purge |
| `request.rs` | 1 h session rule, 3 posts before accept, decline closes the chat |
| `block.rs` | block refuses posts, unblock lets them through |
| `report.rs` | evidence decrypts on the admin route, decrypt is audited |
| `relogin.rs` | unbound session, bound to another device, sealed pull and push links bind the new device |
| `devices.rs` | young device rules, not me revokes |
| `history.rs` | a new device reads only what came after it |
| `replay.rs` | a captured request sent again is `chat_replay` |
| `tiers.rs` | a dead direct route falls through to the relay |
