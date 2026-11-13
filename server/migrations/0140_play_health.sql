-- Playback health, per client class and host and day, with no person in it.
--
-- `0110_play.sql` made two tables and neither was ever used. `reachability (class, host, tries,
-- starts)` has no insert, update or select anywhere in the tree. `play_events` has a writer
-- (`server-play/src/log.rs`) reached through `POST /v1/play/{id}/events`, and no client has ever
-- called that route: the core builds `/v1/play`, `/more` and `/marks` and nothing else.
--
-- The core now reports what happened during a play, so the question is which of the two tables
-- takes it. A rebuffer is a fact about a host, a network and a build. It is not a fact about a
-- person, and `play_events` keyed it to `profile_id` behind the personal-rows consent answer,
-- which made playback health something one viewer could switch off and made a row about a stream
-- into a row about a viewer. `reachability` is already declared `Keyed::NoOne` with
-- `Asked::Working` and `Removal::Keeps` in `server-accounts::holdings`, and the privacy policy
-- already accounts for it under "Not about you ... counts of how often playback reached each
-- source". So the table built for this is the one that gets it: no new table, no new policy text,
-- and no profile column to leave out, because there is none to leave out.
--
-- There was nothing to migrate. Both tables are empty on every deployment, because nothing ever
-- wrote to either.
--
-- A day is added so a number reads back as a series rather than as a running total that can never
-- be compared with anything, and a fault word is added so a failure keeps its error class without
-- a column per class and without a schema change when the contract learns a new word. The key
-- becomes (class, host, day, fault): the row with the empty fault carries the session counts, and
-- a row per error class carries that class's failures.
--
-- What is counted, and what is deliberately not:
--   tries          one per session that opened an attempt at this host      (fault '')
--   starts         one per session that showed a first frame from this host (fault '')
--   start_ms_total the sum of those first-frame delays, so the mean is total / starts
--   rebuffers      a count, never a duration                                (fault '')
--   gave_up        one per session that ended here having never shown a frame (fault '')
--   failures       one per driver error, on the row of its error class
--
-- There is no column for a rebuffer's length, and that is the point rather than an omission.
-- `Media3Player.kt` ticks `Position` at 1 Hz and only while playing, so a measured rebuffer is
-- quantised to a second and biased upward. A count does not have that problem. Leaving the column
-- out means no later writer can put a duration here and have it read as if it were precise.
--
-- Only the key moves. No column and no table goes, and no column changes type, so a build of the
-- previous release keeps working against this schema. It had no statement that touched this
-- table in any case.

alter table reachability add column if not exists day date not null default current_date;

alter table reachability add column if not exists fault text not null default '';

alter table reachability drop constraint if exists reachability_pkey;

alter table reachability add primary key (class, host, day, fault);

alter table reachability
  add column if not exists rebuffers integer not null default 0 check (rebuffers >= 0);

alter table reachability
  add column if not exists failures integer not null default 0 check (failures >= 0);

alter table reachability
  add column if not exists gave_up integer not null default 0 check (gave_up >= 0);

alter table reachability
  add column if not exists start_ms_total bigint not null default 0 check (start_ms_total >= 0);

-- A host and a fault word arrive from a client, so their length is held here and not only in the
-- fold that writes them. The table is empty on every deployment, so these apply to no row.
alter table reachability
  add constraint reachability_host_length check (char_length(host) <= 120);

alter table reachability
  add constraint reachability_fault_length check (char_length(fault) <= 40);

-- The read path walks days backwards and groups by (class, host, day), which is this order.
create index reachability_day_idx on reachability (day desc, class, host);
