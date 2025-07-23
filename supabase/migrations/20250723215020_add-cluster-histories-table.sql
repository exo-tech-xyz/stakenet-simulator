create table "public"."cluster_histories" (
    "id" integer not null,
    "struct_version" bigint not null,
    "bump" smallint not null,
    "cluster_history_last_update_slot" u_64 not null
);


alter table "public"."cluster_histories" enable row level security;

CREATE UNIQUE INDEX cluster_histories_pkey ON public.cluster_histories USING btree (id);

alter table "public"."cluster_histories" add constraint "cluster_histories_pkey" PRIMARY KEY using index "cluster_histories_pkey";

grant delete on table "public"."cluster_histories" to "anon";

grant insert on table "public"."cluster_histories" to "anon";

grant references on table "public"."cluster_histories" to "anon";

grant select on table "public"."cluster_histories" to "anon";

grant trigger on table "public"."cluster_histories" to "anon";

grant truncate on table "public"."cluster_histories" to "anon";

grant update on table "public"."cluster_histories" to "anon";

grant delete on table "public"."cluster_histories" to "authenticated";

grant insert on table "public"."cluster_histories" to "authenticated";

grant references on table "public"."cluster_histories" to "authenticated";

grant select on table "public"."cluster_histories" to "authenticated";

grant trigger on table "public"."cluster_histories" to "authenticated";

grant truncate on table "public"."cluster_histories" to "authenticated";

grant update on table "public"."cluster_histories" to "authenticated";

grant delete on table "public"."cluster_histories" to "service_role";

grant insert on table "public"."cluster_histories" to "service_role";

grant references on table "public"."cluster_histories" to "service_role";

grant select on table "public"."cluster_histories" to "service_role";

grant trigger on table "public"."cluster_histories" to "service_role";

grant truncate on table "public"."cluster_histories" to "service_role";

grant update on table "public"."cluster_histories" to "service_role";

create policy "Enable read access for all users"
on "public"."cluster_histories"
as permissive
for select
to public
using (true);



