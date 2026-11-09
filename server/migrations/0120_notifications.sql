-- Reminders on coming titles and notification read marks per profile.

create table profile_reminders (
  profile_id        uuid not null references profiles (id) on delete cascade,
  kind              text not null check (kind in ('movie', 'tv')),
  title_id          integer not null check (title_id > 0),
  remind            boolean not null default false,
  remind_version    bigint not null default 0,
  remind_changed_at timestamptz,
  seq               bigint not null,
  primary key (profile_id, kind, title_id)
);
create index profile_reminders_idx on profile_reminders (profile_id, remind_changed_at desc) where remind;
create index profile_reminders_seq_idx on profile_reminders (profile_id, seq);

create table notification_reads (
  profile_id uuid primary key references profiles (id) on delete cascade,
  read_up_to timestamptz not null default now(),
  dismissed  jsonb not null default '[]'::jsonb check (jsonb_typeof(dismissed) = 'array')
);
