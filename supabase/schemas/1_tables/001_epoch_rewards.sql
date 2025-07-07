--
-- Epoch Rewards Table
--
-- This table tracks all rewards for a given vote_pubkey in a given epoch
--
CREATE TABLE IF NOT EXISTS "public"."epoch_rewards"(
    "id" VARCHAR(70) NOT NULL PRIMARY KEY, -- {epoch}-{vote_pubkey}
    "vote_pubkey" "public"."solana_pubkey" NOT NULL,
    "epoch" "public"."u_64" NOT NULL,
    "inflation_commission_bps" SMALLINT NOT NULL DEFAULT 0,
    "total_inflation_rewards" "public"."u_64" NOT NULL DEFAULT 0, -- total inflation across the entire validator
    "mev_commission_bps" SMALLINT NOT NULL DEFAULT 0,
    "total_mev_rewards" "public"."u_64" NOT NULL DEFAULT 0, -- Total MEV tips across the entire validator
    "priority_fee_commission_bps" SMALLINT NOT NULL DEFAULT 0,
    "total_priority_fee_rewards" "public"."u_64" NOT NULL DEFAULT 0, -- Total Priority Fees across the entire validator
    "active_stake" "public"."u_64"
);

--
-- Row Level Security Policies
--
ALTER TABLE "public"."epoch_rewards" ENABLE ROW LEVEL SECURITY;

-- Policy: Enable read access for all users
CREATE POLICY "Enable read access for all users" ON "public"."epoch_rewards"
    FOR SELECT
        USING (TRUE);