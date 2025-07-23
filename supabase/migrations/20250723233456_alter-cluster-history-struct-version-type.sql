alter table "public"."cluster_histories" alter column "struct_version" set data type u_64 using "struct_version"::u_64;


