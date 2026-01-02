PostgreSQL拡張でRRF/スコア融合を関数化（SQLがきれいになるやつ）

ねらい：いま手元のハイブリッド検索を「実験」から「道具」に昇格

MVP：rrf(rank_a, rank_b, k) / rrf3(...) を拡張関数で提供

発展：距離減衰（PostGIS）＋全文＋ベクトルの3者融合を“同じ器”に

実装手段：Rust の pgrx（Postgres拡張のRustフレームワーク）