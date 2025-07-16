create table "public"."inflation_rewards" (
    "id" character varying(70) not null,
    "stake_account" solana_pubkey not null,
    "epoch" u_64 not null,
    "effective_slot" u_64 not null,
    "amount" u_64 not null,
    "post_balance" u_64 not null,
    "commission" smallint
);


alter table "public"."inflation_rewards" enable row level security;

CREATE UNIQUE INDEX inflation_rewards_pkey ON public.inflation_rewards USING btree (id);

alter table "public"."inflation_rewards" add constraint "inflation_rewards_pkey" PRIMARY KEY using index "inflation_rewards_pkey";

alter table "public"."inflation_rewards" add constraint "stake_account_fk" FOREIGN KEY (stake_account) REFERENCES stake_accounts(pubkey) not valid;

alter table "public"."inflation_rewards" validate constraint "stake_account_fk";

grant delete on table "public"."inflation_rewards" to "anon";

grant insert on table "public"."inflation_rewards" to "anon";

grant references on table "public"."inflation_rewards" to "anon";

grant select on table "public"."inflation_rewards" to "anon";

grant trigger on table "public"."inflation_rewards" to "anon";

grant truncate on table "public"."inflation_rewards" to "anon";

grant update on table "public"."inflation_rewards" to "anon";

grant delete on table "public"."inflation_rewards" to "authenticated";

grant insert on table "public"."inflation_rewards" to "authenticated";

grant references on table "public"."inflation_rewards" to "authenticated";

grant select on table "public"."inflation_rewards" to "authenticated";

grant trigger on table "public"."inflation_rewards" to "authenticated";

grant truncate on table "public"."inflation_rewards" to "authenticated";

grant update on table "public"."inflation_rewards" to "authenticated";

grant delete on table "public"."inflation_rewards" to "service_role";

grant insert on table "public"."inflation_rewards" to "service_role";

grant references on table "public"."inflation_rewards" to "service_role";

grant select on table "public"."inflation_rewards" to "service_role";

grant trigger on table "public"."inflation_rewards" to "service_role";

grant truncate on table "public"."inflation_rewards" to "service_role";

grant update on table "public"."inflation_rewards" to "service_role";

create policy "Enable read access for all users"
on "public"."inflation_rewards"
as permissive
for select
to public
using (true);



