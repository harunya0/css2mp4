# TODO & ロードマップ

## フェーズ 1: コア API / ライブラリの実装 (`css2mp4-core`)
- [x] レンダリング設定 (`RenderOptions`) とタスク実行トレイトの定義
- [x] ヘッドレス Chromium を起動し、指定 HTML の特定フレームをキャプチャする API の実装 (`FrameCapturer`)
  - [x] CDP の `Animation` ドメインによる仮想時間シーク (`seekAnimations`, `setPlaybackRate: 0`)
  - [x] 透過背景（アルファチャンネル）PNG キャプチャ対応
- [x] キャプチャしたフレームストリームを FFmpeg にパイプして動画化する API の実装 (`FfmpegEncoder`)
  - [x] `image2pipe` によるディスク I/O なしのストリーミングエンコード
  - [x] MP4 (H.264 / yuv420p) 出力対応
  - [x] 透過 WebM (VP9 / yuva420p) 出力対応
- [x] CSS プロパティ（Transform / Opacity）を時系列サンプリングして `.ymmp` 用 JSON を生成する API の実装 (`YmmpProject`, `MotionSamples`)
  - [x] `DOMMatrixReadOnly` による行列分解（移動量・回転・スケール）
  - [x] UTF-8 BOM 付き JSON のラウンドトリップ（フィールド欠落防止）
  - [x] YMM4 内部 Enum (`直線移動` / `なし`) との完全互換対応
  - [x] **RDP & 極値検出によるキーフレーム（中間点）自動削減・最適化オプティマイザ** (`ymmp/optimizer.rs`)
- [x] HTML 文字列 / 一時ファイルからの単発プレビューフレーム生成 API の実装 (`preview_frame`)

## フェーズ 2: CLI ラッパーの実装 (`css2mp4-cli`)
- [x] clap を用いた CLI コマンド・オプションの定義
- [x] コア API を呼び出す CLI サブコマンドの実装
  - [x] `render`: 動画レンダリング (MP4 / 透過 WebM)
  - [x] `export-ymmp`: 既存 `.ymmp` 内アイテムへのモーション上書き（`--tolerance` による間引き調整対応）
- [x] indicatif を用いたレンダリング進捗プログレスバーの表示
- [x] 実行時エラーのハンドリング（Chromium / FFmpeg が見つからない場合の警告等）

## フェーズ 3: API インターフェース拡張（フロントエンド接続準備）
- [x] Axum を用いたローカル HTTP / SSE API エンドポイントの実装 (`serve` コマンド)
- [x] フロントエンドからの HTML / CSS 送信とプレビュー用フレーム取得 API の実装 (`POST /api/preview`)
- [x] 非同期レンダリングタスク作成 API の実装 (`POST /api/render`)
- [x] エクスポート進捗をリアルタイム通知するイベント配信 (SSE) の実装 (`GET /api/render/:id/events`)
- [x] 生成動画ダウンロード API の実装 (`GET /api/render/:id/download`)

## 今後の改善・検討課題
- [ ] **バッチ処理・複数セレクタ対応**:
  - 複数要素のモーションを一度にサンプリングして複数アイテムに適用する機能
- [ ] **パフォーマンス最適化**:
  - Chromium セッションのプーリング / 再利用
  - フレームキャプチャの並列化（複数タブでの分散レンダリング検討）
