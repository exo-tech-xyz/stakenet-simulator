--
-- Cluster History Entry Table
--
-- Stores cluster level metrics per epoch
--
CREATE TABLE IF NOT EXISTS "public"."cluster_history_entries"(
    "epoch" INTEGER NOT NULL PRIMARY KEY,
    "total_blocks" BIGINT NOT NULL,
    "epoch_start_timestamp" "public"."u_64" NOT NULL
);

--
-- Row Level Security Policies
--
ALTER TABLE "public"."cluster_history_entries" ENABLE ROW LEVEL SECURITY;

-- Policy: Enable read access for all users
CREATE POLICY "Enable read access for all users" ON "public"."cluster_history_entries"
    FOR SELECT
        USING (TRUE);