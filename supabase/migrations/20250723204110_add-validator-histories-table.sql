create table "public"."validator_histories" (
    "vote_account" solana_pubkey not null,
    "struct_version" bigint not null,
    "index" bigint not null,
    "bump" smallint not null,
    "last_ip_timestamp" u_64 not null,
    "last_version_timestamp" u_64 not null
);


alter table "public"."validator_histories" enable row level security;

CREATE UNIQUE INDEX validator_histories_pkey ON public.validator_histories USING btree (vote_account);

alter table "public"."validator_histories" add constraint "validator_histories_pkey" PRIMARY KEY using index "validator_histories_pkey";

grant delete on table "public"."validator_histories" to "anon";

grant insert on table "public"."validator_histories" to "anon";

grant references on table "public"."validator_histories" to "anon";

grant select on table "public"."validator_histories" to "anon";

grant trigger on table "public"."validator_histories" to "anon";

grant truncate on table "public"."validator_histories" to "anon";

grant update on table "public"."validator_histories" to "anon";

grant delete on table "public"."validator_histories" to "authenticated";

grant insert on table "public"."validator_histories" to "authenticated";

grant references on table "public"."validator_histories" to "authenticated";

grant select on table "public"."validator_histories" to "authenticated";

grant trigger on table "public"."validator_histories" to "authenticated";

grant truncate on table "public"."validator_histories" to "authenticated";

grant update on table "public"."validator_histories" to "authenticated";

grant delete on table "public"."validator_histories" to "service_role";

grant insert on table "public"."validator_histories" to "service_role";

grant references on table "public"."validator_histories" to "service_role";

grant select on table "public"."validator_histories" to "service_role";

grant trigger on table "public"."validator_histories" to "service_role";

grant truncate on table "public"."validator_histories" to "service_role";

grant update on table "public"."validator_histories" to "service_role";

create policy "Enable read access for all users"
on "public"."validator_histories"
as permissive
for select
to public
using (true);



