# css2mp4 サンプル集

本フォルダには、`css2mp4` で動画化（MP4 / 透過WebM）や YMM4（.ymmp）へのモーション連携を試すための HTML / CSS サンプルが含まれています。

---

## 📂 サンプル一覧

### 1. `01-pop-in` (ポップアップカード)
中央からバウンドしながら拡大・出現するモダンなグラデーションカードのアニメーションです。

- **ファイル**: [`samples/01-pop-in/index.html`](01-pop-in/index.html), [`samples/01-pop-in/style.css`](01-pop-in/style.css)
- **推奨アニメーション時間**: `3.0` 秒

#### 実行例:
```bash
# MP4 動画として出力 (1920x1080, 60fps, 3秒)
css2mp4-cli render samples/01-pop-in/index.html -o pop-in.mp4 --fps 60 --duration 3.0

# 透過背景の WebM 動画として出力 (動画編集ソフトでの合成用)
css2mp4-cli render samples/01-pop-in/index.html -o pop-in.webm --transparent --fps 60 --duration 3.0

# YMM4 プロジェクト内のアイテムへモーションを上書き
css2mp4-cli export-ymmp samples/01-pop-in/index.html --selector "#target" --ymmp your-project.ymmp -o output.ymmp --duration 3.0
```

---

### 2. `02-slide-badge` (スライドインバッジ)
画面左からスムーズにスライドインし、定位置に収まるダークテーマのバッジアニメーションです。

- **ファイル**: [`samples/02-slide-badge/index.html`](02-slide-badge/index.html), [`samples/02-slide-badge/style.css`](02-slide-badge/style.css)
- **推奨アニメーション時間**: `2.5` 秒

#### 実行例:
```bash
# MP4 出力
css2mp4-cli render samples/02-slide-badge/index.html -o badge.mp4 --fps 60 --duration 2.5

# 透過 WebM 出力
css2mp4-cli render samples/02-slide-badge/index.html -o badge.webm --transparent --fps 60 --duration 2.5
```

---

## 💡 HTML / CSS 作成のポイント

1. **外部CSSの相対パス参照**:
   `index.html` から `<link rel="stylesheet" href="./style.css">` のように相対パスで読み込むことができます。
2. **背景透過 (`--transparent`)**:
   `body` に `background: transparent;` を指定しておくと、`--transparent` フラグで書き出した際に背景が抜けたアルファチャンネル付き WebM になります。
3. **YMM4 モーション抽出**:
   動かしたい要素に `id="target"` などのセレクタを付与し、`--selector "#target"` を指定して `export-ymmp` を実行します。
