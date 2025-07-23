--
-- Validator History Table
--
-- Stores the top level data of the ValidatorHistory data structure
--
CREATE TABLE IF NOT EXISTS "public"."validator_histories"(
    "vote_account" "public"."solana_pubkey" NOT NULL PRIMARY KEY,
    "struct_version" BIGINT NOT NULL,
    "index" BIGINT NOT NULL,
    "bump" SMALLINT NOT NULL,
    "last_ip_timestamp" "public"."u_64" NOT NULL,
    "last_version_timestamp" "public"."u_64" NOT NULL
);

--
-- Row Level Security Policies
--
ALTER TABLE "public"."validator_histories" ENABLE ROW LEVEL SECURITY;

-- Policy: Enable read access for all users
CREATE POLICY "Enable read access for all users" ON "public"."validator_histories"
    FOR SELECT
        USING (TRUE);