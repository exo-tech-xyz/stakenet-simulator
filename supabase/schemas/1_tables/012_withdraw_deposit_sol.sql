--
-- Withdraws And Deposits Sol Table
--
-- This table stores epoch wise sol withdraw and deposits 
--
CREATE TABLE
    IF NOT EXISTS "public"."withdraw_and_deposit_sol" (
        "epoch" "public"."u_64" NOT NULL PRIMARY KEY,
        "withdraw_sol" NUMERIC(20, 9) DEFAULT 0,
        "deposit_sol" NUMERIC(20, 9) DEFAULT 0
    );

--
-- Row Level Security Policies
--
ALTER TABLE "public"."withdraw_and_deposit_sol" ENABLE ROW LEVEL SECURITY;

-- Policy: Enable read access for all users
CREATE POLICY "Enable read access for all users" ON "public"."withdraw_and_deposit_sol" FOR
SELECT
    USING (TRUE);