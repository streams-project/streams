-- What play and devices report, by client class and host.

create table reachability (
  class      text not null,
  host       text not null,
  tries      integer not null default 0 check (tries >= 0),
  starts     integer not null default 0 check (starts >= 0),
  updated_at timestamptz not null default now(),
  primary key (class, host)
);

create table play_events (
  id         bigint generated always as identity primary key,
  profile_id uuid references profiles (id) on delete set null,
  ts         timestamptz not null default now(),
  body       jsonb not null
);
create index play_events_ts_idx on play_events (ts);

create table client_reports (
  id    bigint generated always as identity primary key,
  ts    timestamptz not null default now(),
  build text not null check (char_length(build) <= 80),
  code  text not null check (char_length(code) <= 80),
  body  jsonb
);
create index client_reports_ts_idx on client_reports (ts);
