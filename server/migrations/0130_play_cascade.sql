-- Playback telemetry goes when the profile goes.
--
-- `0110_play.sql` gave `play_events.profile_id` `on delete set null`. The schema has one other
-- `set null` on a profile key, `device_sessions.active_profile` (`0001_accounts.sql:104`), and
-- that one is right: the row it nulls is itself carried away by its own cascade from
-- `auth.users`, so nulling it only clears the active profile of a session that outlives a
-- profile. `play_events` had no key to the account at all, which made it the one row about a
-- person that could outlive that person: deleting an account left its
-- playback rows in place with the profile nulled, and the prune that removes them reads `ts`
-- alone, so they went no sooner than anyone else's, up to 30 days later. Three help texts promise
-- that deleting an account deletes what we hold at once (`help/en.json:160`, `:1114`, `:1144`),
-- and a nulled row is still a timestamped failure and timing sample with a streaming host in it,
-- which lines up by time with `client_reports`. The key now carries the row away in the same
-- statement that removes the account.
--
-- `server-accounts::holdings` declares this as `Removal::Cascade` for `play_events`, and the test
-- beside that list reads these files and fails if the chain does not really cascade, so the two
-- cannot drift.
--
-- The column stays nullable. Nothing writes a null (`server-play/src/log.rs` always binds the
-- profile), the rows that hold one are exactly the orphans this file removes, and narrowing the
-- column would take a table scan for no further guarantee.

-- The name is the one Postgres gives a column level key in `create table play_events`, so it is
-- the name on every database this chain made. An operator who renamed it by hand drops theirs
-- first, or this file stops on the name it adds.

delete from play_events where profile_id is null;

alter table play_events drop constraint if exists play_events_profile_id_fkey;

alter table play_events
  add constraint play_events_profile_id_fkey
  foreign key (profile_id) references profiles (id) on delete cascade;
