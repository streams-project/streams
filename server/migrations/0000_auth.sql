-- Sign in, built into the server. The table keeps the shape and bcrypt hashes of the auth service
-- it replaces, so an existing `auth.users` carries over as it is and every password keeps working.
create schema if not exists auth;

create table if not exists auth.users (
  id                 uuid primary key default gen_random_uuid(),
  email              text,
  phone              text,
  encrypted_password text
);

create unique index if not exists users_email_built_in on auth.users (lower(email));

-- One live code per address and purpose. Only a hash is kept.
create table auth.login_codes (
  email      text        not null,
  purpose    text        not null check (purpose in ('sign_in', 'recovery')),
  code_hash  text        not null,
  expires_at timestamptz not null,
  tries      integer     not null default 0,
  primary key (email, purpose)
);
