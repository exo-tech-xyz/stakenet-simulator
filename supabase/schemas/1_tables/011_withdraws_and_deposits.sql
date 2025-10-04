--
-- Withdraws And Deposits Table
--
-- This table stores transactions of withdraws and deposits 
--
CREATE TABLE
    IF NOT EXISTS "public"."withdraw_and_deposit_stakes" (
        "id" VARCHAR(70) NOT NULL PRIMARY KEY, -- {epoch}-{vote_pubkey}
        "epoch" "public"."u_64" NOT NULL,
        "vote_pubkey" "public"."solana_pubkey",
        "withdraw_stake" NUMERIC(20, 9) DEFAULT 0,
        "deposit_stake" NUMERIC(20, 9) DEFAULT 0
    );

--
-- Row Level Security Policies
--
ALTER TABLE "public"."withdraw_and_deposit_stakes" ENABLE ROW LEVEL SECURITY;

-- Policy: Enable read access for all users
CREATE POLICY "Enable read access for all users" ON "public"."withdraw_and_deposit_stakes" FOR
SELECT
    USING (TRUE);