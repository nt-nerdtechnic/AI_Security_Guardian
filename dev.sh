#!/bin/bash

# 定義顏色
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🚀 正在啟動 Aegis Guardian 開發環境...${NC}"

# 檢查虛擬環境是否存在
if [ ! -d "venv" ]; then
    echo -e "${RED}❌ 找不到 venv 虛擬環境，請先執行 python3 -m venv venv${NC}"
    exit 1
fi

# 1. 啟動 Python 後端監控核心
echo -e "${GREEN}Step 1: 啟動後端監控核心 (Python)...${NC}"
source venv/bin/activate
python core/main.py &
BACKEND_PID=$!

# 2. 啟動 Tauri 前端 UI
echo -e "${GREEN}Step 2: 啟動前端 UI 儀表板 (Tauri)...${NC}"
cd ui
npm run tauri dev &
FRONTEND_PID=$!

# 設定清理機制：當按下 Ctrl+C 時，同時關閉前後端
cleanup() {
    echo -e "\n${RED}🛑 正在關閉所有程序...${NC}"
    kill $BACKEND_PID 2>/dev/null
    kill $FRONTEND_PID 2>/dev/null
    echo -e "${BLUE}👋 已安全結束。${NC}"
    exit
}

trap cleanup SIGINT

echo -e "${BLUE}---------------------------------------${NC}"
echo -e "✅ 服務已啟動！"
echo -e "   - 後端 PID: $BACKEND_PID"
echo -e "   - 前端 PID: $FRONTEND_PID"
echo -e "按 ${RED}Ctrl+C${NC} 即可同時停止前後端服務。"
echo -e "${BLUE}---------------------------------------${NC}"

# 等待進程結束
wait
