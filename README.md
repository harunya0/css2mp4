# css2mp4

CSS アニメーションで定義されたモーションを、高品質な動画ファイル（MP4 / 透過 WebM）および「ゆっくりMovieMaker4（YMM4）」のプロジェクトファイル（`.ymmp`）へ書き出すためのレンダリングエンジン / CLI ツールです。

---

## 主な特徴

- **決定論的なコマ送りレンダリング**  
  Chrome DevTools Protocol (CDP) の `Animation` ドメインを用いて仮想時間を制御。マシンスペックやリアルタイム再生速度に依存せず、指定したフレームレートで正確にキャプチャします。
- **高速なストリーミングエンコード**  
  キャプチャした PNG フレームをパイプ経由（`image2pipe`）で直接 FFmpeg に流し込み、ディスクへの無駄な中間ファイル書き出しを行いません。
- **透過動画のサポート**  
  アルファチャンネル付きの背景透過 WebM（VP9 コーデック）出力に対応しており、動画編集ソフトへの素材合成に最適です。
- **ゆっくりMovieMaker4 (YMM4) 連携**  
  CSS で記述した `transform`（移動・回転・拡縮）や `opacity`（不透明度）をフレームごとに解析・サンプリングし、既存 `.ymmp` ファイル内のアイテムのキーフレームとして直接上書き反映できます。
- **モジュラー設計**  
  コア機能は独立した Rust クレート（`css2mp4-core`）として設計されており、CLI だけでなく将来的な Web サーバー / GUI アプリケーション（Tauri 等）への組み込みが容易です。

---

## 前提条件

本ツールを実行するには、以下の環境が必要です。

1. **Rust** (1.75 以上推奨)
2. **Google Chrome / Chromium** (ヘッドレスブラウザとして使用。自動検出されますが、パス指定も可能)
3. **FFmpeg** (システム PATH に通っているか、CLI 引数で実行パスを指定)

---

## インストール / ビルド

リポジトリをクローンし、Cargo でビルドします。

```bash
git clone <repository-url>
cd css-motion-generation

# リリースビルド
cargo build --release
```

バイナリは `target/release/css2mp4-cli` に生成されます。

---

## 使い方

### 1. 動画レンダリング (`render`)

HTML / CSS で定義されたアニメーションを動画として書き出します。

```bash
# 基本的な MP4 出力（1920x1080, 60fps, 3秒）
css2mp4-cli render input.html -o output.mp4 --fps 60 --duration 3.0

# 透過背景の WebM 出力
css2mp4-cli render input.html -o output.webm --transparent --fps 60 --duration 3.0

# 解像度や Chrome/FFmpeg パスを指定して出力
css2mp4-cli render input.html -o output.mp4 \
  --width 1280 \
  --height 720 \
  --fps 30 \
  --duration 5.0 \
  --chrome "C:\Program Files\Google\Chrome\Application\chrome.exe" \
  --ffmpeg "C:\ffmpeg\bin\ffmpeg.exe"
```

#### 主なオプション (`render`)
| オプション | デフォルト値 | 説明 |
| :--- | :--- | :--- |
| `input` | *(必須)* | 入力となる HTML ファイルのパス |
| `-o, --output` | *(必須)* | 出力ファイルパス（拡張子 `.mp4` / `.webm` を判別） |
| `--fps` | `60` | フレームレート |
| `--duration` | `3.0` | アニメーションの長さ（秒） |
| `--width` | `1920` | レンダリング幅（px） |
| `--height` | `1080` | レンダリング高さ（px） |
| `--transparent` | `false` | 透過動画（WebM VP9 + アルファ）として出力 |
| `--ffmpeg` | `ffmpeg` | FFmpeg 実行ファイルのパス |
| `--chrome` | *(自動検出)* | Chromium / Chrome 実行ファイルのパス |

---

### 2. YMM4 プロジェクトへのモーション書き出し (`export-ymmp`)

HTML 内の指定要素（CSS セレクタ）のアニメーション軌跡をサンプリングし、既存の YMM4 プロジェクトファイル（`.ymmp`）内のアイテムにキーフレーム情報として注入します。

```bash
# #target 要素のアニメーションを sample.ymmp のタイムライン 0, アイテム 0 に上書き
css2mp4-cli export-ymmp input.html \
  --selector "#target" \
  --ymmp sample.ymmp \
  -o output.ymmp \
  --fps 60 \
  --duration 3.0
```

#### 主なオプション (`export-ymmp`)
| オプション | デフォルト値 | 説明 |
| :--- | :--- | :--- |
| `input` | *(必須)* | 入力となる HTML ファイルのパス |
| `--selector` | *(必須)* | モーションサンプリング対象の CSS セレクタ（例: `#target`, `.box`） |
| `--ymmp` | *(必須)* | 上書き対象の既存 `.ymmp` ファイルのパス |
| `-o, --output` | *(省略時は `--ymmp` を上書き)* | 書き出し先の `.ymmp` ファイルパス |
| `--timeline-index`| `0` | 対象タイムラインのインデックス（0始まり） |
| `--item-index` | `0` | 対象アイテムのインデックス（0始まり） |
| `--fps` | `60` | サンプリング密度（fps） |
| `--duration` | `3.0` | サンプリング時間（秒） |

