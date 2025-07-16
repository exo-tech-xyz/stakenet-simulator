create table "public"."stake_accounts" (
    "pubkey" solana_pubkey not null,
    "discriminator" integer not null default 0,
    "rent_exempt_reserve" u_64,
    "authorized_staker" solana_pubkey,
    "authorized_withdrawer" solana_pubkey,
    "lockup_unix_timestamp" bigint,
    "lockup_epoch" u_64,
    "lockup_custodian" solana_pubkey,
    "delegation_voter_pubkey" solana_pubkey,
    "delegation_stake" u_64,
    "delegation_activation_epoch" u_64,
    "delegation_deactivation_epoch" u_64,
    "delegation_warmup_cooldown_rate" double precision,
    "credits_observed" u_64
);


alter table "public"."stake_accounts" enable row level security;

CREATE INDEX idx_stake_accounts_on_vote_pubkey ON public.stake_accounts USING btree (delegation_voter_pubkey);

CREATE UNIQUE INDEX stake_accounts_pkey ON public.stake_accounts USING btree (pubkey);

alter table "public"."stake_accounts" add constraint "stake_accounts_pkey" PRIMARY KEY using index "stake_accounts_pkey";

grant delete on table "public"."stake_accounts" to "anon";

grant insert on table "public"."stake_accounts" to "anon";

grant references on table "public"."stake_accounts" to "anon";

grant select on table "public"."stake_accounts" to "anon";

grant trigger on table "public"."stake_accounts" to "anon";

grant truncate on table "public"."stake_accounts" to "anon";

grant update on table "public"."stake_accounts" to "anon";

grant delete on table "public"."stake_accounts" to "authenticated";

grant insert on table "public"."stake_accounts" to "authenticated";

grant references on table "public"."stake_accounts" to "authenticated";

grant select on table "public"."stake_accounts" to "authenticated";

grant trigger on table "public"."stake_accounts" to "authenticated";

grant truncate on table "public"."stake_accounts" to "authenticated";

grant update on table "public"."stake_accounts" to "authenticated";

grant delete on table "public"."stake_accounts" to "service_role";

grant insert on table "public"."stake_accounts" to "service_role";

grant references on table "public"."stake_accounts" to "service_role";

grant select on table "public"."stake_accounts" to "service_role";

grant trigger on table "public"."stake_accounts" to "service_role";

grant truncate on table "public"."stake_accounts" to "service_role";

grant update on table "public"."stake_accounts" to "service_role";

create policy "Enable read access for all users"
on "public"."stake_accounts"
as permissive
for select
to public
using (true);



