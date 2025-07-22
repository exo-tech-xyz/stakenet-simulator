create table "public"."cluster_history_entries" (
    "epoch" integer not null,
    "total_blocks" bigint not null,
    "epoch_start_timestamp" u_64 not null
);


alter table "public"."cluster_history_entries" enable row level security;

CREATE UNIQUE INDEX cluster_history_entries_pkey ON public.cluster_history_entries USING btree (epoch);

alter table "public"."cluster_history_entries" add constraint "cluster_history_entries_pkey" PRIMARY KEY using index "cluster_history_entries_pkey";

grant delete on table "public"."cluster_history_entries" to "anon";

grant insert on table "public"."cluster_history_entries" to "anon";

grant references on table "public"."cluster_history_entries" to "anon";

grant select on table "public"."cluster_history_entries" to "anon";

grant trigger on table "public"."cluster_history_entries" to "anon";

grant truncate on table "public"."cluster_history_entries" to "anon";

grant update on table "public"."cluster_history_entries" to "anon";

grant delete on table "public"."cluster_history_entries" to "authenticated";

grant insert on table "public"."cluster_history_entries" to "authenticated";

grant references on table "public"."cluster_history_entries" to "authenticated";

grant select on table "public"."cluster_history_entries" to "authenticated";

grant trigger on table "public"."cluster_history_entries" to "authenticated";

grant truncate on table "public"."cluster_history_entries" to "authenticated";

grant update on table "public"."cluster_history_entries" to "authenticated";

grant delete on table "public"."cluster_history_entries" to "service_role";

grant insert on table "public"."cluster_history_entries" to "service_role";

grant references on table "public"."cluster_history_entries" to "service_role";

grant select on table "public"."cluster_history_entries" to "service_role";

grant trigger on table "public"."cluster_history_entries" to "service_role";

grant truncate on table "public"."cluster_history_entries" to "service_role";

grant update on table "public"."cluster_history_entries" to "service_role";

create policy "Enable read access for all users"
on "public"."cluster_history_entries"
as permissive
for select
to public
using (true);



