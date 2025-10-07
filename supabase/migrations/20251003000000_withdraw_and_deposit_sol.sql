CREATE TABLE
    IF NOT EXISTS "public"."withdraw_and_deposit_sol" (
        "epoch" "public"."u_64" NOT NULL PRIMARY KEY,
        "withdraw_sol" NUMERIC(20, 9) DEFAULT 0,
        "deposit_sol" NUMERIC(20, 9) DEFAULT 0
    );


-- Enable RLS
ALTER TABLE public.withdraw_and_deposit_sol ENABLE ROW LEVEL SECURITY;

-- Grants: anon
GRANT DELETE,
INSERT,
REFERENCES,
SELECT
,
    TRIGGER,
    TRUNCATE,
UPDATE ON public.withdraw_and_deposit_sol TO anon;

-- Grants: authenticated
GRANT DELETE,
INSERT,
REFERENCES,
SELECT
,
    TRIGGER,
    TRUNCATE,
UPDATE ON public.withdraw_and_deposit_sol TO authenticated;

-- Grants: service_role
GRANT DELETE,
INSERT,
REFERENCES,
SELECT
,
    TRIGGER,
    TRUNCATE,
UPDATE ON public.withdraw_and_deposit_sol TO service_role;

-- Policy: Enable read access for all users
CREATE POLICY "Enable read access for all users" ON public.withdraw_and_deposit_sol AS PERMISSIVE FOR
SELECT
    TO public USING (TRUE);