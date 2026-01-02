# Agent Memo: RRF Extension Plan (更新)

## 動機（forks/study-pg-hybrid-search より）
- RRF を Python 側の実装から DB 内関数へ寄せ、SQL を簡潔化・再現性を向上する。
- レポートで「RRF 重み最適化」「テキスト側改善と RRF 寄与測定」が次アクションとして挙がっており、関数化が実験の効率を上げる。

## 実装状況（2026-01-02）
- Rust + pgrx で拡張の雛形を作成済み。
- `rrf(rank_a, rank_b, k)` と `rrf3(rank_a, rank_b, rank_c, k)` を実装済み。
- NULL 安全: 片方 NULL の場合はもう片方のみ、全 NULL は NULL。
- `rank <= 0` はスキップ、`k <= 0` はエラー。
- Docker でビルド・インストール・動作確認まで通過。

## 確認済みの動作
- Docker 内で `CREATE EXTENSION pg_rrf;` 実行成功。
- `SELECT rrf(1,2,60), rrf3(1,2,3,60);` で期待通りの数値を返すことを確認。
- `make test` が Docker 内で実行され、5 テストすべてパス。

## Docker/Makefile 方針
- ローカル環境は信用せず、`make build` / `make test` は必ず docker compose 環境で実行。
- `dev` サービスでカレントディレクトリを `/workspace` にボリュームマウント。
- `make build` で `build/` に `.so/.control/.sql` を収集。

## 作成/更新ファイル
- `Cargo.toml` (pgrx 0.16.1 / pg17)
- `src/lib.rs` (rrf/rrf3 実装 + tests + pg_test module)
- `pg_rrf.control`
- `src/bin/pgrx_embed.rs`
- `.cargo/config.toml`
- `Dockerfile` (sudo 追加)
- `docker-compose.yml` (db + dev サービス、ポート 5436)
- `Makefile` (docker compose 経由の build/test)
- `.tool-versions` (rust stable)

## 仕様（MVP）
- 関数:
  - `rrf(rank_a, rank_b, k) -> float8`
  - `rrf3(rank_a, rank_b, rank_c, k) -> float8`
- `rank` は 1-based 順位。
- NULL 安全: 片方 NULL の場合はもう片方のみ、全 NULL は NULL。
- `rank <= 0` はスキップ。
- `k <= 0` はエラー。

## 次の候補タスク
1. README か `sql/` に利用例を追加。
2. 仕様の確定（NULL/不正値/重み付け/距離減衰の扱い）。
3. CI の追加（docker compose + make test）。
