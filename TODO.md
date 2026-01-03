# TODO (pg_rrf)

## 現状（2026-01-03）
- v0.0.1 相当: `rrf` / `rrf3` 実装済み、NULL/非正 rank/k の扱いは仕様通り
- Docker 経由の `make build` / `make test` は通過済み
- README は v0.0.1 相当の説明のみ

## v0.0.2: SRF `rrf_fuse`（2チャネル融合）
- `rrf_fuse(ids_a bigint[], ids_b bigint[], k int default 60)` を実装
- 内部の共通スコア計算ヘルパを用意（既存 `rrf_sum` の整理・再利用）
- 仕様準拠の挙動確認（NULL 配列は空扱い、重複 ID は最良 rank を採用）
- 回帰テスト追加（重複、片側 NULL/空、k<=0 エラー、rrf との整合）
- 拡張 SQL ファイルとアップグレードスクリプト作成
  - `pg_rrf--0.0.2.sql`
  - `pg_rrf--0.0.1--0.0.2.sql`
- README に SRF の説明と SQL 例を追記
- バージョン方針の確認（SPEC は 0.0.x、現状 Cargo.toml は 0.1.0）

## v0.0.3: 汎用スコア関数 `rrfn`
- `rrfn(ranks int[], k int)` を実装（空配列は 0.0）
- 可能なら `rrfn(k int, VARIADIC ranks int[])` を追加
- `rrf` / `rrf3` を `rrfn` の薄いラッパーに置き換え
- 回帰テスト追加（NULL/<=0 無視、rrf/rrf3 と一致、k<=0 エラー）
- 拡張 SQL ファイルとアップグレードスクリプト作成
  - `pg_rrf--0.0.3.sql`
  - `pg_rrf--0.0.2--0.0.3.sql`
- README に `rrfn` の説明と例を追記

## 追加検討（任意）
- CI 追加（docker compose + make test）
- `uuid[]` 等の ID 型対応（v0.0.2.1 以降）
- N チャネル SRF の設計（v0.0.4 以降）
