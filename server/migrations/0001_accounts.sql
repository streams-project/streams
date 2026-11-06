-- Accounts, profiles, devices and sessions. This part of the chain holds no video words, so a
-- project that takes only accounts stops at file 0099. File 0000 makes `auth.users`. Rules that
-- used to live in functions and triggers moved to Rust (crate server-accounts); constraints stay here.

create table account_details (
  user_id            uuid primary key references auth.users (id) on delete cascade,
  first_name         text not null default '' check (char_length(first_name) <= 60),
  last_name          text not null default '' check (char_length(last_name) <= 60),
  test_participation boolean not null default true,
  profile_transfer   boolean not null default true,
  rating_system      text not null default 'auto' check (rating_system in ('auto', 'uk', 'us')),
  updated_at         timestamptz not null default now()
);

create table rating_system_by_country (
  country text primary key check (country ~ '^[A-Z]{2}$'),
  system  text not null check (system in ('uk', 'us'))
);
insert into rating_system_by_country (country, system) values
  ('US', 'us'), ('CA', 'us'), ('MX', 'us'), ('PR', 'us'), ('GU', 'us'), ('VI', 'us'), ('AS', 'us'), ('MP', 'us'),
  ('GB', 'uk'), ('IE', 'uk');

create table profiles (
  id            uuid primary key default gen_random_uuid(),
  user_id       uuid not null references auth.users (id) on delete cascade,
  name          text not null check (char_length(btrim(name)) between 1 and 30),
  avatar        text not null default 'default' check (char_length(avatar) <= 64),
  kind          text not null default 'adult' check (kind in ('adult', 'kids')),
  maturity      text not null default 'adult' check (maturity in ('little_kids', 'older_kids', 'teens', 'adult')),
  has_pin       boolean not null default false,
  is_main       boolean not null default false,
  auto_kids     boolean not null default false,
  sort_order    smallint not null default 0,
  guide_seen_at timestamptz,
  created_at    timestamptz not null default now(),
  updated_at    timestamptz not null default now(),
  constraint profiles_kids_maturity check (kind <> 'kids' or maturity in ('little_kids', 'older_kids')),
  constraint profiles_main_not_kids check (not (is_main and kind = 'kids'))
);
create unique index profiles_user_name_key on profiles (user_id, lower(btrim(name)));
create unique index profiles_one_main on profiles (user_id) where is_main;
create unique index profiles_one_auto_kids on profiles (user_id) where auto_kids;

create table profile_pins (
  profile_id      uuid primary key references profiles (id) on delete cascade,
  pin_hash        text not null,
  failed_attempts smallint not null default 0,
  lock_count      smallint not null default 0,
  locked_until    timestamptz,
  updated_at      timestamptz not null default now()
);

-- Written only by the step up command, apart from settings, so a queued write can never reach it.
create table profile_limits (
  profile_id    uuid primary key references profiles (id) on delete cascade,
  age_limit     text not null default '18' check (age_limit in ('U', 'PG', '12', '15', '18')),
  blocked_names jsonb not null default '[]'::jsonb
    check (jsonb_typeof(blocked_names) = 'array' and jsonb_array_length(blocked_names) <= 200),
  updated_at    timestamptz not null default now()
);

create table account_events (
  id      bigint generated always as identity primary key,
  user_id uuid not null references auth.users (id) on delete cascade,
  ts      timestamptz not null default now(),
  event   text not null,
  meta    jsonb not null default '{}'::jsonb
);
create index account_events_user_ts_idx on account_events (user_id, ts desc);

create table account_devices (
  id            uuid primary key default gen_random_uuid(),
  user_id       uuid not null references auth.users (id) on delete cascade,
  device_key    text not null check (char_length(device_key) between 16 and 64),
  browser       text check (char_length(browser) <= 60),
  browser_major integer,
  os            text check (char_length(os) <= 60),
  device_type   text not null default 'other' check (device_type in ('desktop', 'mobile', 'tablet', 'tv', 'other')),
  language      text check (char_length(language) <= 20),
  timezone      text check (char_length(timezone) <= 60),
  screen        text check (char_length(screen) <= 20),
  user_agent    text check (char_length(user_agent) <= 400),
  last_country  text check (last_country ~ '^[A-Z]{2}$'),
  countries     text[] not null default '{}',
  session_id    uuid,
  first_seen    timestamptz not null default now(),
  last_seen     timestamptz not null default now(),
  unique (user_id, device_key)
);
create index account_devices_user_idx on account_devices (user_id, last_seen desc);

-- One row per signed in device. Tokens are stored as hashes; the active profile, the last step up
-- and consent live here and nowhere else.
create table device_sessions (
  id                    uuid primary key,
  user_id               uuid not null references auth.users (id) on delete cascade,
  device_name           text not null default '' check (char_length(device_name) <= 120),
  access_hash           bytea not null unique,
  access_expires_at     timestamptz not null,
  refresh_current       bytea not null unique,
  refresh_next          bytea not null unique,
  refresh_salt          text not null,
  refresh_first_used_at timestamptz,
  active_profile        uuid references profiles (id) on delete set null,
  entered_at            timestamptz,
  step_up_at            timestamptz,
  step_up_method        text check (step_up_method in ('password', 'code')),
  consent_personal      boolean not null default false,
  card_hash             text,
  card                  jsonb,
  last_seen_at          timestamptz not null,
  created_at            timestamptz not null
);
create index device_sessions_user_idx on device_sessions (user_id, last_seen_at desc);

create table device_codes (
  device_hash bytea primary key,
  user_code   text not null unique,
  device_name text not null check (char_length(device_name) <= 120),
  user_id     uuid references auth.users (id) on delete cascade,
  denied      boolean not null default false,
  tries       smallint not null default 0,
  last_poll   timestamptz,
  expires_at  timestamptz not null
);
