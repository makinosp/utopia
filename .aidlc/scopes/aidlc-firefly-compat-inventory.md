---
name: firefly-compat-inventory
depth: Minimal
keywords: []
description: Firefly III互換APIの棚卸し — 実装済み仕様と本家差分、優先順位整理
skeleton: on
review_cap: advisory
---

# firefly-compat-inventory scope

Firefly III互換APIの現状を棚卸しし、実装済み仕様と本家Firefly IIIとの差分、今後の優先順位を整理する分析スコープ。コード実装ではなくドキュメント/分析が成果物。

## Why these stages, why skip those

- **intent-capture** — 棚卸しの範囲・比較対象バージョン・優先順位基準を確定（IAE/UA解消）
- **reverse-engineering** — openapi.yaml / handlers / modules / core / migrations を走査し実装済み仕様を抽出（CSU削減）— 中核
- **requirements-analysis** — 本家Firefly III仕様との差分マトリクスと優先順位付けフレームを文書化 — 中核

市場調査・フィージビリティ・スコープ分割・チーム編成・モック・設計・実装・検証・デプロイ・運用はすべて対象外（読み取り専用の棚卸し、R=LOW, VE=LOW）。3ステージに絞ることで最小コストで「現状→差分→優先順位」の文書を完結する。

## Membership

Initialization 3 stages + intent-capture, reverse-engineering, requirements-analysis が EXECUTE。残り26ステージは SKIP。
