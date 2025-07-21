--
-- Inflation Rewards Table
--
-- Stores inflation rewards received by a stake account for a given epoch. It should mirror data
-- returned from getInflationReward RPC method
--
CREATE TABLE IF NOT EXISTS "public"."inflation_rewards"(
    "id" VARCHAR(70) NOT NULL PRIMARY KEY, -- {epoch}-{stake_account}
    "stake_account" "public"."solana_pubkey" NOT NULL, 
    "epoch" "public"."u_64" NOT NULL,
    "effective_slot" "public"."u_64" NOT NULL,
    "amount" "public"."u_64" NOT NULL,
    "post_balance" "public"."u_64" NOT NULL,
    "commission" SMALLINT
);

ALTER TABLE "public"."inflation_rewards" ADD CONSTRAINT "stake_account_fk" FOREIGN KEY ("stake_account") REFERENCES "public"."stake_accounts"("pubkey");

--
-- Row Level Security Policies
--
ALTER TABLE "public"."inflation_rewards" ENABLE ROW LEVEL SECURITY;

-- Policy: Enable read access for all users
CREATE POLICY "Enable read access for all users" ON "public"."inflation_rewards"
    FOR SELECT
        USING (TRUE);