--
-- Stake Accounts Table
--
-- Stores each a stake account for each validator
--
CREATE TABLE IF NOT EXISTS "public"."stake_accounts"(
    "pubkey" "public"."solana_pubkey" NOT NULL PRIMARY KEY, 
    "discriminator" INTEGER NOT NULL DEFAULT 0, 
    "rent_exempt_reserve" "public"."u_64",
    "authorized_staker" "public"."solana_pubkey",
    "authorized_withdrawer" "public"."solana_pubkey",
    "lockup_unix_timestamp" BIGINT,
    "lockup_epoch" "public"."u_64",
    "lockup_custodian" "public"."solana_pubkey",
    "delegation_voter_pubkey" "public"."solana_pubkey",
    "delegation_stake" "public"."u_64",
    "delegation_activation_epoch" "public"."u_64",
    "delegation_deactivation_epoch" "public"."u_64",
    "delegation_warmup_cooldown_rate" DOUBLE PRECISION,
    "credits_observed" "public"."u_64"

);


CREATE INDEX "idx_stake_accounts_on_vote_pubkey" ON "public"."stake_accounts" USING "btree"("delegation_voter_pubkey");

--
-- Row Level Security Policies
--
ALTER TABLE "public"."stake_accounts" ENABLE ROW LEVEL SECURITY;

-- Policy: Enable read access for all users
CREATE POLICY "Enable read access for all users" ON "public"."stake_accounts"
    FOR SELECT
        USING (TRUE);