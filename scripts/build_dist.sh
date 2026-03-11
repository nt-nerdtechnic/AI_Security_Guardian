#!/bin/bash
set -e

echo "=== AI Security Guardian 打包腳本 ==="

# 設定目錄
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$PROJECT_DIR/dist/guardian_release"

# 尋找 PyInstaller
PYINSTALLER="$PROJECT_DIR/venv/bin/pyinstaller"
if [ ! -f "$PYINSTALLER" ]; then
    if ! command -v pyinstaller &> /dev/null; then
        echo "錯誤: 找不到 pyinstaller，請透過 pip install pyinstaller 安裝。"
        exit 1
    fi
    PYINSTALLER="pyinstaller"
fi

# 確認 cargo
if ! command -v cargo &> /dev/null; then
    echo "錯誤: 找不到 cargo，請安裝 Rust。"
    exit 1
fi

# 確認 npm / node
if ! command -v npm &> /dev/null; then
    echo "錯誤: 找不到 npm，請安裝 Node.js。"
    exit 1
fi

echo "清理舊的發布檔..."
rm -rf "$PROJECT_DIR/dist"
rm -rf "$PROJECT_DIR/build"
mkdir -p "$DIST_DIR"

echo "=== 1. 編譯 Rust 核心 (guardian_core) ==="
cd "$PROJECT_DIR/guardian_core"
cargo build --release
cp target/release/aegis-guardian-core "$DIST_DIR/" 2>/dev/null || cp target/release/guardian_core "$DIST_DIR/" 2>/dev/null || true

echo "=== 2. 編譯 Python 大腦 (guardian.py) ==="
cd "$PROJECT_DIR"
# 使用 PyInstaller 封裝，包含 models 目錄
$PYINSTALLER \
    --noconfirm \
    --onedir \
    --add-data "models:models" \
    --hidden-import "plyer.platforms.mac.notification" \
    --name AI_Guardian_Brain \
    guardian.py

echo "=== 3. 打包 Tauri UI (guardian_ui) ==="
cd "$PROJECT_DIR/guardian_ui"
npm install
npm run tauri build

# 找出並複製 .dmg (macOS)
DMG_FILE=$(find "$PROJECT_DIR/guardian_ui/src-tauri/target/release/bundle/dmg" -name "*.dmg" 2>/dev/null | head -1)
if [ -n "$DMG_FILE" ]; then
    cp "$DMG_FILE" "$DIST_DIR/"
    echo "✅ .dmg 安裝檔已複製至: $DIST_DIR"
else
    echo "⚠️  未找到 .dmg，請確認 macOS 打包是否成功。"
fi

# 找出並複製 .msi / .exe (Windows，若在跨平台環境)
MSI_FILE=$(find "$PROJECT_DIR/guardian_ui/src-tauri/target/release/bundle/msi" -name "*.msi" 2>/dev/null | head -1)
if [ -n "$MSI_FILE" ]; then
    cp "$MSI_FILE" "$DIST_DIR/"
    echo "✅ .msi 安裝檔已複製至: $DIST_DIR"
fi

echo "=== 4. 物理打包 ==="
# 將 PyInstaller 產出的結果連同 Rust 核心整理為最終發行版目錄
cp -r "$PROJECT_DIR/dist/AI_Guardian_Brain" "$DIST_DIR/" 2>/dev/null || true

echo ""
echo "🎉 打包完成！發行檔已準備於: $DIST_DIR"
echo "$(ls "$DIST_DIR")"
