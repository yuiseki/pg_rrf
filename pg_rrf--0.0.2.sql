/*
This file defines extension objects for pg_rrf v0.0.2.
*/

-- rrf
CREATE FUNCTION "rrf"(
    "rank_a" bigint,
    "rank_b" bigint,
    "k" bigint
) RETURNS double precision
LANGUAGE c
AS 'MODULE_PATHNAME', 'rrf_wrapper';

-- rrf3
CREATE FUNCTION "rrf3"(
    "rank_a" bigint,
    "rank_b" bigint,
    "rank_c" bigint,
    "k" bigint
) RETURNS double precision
LANGUAGE c
AS 'MODULE_PATHNAME', 'rrf3_wrapper';

-- rrf_fuse (SRF)
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
