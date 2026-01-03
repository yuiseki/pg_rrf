# AGENTS.md

## Commit/Push Rule
- `make test` が成功した場合にのみ commit & push を行うこと（失敗時は実施しない）。
- 例外: ユーザーが明示的に許可した docs-only 変更は `make test` なしで可。

## Test Runtime Note
- `make test` は 5 分以上かかる場合がある。

## バージョンアップ手順

1. `Cargo.toml` の `version` を更新
2. `META.json` の `version` / `provides.pg_rrf.version` を更新
3. `README.md` の Status Version を更新
4. `make test` を実行
5. コミット → tag（例: `v0.0.3`）→ push
6. GitHub Release を作成（tag と同名）

### ルール

- 一度切ったバージョンは絶対に上書きしない（必ず version bump する）
