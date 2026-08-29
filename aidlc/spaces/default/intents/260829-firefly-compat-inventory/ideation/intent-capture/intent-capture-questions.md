# Intent Capture Questions

## Sources

- [desc] Initial description: "Firefly III互換APIの現状を棚卸しして、実装済み仕様と本家との差分、今後の優先順位を整理したい。"
- [scope] Workflow-selected scope: `firefly-compat-inventory`.

## Q1. 今回の棚卸しで解決したいビジネス課題は何ですか？

Firefly III互換APIの現状把握と今後の開発優先順位付けの背景を教えてください。

A. 既存実装がどこまで本家Firefly IIIと互換か不明で、外部クライアント連携時に不具合が出るリスクがある
B. 新機能追加の前に、未実装領域を可視化してロードマップを策定したい
C. ドキュメントが分散しており、チーム内で実装状況の共通認識を作りたい
D. 本家Firefly IIIのバージョンアップに追従するための差分管理基盤が欲しい
E. Not yet defined

[Answer]:

## Q2. 誰がこの棚卸し結果を利用しますか？（顧客/利用者）

A. 開発チーム内部 — 今後の実装計画・優先順位決定の材料として
B. 外部連携先（Firefly IIIクライアントアプリ等）の開発者 — 互換性確認のため
C. プロダクトオーナー/意思決定者 — 投資判断・スコープ調整のため
D. 全員（開発チーム + 外部連携先 + 意思決定者）
E. Not identified

[Answer]:

## Q3. 成功の定義と成果物に求めるものは何ですか？

A. エンドポイント単位の互換性マトリクス（実装済み/未実装/部分実装）と優先順位付きバックログが文書化されている
B. openapi.yaml と本家Firefly III OpenAPI仕様の差分が表形式で整理されている
C. 優先順位付けの評価軸（利用頻度/工数/互換性重要度）が合意され、Top Nの次期実装候補が決まっている
D. A + B + C すべて
E. Not yet defined

[Answer]:

## Q4. なぜ今この棚卸しが必要ですか？（トリガー）

A. UtopiaのAccounts/Transactions等のコア機能が一通り実装され、次の拡張を計画する節目だから
B. 外部からの互換性問い合わせや不具合報告が増えてきたため
C. 本家Firefly IIIの仕様変更やバージョンアップに対応する必要があるため
D. チーム内の技術的負債や仕様の曖昧さを解消したいため
E. Not applicable

[Answer]:

## Q5. 比較対象とする本家Firefly IIIのバージョン・範囲をどうしますか？

A. 最新安定版（v6系）の全APIエンドポイントを対象に比較
B. Utopiaが現在対象としているドメイン（Accounts, Transactions, Budgets, Categories等）に絞って比較
C. openapi.yamlに既に定義されているエンドポイント群と本家仕様を1:1で突合
D. 本家Firefly IIIの公式ドキュメント（firefly-iii.org/api）に記載のエンドポイントを手動で棚卸し
E. Not yet defined — 推奨アプローチを提案してほしい

[Answer]:

## Q6. 優先順位付けの評価軸は何を重視しますか？

A. 互換性重要度 — Firefly IIIクライアントが頻繁に利用するエンドポイントを優先
B. 実装工数 — 少ない工数で互換性を大きく向上できるものから
C. ビジネス価値 — Utopiaのプロダクトゴールに直結するドメインを優先
D. A + B + C のバランス（重み付けしてスコアリング）
E. Not yet defined

[Answer]:

## Q7. 主要なステークホルダーと意思決定者は誰ですか？

A. 開発チーム（本リポジトリのコミッター）が主体で、プロダクトオーナーが優先順位を最終決定
B. 開発チーム + 外部連携先の代表者が協議して決定
C. 個人プロジェクト — 自分自身で全て決定
D. Not identified — 整理してほしい
E. Not applicable

[Answer]:

## Q8. ワークフローに選択されたスコープ `firefly-compat-inventory` は意図した境界と一致しますか？

本ワークフローは `firefly-compat-inventory`（Minimal depth, 6 stages: intent-capture / reverse-engineering / requirements-analysis）で開始されました。これは「コード実装なし、ドキュメント/分析のみ」の棚卸し専用スコープです。

A. はい、一致する — このまま棚卸し専用スコープで進めたい
B. いいえ、将来的に未実装APIの実装まで含めたいので、より広いスコープ（feature/mvp等）に変更したい
C. 棚卸し後に実装フェーズを別ワークフローで開始する予定なので、このスコープで問題ない
D. スコープの意味がよくわからないので説明してほしい
E. Other (please describe…)

[Answer]:
