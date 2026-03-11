#!/usr/bin/env bash
# =============================================================================
# Aegis Guardian - Python Daemon Builder
# 將 Python 後端 (core/main.py) 使用 PyInstaller 打包成獨立執行檔，
# 並放置到 Tauri 的 binaries/ 目錄，供 Tauri bundle 時一同封裝。
#
# 使用方式：
#   chmod +x scripts/build_daemon.sh
#   ./scripts/build_daemon.sh
# =============================================================================

set -e  # 任何指令失敗立即終止腳本

# ── 路徑設定 ─────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VENV_DIR="$PROJECT_ROOT/venv"
BINARIES_DIR="$PROJECT_ROOT/ui/src-tauri/binaries"
ENTRY_POINT="$PROJECT_ROOT/core/main.py"
DAEMON_NAME="aegis-core-daemon"

echo "=========================================="
echo " Aegis Guardian - Python Daemon Builder"
echo "=========================================="

# ── Step 1: 確認並啟動 venv ──────────────────────────────────────────────────
if [ ! -f "$VENV_DIR/bin/activate" ]; then
    echo "[ERROR] venv 不存在，請先執行: python3 -m venv venv && pip install -r requirements.txt"
    exit 1
fi

echo "[1/5] 啟動 Python 虛擬環境..."
source "$VENV_DIR/bin/activate"

# ── Step 2: 安裝 PyInstaller ─────────────────────────────────────────────────
echo "[2/5] 確認 PyInstaller 已安裝..."
pip install --quiet pyinstaller

# ── Step 3: 偵測目標平台，決定輸出檔名 ──────────────────────────────────────
ARCH=$(uname -m)
OS=$(uname -s)

if [ "$OS" = "Darwin" ]; then
    if [ "$ARCH" = "arm64" ]; then
        TARGET_TRIPLE="aarch64-apple-darwin"
    else
        TARGET_TRIPLE="x86_64-apple-darwin"
    fi
elif [ "$OS" = "Linux" ]; then
    if [ "$ARCH" = "aarch64" ]; then
        TARGET_TRIPLE="aarch64-unknown-linux-gnu"
    else
        TARGET_TRIPLE="x86_64-unknown-linux-gnu"
    fi
else
    echo "[WARN] 未知平台 $OS，使用 generic 作為後綴"
    TARGET_TRIPLE="generic"
fi

BINARY_FILENAME="${DAEMON_NAME}-${TARGET_TRIPLE}"
echo "[3/5] 偵測到平台: $OS/$ARCH -> 目標為: $BINARY_FILENAME"

# ── Step 4: 執行 PyInstaller 打包 ────────────────────────────────────────────
echo "[4/5] 執行 PyInstaller 打包..."
cd "$PROJECT_ROOT"

pyinstaller \
    --onefile \
    --noconfirm \
    --name "$BINARY_FILENAME" \
    --distpath "$BINARIES_DIR" \
    --workpath /tmp/aegis_build \
    --specpath /tmp/aegis_build \
    --add-data "$PROJECT_ROOT/locales:locales" \
    --add-data "$PROJECT_ROOT/config.yaml:." \
    "$ENTRY_POINT"

# ── Step 5: 設定執行權限並確認 ───────────────────────────────────────────────
echo "[5/5] 設定執行權限..."
chmod +x "$BINARIES_DIR/$BINARY_FILENAME"

echo ""
echo "✅ 打包完成！"
echo "   輸出路徑: $BINARIES_DIR/$BINARY_FILENAME"
echo ""
echo "下一步: 在 ui/ 目錄執行以下指令以產出安裝包："
echo "   cd ui && npm run tauri build"
echo "=========================================="
