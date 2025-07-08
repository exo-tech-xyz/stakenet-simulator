--
-- Validator History Entry Table
--
-- Stores each validator history entry for a given vote_pubkey in a given epoch
--
CREATE TABLE IF NOT EXISTS "public"."validator_history_entries"(
    "id" VARCHAR(70) NOT NULL PRIMARY KEY, -- {epoch}-{vote_pubkey}
    "vote_pubkey" "public"."solana_pubkey" NOT NULL,
    "activated_stake_lamports" "public"."u_64" NOT NULL,
    "epoch" INTEGER NOT NULL,
    "mev_commission"  INTEGER NOT NULL,
    "epoch_credits" BIGINT NOT NULL,
    "commission" INTEGER NOT NULL,
    "client_type" SMALLINT NOT NULL,
    "version" JSONB NOT NULL,
    "ip" VARCHAR(256),
    "merkle_root_upload_authority" SMALLINT DEFAULT 0 NOT NULL,
    "is_superminority" SMALLINT NOT NULL,
    "rank" BIGINT NOT NULL,
    "vote_account_last_update_slot" "public"."u_64" NOT NULL,
    "mev_earned" BIGINT NOT NULL,
    "priority_fee_commission" INTEGER,
    "priority_fee_tips" "public"."u_64",
    "total_priority_fees" "public"."u_64",
    "total_leader_slots" BIGINT,
    "blocks_produced" BIGINT,
    "block_data_updated_at_slot" "public"."u_64"
);

--
-- Row Level Security Policies
--
ALTER TABLE "public"."validator_history_entries" ENABLE ROW LEVEL SECURITY;

-- Policy: Enable read access for all users
CREATE POLICY "Enable read access for all users" ON "public"."validator_history_entries"
    FOR SELECT
        USING (TRUE);