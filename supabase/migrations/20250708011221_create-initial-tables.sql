CREATE DOMAIN "public"."solana_pubkey" AS character varying(44);

ALTER DOMAIN "public"."solana_pubkey" OWNER TO "postgres";

CREATE DOMAIN "public"."u_64" AS numeric(20, 0);

ALTER DOMAIN "public"."u_64" OWNER TO "postgres";

create table "public"."epoch_rewards" (
    "id" character varying(70) not null,
    "vote_pubkey" solana_pubkey not null,
    "epoch" u_64 not null,
    "inflation_commission_bps" smallint not null default 0,
    "total_inflation_rewards" u_64 not null default 0,
    "mev_commission_bps" smallint not null default 0,
    "total_mev_rewards" u_64 not null default 0,
    "priority_fee_commission_bps" smallint not null default 0,
    "total_priority_fee_rewards" u_64 not null default 0,
    "active_stake" u_64
);


alter table "public"."epoch_rewards" enable row level security;

create table "public"."validator_history_entries" (
    "id" character varying(70) not null,
    "vote_pubkey" solana_pubkey not null,
    "activated_stake_lamports" u_64 not null,
    "epoch" integer not null,
    "mev_commission" integer not null,
    "epoch_credits" bigint not null,
    "commission" integer not null,
    "client_type" smallint not null,
    "version" jsonb not null,
    "ip" character varying(256),
    "merkle_root_upload_authority" smallint not null default 0,
    "is_superminority" smallint not null,
    "rank" bigint not null,
    "vote_account_last_update_slot" u_64 not null,
    "mev_earned" bigint not null,
    "priority_fee_commission" integer,
    "priority_fee_tips" u_64,
    "total_priority_fees" u_64,
    "total_leader_slots" bigint,
    "blocks_produced" bigint,
    "block_data_updated_at_slot" u_64
);


alter table "public"."validator_history_entries" enable row level security;

CREATE UNIQUE INDEX epoch_rewards_pkey ON public.epoch_rewards USING btree (id);

CREATE UNIQUE INDEX validator_history_entries_pkey ON public.validator_history_entries USING btree (id);

alter table "public"."epoch_rewards" add constraint "epoch_rewards_pkey" PRIMARY KEY using index "epoch_rewards_pkey";

alter table "public"."validator_history_entries" add constraint "validator_history_entries_pkey" PRIMARY KEY using index "validator_history_entries_pkey";

grant delete on table "public"."epoch_rewards" to "anon";

grant insert on table "public"."epoch_rewards" to "anon";

grant references on table "public"."epoch_rewards" to "anon";

grant select on table "public"."epoch_rewards" to "anon";

grant trigger on table "public"."epoch_rewards" to "anon";

grant truncate on table "public"."epoch_rewards" to "anon";

grant update on table "public"."epoch_rewards" to "anon";

grant delete on table "public"."epoch_rewards" to "authenticated";

grant insert on table "public"."epoch_rewards" to "authenticated";

grant references on table "public"."epoch_rewards" to "authenticated";

grant select on table "public"."epoch_rewards" to "authenticated";

grant trigger on table "public"."epoch_rewards" to "authenticated";

grant truncate on table "public"."epoch_rewards" to "authenticated";

grant update on table "public"."epoch_rewards" to "authenticated";

grant delete on table "public"."epoch_rewards" to "service_role";

grant insert on table "public"."epoch_rewards" to "service_role";

grant references on table "public"."epoch_rewards" to "service_role";

grant select on table "public"."epoch_rewards" to "service_role";

grant trigger on table "public"."epoch_rewards" to "service_role";

grant truncate on table "public"."epoch_rewards" to "service_role";

grant update on table "public"."epoch_rewards" to "service_role";

grant delete on table "public"."validator_history_entries" to "anon";

grant insert on table "public"."validator_history_entries" to "anon";

grant references on table "public"."validator_history_entries" to "anon";

grant select on table "public"."validator_history_entries" to "anon";

grant trigger on table "public"."validator_history_entries" to "anon";

grant truncate on table "public"."validator_history_entries" to "anon";

grant update on table "public"."validator_history_entries" to "anon";

grant delete on table "public"."validator_history_entries" to "authenticated";

grant insert on table "public"."validator_history_entries" to "authenticated";

grant references on table "public"."validator_history_entries" to "authenticated";

grant select on table "public"."validator_history_entries" to "authenticated";

grant trigger on table "public"."validator_history_entries" to "authenticated";

grant truncate on table "public"."validator_history_entries" to "authenticated";

grant update on table "public"."validator_history_entries" to "authenticated";

grant delete on table "public"."validator_history_entries" to "service_role";

grant insert on table "public"."validator_history_entries" to "service_role";

grant references on table "public"."validator_history_entries" to "service_role";

grant select on table "public"."validator_history_entries" to "service_role";

grant trigger on table "public"."validator_history_entries" to "service_role";

grant truncate on table "public"."validator_history_entries" to "service_role";

grant update on table "public"."validator_history_entries" to "service_role";

create policy "Enable read access for all users"
on "public"."epoch_rewards"
as permissive
for select
to public
using (true);


create policy "Enable read access for all users"
on "public"."validator_history_entries"
as permissive
for select
to public
using (true);



