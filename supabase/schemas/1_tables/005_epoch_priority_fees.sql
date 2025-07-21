--
-- Epoch Priority Fees Table
--
-- This table tracks sum of a validator's priority fees for an epoch
--
CREATE TABLE IF NOT EXISTS "public"."epoch_priority_fees"(
    "id" VARCHAR(70) NOT NULL PRIMARY KEY, -- concatenation of {epoch}-{identity_pubkey}
    "identity_pubkey" "public"."solana_pubkey" NOT NULL,
    "epoch" "public"."u_64",
    "priority_fees" "public"."u_64"
);

ALTER TABLE "public"."epoch_priority_fees" OWNER TO "postgres";

-- INDEXES
CREATE INDEX "idx_epoch_priority_fees_by_identity_and_epoch" ON "public"."epoch_priority_fees" USING "btree"("identity_pubkey", "epoch");

--
-- Row Level Security Policies
--
ALTER TABLE "public"."epoch_priority_fees" ENABLE ROW LEVEL SECURITY;

-- Policy: Enable read access for all users
CREATE POLICY "Enable read access for all users" ON "public"."epoch_priority_fees"
    FOR SELECT
        USING (TRUE);
