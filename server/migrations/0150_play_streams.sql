-- What is playing right now, so an account's simultaneous streams can be counted.
--
-- The count has to be in the database and not in a server. `server-play/src/gate.rs` already
-- keeps plays in a `moka` cache, and that cache is per process: with two servers answering
-- `/v1/play`, each would count only the plays it started itself and an account would get its
-- allowance once per server. A row here is seen by every server, and `fresh_at` is written with
-- the database's `now()`, so two servers whose clocks disagree still agree on what is live.
--
-- One row per (session, profile) rather than per play. A device plays one thing at a time, so a
-- viewer who changes title writes over their own row and never takes a second slot. Nothing here
-- limits how many devices an account registers; `account_devices` is untouched, and this counts
-- only what is playing at this moment.
--
-- `fresh_at` is what keeps the count honest in the other direction. A counter that only goes up
-- locks a viewer out of their own account the first time a device loses power mid-film, so a row
-- counts only while it is fresh: the core sends a position every 30 s while a play runs
-- (`platform/core/src/player.rs:17`), and the window in `server-play/src/streams.rs` is 150 s,
-- five missed beats. There is no prune job: a claim deletes that account's stale rows in the same
-- transaction it counts in, which is the only moment they matter.
--
-- `play_id` is the play this row is for. A beat or a release that names an older play of the same
-- device does nothing, so a late message from a play that is over cannot hold a slot open or take
-- away the one the device holds now.
--
-- Three keys, all cascading. The session key means ending a session (`DELETE /v1/devices`) frees
-- its slot in the same statement; the profile key means deleting a profile does; the account key
-- means deleting the account takes every row with it, which is what `Removal::Cascade` in
-- `server-accounts::holdings` claims and what the test beside that list proves.

create table play_streams (
  session_id uuid not null references device_sessions (id) on delete cascade,
  profile_id uuid not null references profiles (id) on delete cascade,
  user_id    uuid not null references auth.users (id) on delete cascade,
  play_id    text not null check (char_length(play_id) <= 64),
  fresh_at   timestamptz not null default now(),
  primary key (session_id, profile_id)
);

-- Both statements of a claim read by account: the sweep of stale rows and the count of live ones.
create index play_streams_user_idx on play_streams (user_id, fresh_at desc);
