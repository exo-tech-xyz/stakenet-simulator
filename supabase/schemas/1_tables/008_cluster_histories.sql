--
-- Cluster History Table
--
-- Stores the top level data of the ClusterHistory data structure
--
CREATE TABLE IF NOT EXISTS "public"."cluster_histories"(
    "id" INTEGER NOT NULL PRIMARY KEY,
    "struct_version" "public"."u_64" NOT NULL,
    "bump" SMALLINT NOT NULL,
    "cluster_history_last_update_slot" "public"."u_64" NOT NULL
);

--
-- Row Level Security Policies
--
ALTER TABLE "public"."cluster_histories" ENABLE ROW LEVEL SECURITY;

-- Policy: Enable read access for all users
CREATE POLICY "Enable read access for all users" ON "public"."cluster_histories"
    FOR SELECT
        USING (TRUE);