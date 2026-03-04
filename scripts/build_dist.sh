#!/bin/bash
set -e

echo "=== AI Security Guardian 打包腳本 ==="

# 確認環境
if ! command -v cargo &> /dev/null; then
    echo "錯誤: 找不到 cargo，請安裝 Rust。"
    exit 1
fi

if ! command -v pyinstaller &> /dev/null; then
    echo "錯誤: 找不到 pyinstaller，請透過 pip install pyinstaller 安裝。"
    exit 1
fi

# 設定目錄
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$PROJECT_DIR/dist/guardian_release"

echo "清理舊的發布檔..."
rm -rf "$PROJECT_DIR/dist"
rm -rf "$PROJECT_DIR/build"
mkdir -p "$DIST_DIR"

echo "=== 1. 編譯 Rust 核心 (guardian_core) ==="
cd "$PROJECT_DIR/guardian_core"
cargo build --release
cp target/release/guardian_core "$DIST_DIR/"

echo "=== 2. 編譯 Python 大腦 (guardian.py) ==="
cd "$PROJECT_DIR"
# 使用 PyInstaller 封裝，包含 models 目錄
# --onedir 方便除錯與包含權重檔。若確定沒問題未來可轉 --onefile
pyinstaller \
    --noconfirm \
    --onedir \
    --add-data "models:models" \
    --hidden-import "plyer.platforms.mac.notification" \
    --name AI_Guardian_Brain \
    guardian.py

echo "=== 3. 物理打包 ==="
# 將 PyInstaller 產出的結果連同 Rust 核心整理為最終發行版目錄
# (AI_Guardian_Brain 整個資料夾會進入 release 內，並放上 guardian_core 執行檔)
cp -r "$PROJECT_DIR/dist/AI_Guardian_Brain" "$DIST_DIR/"

echo "打包完成！發行檔已準備於: $DIST_DIR"
