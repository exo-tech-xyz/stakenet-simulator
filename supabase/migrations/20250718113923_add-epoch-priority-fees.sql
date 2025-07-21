create table "public"."epoch_priority_fees" (
    "id" character varying(70) not null,
    "identity_pubkey" solana_pubkey not null,
    "epoch" u_64,
    "priority_fees" u_64
);


alter table "public"."epoch_priority_fees" enable row level security;

CREATE UNIQUE INDEX epoch_priority_fees_pkey ON public.epoch_priority_fees USING btree (id);

CREATE INDEX idx_epoch_priority_fees_by_identity_and_epoch ON public.epoch_priority_fees USING btree (identity_pubkey, epoch);

alter table "public"."epoch_priority_fees" add constraint "epoch_priority_fees_pkey" PRIMARY KEY using index "epoch_priority_fees_pkey";

grant delete on table "public"."epoch_priority_fees" to "anon";

grant insert on table "public"."epoch_priority_fees" to "anon";

grant references on table "public"."epoch_priority_fees" to "anon";

grant select on table "public"."epoch_priority_fees" to "anon";

grant trigger on table "public"."epoch_priority_fees" to "anon";

grant truncate on table "public"."epoch_priority_fees" to "anon";

grant update on table "public"."epoch_priority_fees" to "anon";

grant delete on table "public"."epoch_priority_fees" to "authenticated";

grant insert on table "public"."epoch_priority_fees" to "authenticated";

grant references on table "public"."epoch_priority_fees" to "authenticated";

grant select on table "public"."epoch_priority_fees" to "authenticated";

grant trigger on table "public"."epoch_priority_fees" to "authenticated";

grant truncate on table "public"."epoch_priority_fees" to "authenticated";

grant update on table "public"."epoch_priority_fees" to "authenticated";

grant delete on table "public"."epoch_priority_fees" to "service_role";

grant insert on table "public"."epoch_priority_fees" to "service_role";

grant references on table "public"."epoch_priority_fees" to "service_role";

grant select on table "public"."epoch_priority_fees" to "service_role";

grant trigger on table "public"."epoch_priority_fees" to "service_role";

grant truncate on table "public"."epoch_priority_fees" to "service_role";

grant update on table "public"."epoch_priority_fees" to "service_role";

create policy "Enable read access for all users"
on "public"."epoch_priority_fees"
as permissive
for select
to public
using (true);



