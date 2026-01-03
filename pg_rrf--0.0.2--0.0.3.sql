/*
Upgrade script from pg_rrf v0.0.2 to v0.0.3.
*/

-- Add rrfn
CREATE FUNCTION "rrfn"(
    "ranks" bigint[],
    "k" bigint
) RETURNS double precision
LANGUAGE c
AS 'MODULE_PATHNAME', 'rrfn_wrapper';
