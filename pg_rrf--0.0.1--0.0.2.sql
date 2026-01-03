/*
Upgrade script from pg_rrf v0.0.1 to v0.0.2.
*/

-- Add rrf_fuse (SRF)
CREATE FUNCTION "rrf_fuse"(
    "ids_a" bigint[],
    "ids_b" bigint[],
    "k" bigint DEFAULT 60
) RETURNS TABLE (
    "id" bigint,
    "score" double precision,
    "rank_a" integer,
    "rank_b" integer
)
LANGUAGE c
AS 'MODULE_PATHNAME', 'rrf_fuse_wrapper';