---

### 3. ローカル API サーバーの起動 (`serve`)

フロントエンド（Web UI / 外部ツール等）からプレビュー生成や非同期動画エクスポート、SSEによる進捗監視を行うための REST + SSE サーバーを起動します。

```bash
# デフォルト（127.0.0.1:3000）で起動
css2mp4-cli serve

# ポート番号やバインドホストを指定して起動
css2mp4-cli serve --host 0.0.0.0 --port 8080
```

#### 提供される API エンドポイント
| メソッド | エンドポイント | 説明 |
| :--- | :--- | :--- |
| `GET` | `/api/health` | サーバーの死活確認 |
| `POST` | `/api/preview` | HTML/CSS 文字列とシーク秒数から 1 フレームの PNG 画像を取得 |
| `POST` | `/api/render` | 非同期レンダリングタスクを作成し `task_id` を返却 |
| `GET` | `/api/render/:task_id` | タスクの現在の進捗・ステータスを取得 |
| `GET` | `/api/render/:task_id/events` | **SSE (Server-Sent Events)** によるリアルタイム進捗ストリーム |
| `GET` | `/api/render/:task_id/download` | レンダリング完了した動画ファイル（MP4 / WebM）をダウンロード |

---

## プロジェクト構成

```text
css2mp4/
├── Cargo.toml               # ワークスペース定義
├── README.md                # 本ドキュメント
├── todo.md                  # 開発ロードマップ & TODO
├── css2mp4-core/            # コアライブラリ
│   ├── src/
│   │   ├── lib.rs           # 公開モジュール定義・re-export
│   │   ├── error.rs         # 共通エラー定義 (Error, Result)
│   │   ├── options.rs       # レンダリング設定 (RenderOptions)
│   │   ├── browser/         # ヘッドレスブラウザ & CDP 制御
│   │   │   ├── mod.rs
│   │   │   ├── capturer.rs  # ブラウザ起動・仮想時間同期・PNGキャプチャ
│   │   │   ├── sampler.rs   # CSS Computed Style サンプリング & 行列分解
│   │   │   └── url.rs       # file:// URL 変換
│   │   ├── encoder/         # 動画エンコード & FFmpeg 連携
│   │   │   ├── mod.rs
│   │   │   ├── ffmpeg.rs    # FFmpeg プロセス管理 & stdin ストリーミング
│   │   │   └── format.rs    # 出力形式 (VideoFormat) & 引数生成
│   │   ├── ymmp/            # YMM4 (.ymmp) ドメイン
│   │   │   ├── mod.rs
│   │   │   ├── model.rs     # プロジェクト / タイムライン / アイテム構造
│   │   │   ├── property.rs  # アニメーションプロパティ / キーフレーム / ベジェ
│   │   │   ├── io.rs        # UTF-8 BOM 対応の読み込み & 保存
│   │   │   └── motion.rs    # サンプリングデータ (MotionSamples) と適用処理
│   │   └── pipeline/        # 高レベルオーケストレーション
│   │       ├── mod.rs
│   │       ├── progress.rs  # 進捗通知トレイト (ProgressSink, NoopProgress)
│   │       ├── video.rs     # 動画レンダリングパイプライン (render_video)
│   │       └── ymmp.rs      # YMM4 モーション上書きパイプライン (overwrite_ymmp_motion)
│   └── tests/               # YMMP 読み書き・ラウンドトリップテスト
└── css2mp4-cli/             # CLI バイナリ
    └── src/
        ├── main.rs          # エントリポイント（ルーティング）
        ├── args.rs          # clap 引数・サブコマンド定義
        ├── commands/        # 各サブコマンド処理
        │   ├── mod.rs
        │   ├── render.rs
        │   ├── export_ymmp.rs
        │   └── serve.rs
        └── ui/              # CLI UI
            ├── mod.rs
            └── progress.rs  # indicatif プログレスバー
```

---

## 技術スタック

- **言語**: Rust (Edition 2021)
- **CLI フレームワーク**: [clap](https://github.com/clap-rs/clap)
- **プログレスバー**: [indicatif](https://github.com/console-rs/indicatif)
- **ブラウザ自動化 (CDP)**: [chromiumoxide](https://github.com/mattsse/chromiumoxide)
- **シリアライズ / JSON**: [serde](https://serde.rs/), [serde_json](https://github.com/serde-rs/json)
- **動画エンコード**: [FFmpeg](https://ffmpeg.org/) (外部プロセス連携)
- **非同期ランタイム**: [tokio](https://tokio.rs/)

---

## テストの実行

```bash
cargo test
```

---

## ロードマップ

実装状況の詳細および今後の開発計画については [todo.md](todo.md) をご覧ください。