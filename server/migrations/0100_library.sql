-- List, rating, progress and taste. Rows are rebuilt from the old ones at the switch: one row and
-- one stamp for list and rating caused lost writes, so each field keeps its own version here.

create table title_state (
  profile_id        uuid not null references profiles (id) on delete cascade,
  kind              text not null check (kind in ('movie', 'tv')),
  title_id          integer not null check (title_id > 0),
  in_list           boolean not null default false,
  list_version      bigint not null default 0,
  list_changed_at   timestamptz,
  added_at          timestamptz,
  rating            text check (rating in ('down', 'up', 'love')),
  rating_version    bigint not null default 0,
  rating_changed_at timestamptz,
  seq               bigint not null,
  primary key (profile_id, kind, title_id)
);
create index title_state_list_idx on title_state (profile_id, added_at desc) where in_list;
create index title_state_seq_idx on title_state (profile_id, seq);

create table episode_progress (
  profile_id uuid not null references profiles (id) on delete cascade,
  kind       text not null check (kind in ('movie', 'tv')),
  title_id   integer not null check (title_id > 0),
  season     integer not null default 0 check (season >= 0),
  episode    integer not null default 0 check (episode >= 0),
  position_s integer not null default 0 check (position_s >= 0),
  duration_s integer not null default 0 check (duration_s >= 0),
  completed  boolean not null default false,
  version    bigint not null default 0,
  changed_at timestamptz not null,
  seq        bigint not null,
  primary key (profile_id, kind, title_id, season, episode)
);
create index episode_progress_recent_idx on episode_progress (profile_id, changed_at desc);
create index episode_progress_seq_idx on episode_progress (profile_id, seq);

-- Hide is one fact about a title, covering every episode of a series.
create table hidden_titles (
  profile_id uuid not null references profiles (id) on delete cascade,
  kind       text not null check (kind in ('movie', 'tv')),
  title_id   integer not null check (title_id > 0),
  hidden_at  timestamptz not null,
  seq        bigint not null,
  primary key (profile_id, kind, title_id)
);

-- One writer per profile: the counter row is locked for each batch, and `seq` is the change cursor.
create table profile_library (
  profile_id         uuid primary key references profiles (id) on delete cascade,
  seq                bigint not null default 0,
  history_cleared_at timestamptz,
  history_cleared_seq bigint
);

-- Idempotency: an op id answers the same outcome when it arrives twice.
create table applied_ops (
  profile_id uuid not null references profiles (id) on delete cascade,
  op_id      text not null check (char_length(op_id) <= 64),
  outcome    text not null check (outcome in ('applied', 'superseded', 'rejected')),
  at         timestamptz not null default now(),
  primary key (profile_id, op_id)
);
create index applied_ops_at_idx on applied_ops (at);

create table profile_settings (
  profile_id        uuid primary key references profiles (id) on delete cascade,
  autoplay_next     boolean not null default true,
  autoplay_previews boolean not null default true,
  audio_language    text,
  subtitle_language text,
  subtitle_style    jsonb not null default '{}'::jsonb,
  ui_language       text,
  data_usage        text not null default 'auto' check (data_usage in ('auto', 'low', 'medium', 'high')),
  notifications     jsonb not null default '{}'::jsonb check (jsonb_typeof(notifications) = 'object'),
  game_handle       text check (game_handle is null or game_handle ~ '^[A-Za-z0-9]{3,16}$'),
  gender            text check (gender is null or gender in ('woman', 'man', 'none')),
  trailers_watched  jsonb not null default '[]'::jsonb
    check (jsonb_typeof(trailers_watched) = 'array' and jsonb_array_length(trailers_watched) <= 40),
  -- Change time of each field, so an older write to one setting never undoes a newer one.
  field_times       jsonb not null default '{}'::jsonb check (jsonb_typeof(field_times) = 'object'),
  version           bigint not null default 0,
  updated_at        timestamptz not null default now(),
  seq               bigint not null default 0
);
create unique index profile_settings_game_handle_key on profile_settings (lower(game_handle)) where game_handle is not null;

-- Trailers that refused to play for this title. The newest 300 of a profile are kept.
create table refused_trailers (
  profile_id uuid not null references profiles (id) on delete cascade,
  kind       text not null check (kind in ('movie', 'tv')),
  title_id   integer not null check (title_id > 0),
  at         timestamptz not null,
  primary key (profile_id, kind, title_id)
);
create index refused_trailers_recent_idx on refused_trailers (profile_id, at desc);

create table profile_taste (
  profile_id    uuid primary key references profiles (id) on delete cascade,
  genre_weights jsonb not null default '{}'::jsonb,
  taste_version integer not null default 0,
  updated_at    timestamptz not null default now()
);

create table feed_events (
  id         bigint generated always as identity primary key,
  profile_id uuid not null references profiles (id) on delete cascade,
  ts         timestamptz not null default now(),
  event      text not null check (event in ('impression', 'click', 'start', 'complete', 'like', 'skip', 'add_list', 'remove_list')),
  kind       text check (kind in ('movie', 'tv')),
  title_id   integer,
  row_title  text,
  position   smallint
);
create index feed_events_profile_ts_idx on feed_events (profile_id, ts desc);
create index feed_events_ts_idx on feed_events (ts);
