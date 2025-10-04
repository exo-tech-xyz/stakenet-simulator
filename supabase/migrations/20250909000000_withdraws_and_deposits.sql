--
-- Withdraws And Deposits Table
-- This table stores transactions of withdraws and deposits
--
CREATE TABLE
    IF NOT EXISTS public.withdraw_and_deposit_stakes (
        "id" VARCHAR(70) NOT NULL PRIMARY KEY, -- {epoch}-{vote_pubkey}
        epoch public.u_64 NOT NULL,
        "vote_pubkey" "public"."solana_pubkey",
        withdraw_stake NUMERIC(20, 9) DEFAULT 0,
        deposit_stake NUMERIC(20, 9) DEFAULT 0
    );

-- Enable RLS
ALTER TABLE public.withdraw_and_deposit_stakes ENABLE ROW LEVEL SECURITY;

-- Grants: anon
GRANT DELETE,
INSERT,
REFERENCES,
SELECT
,
    TRIGGER,
    TRUNCATE,
UPDATE ON public.withdraw_and_deposit_stakes TO anon;

-- Grants: authenticated
GRANT DELETE,
INSERT,
REFERENCES,
SELECT
,
    TRIGGER,
    TRUNCATE,
UPDATE ON public.withdraw_and_deposit_stakes TO authenticated;

-- Grants: service_role
GRANT DELETE,
INSERT,
REFERENCES,
SELECT
,
    TRIGGER,
    TRUNCATE,
UPDATE ON public.withdraw_and_deposit_stakes TO service_role;

-- Policy: Enable read access for all users
CREATE POLICY "Enable read access for all users" ON public.withdraw_and_deposit_stakes AS PERMISSIVE FOR
SELECT
    TO public USING (TRUE);