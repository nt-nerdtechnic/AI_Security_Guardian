@echo off
chcp 65001 > nul
setlocal

:: ============================================================================
:: Aegis Guardian - Models Installation Script (Windows)
:: ============================================================================

echo 🤖 歡迎使用 Aegis Guardian 模型一鍵安裝工具
echo ===========================================================

:: 1. 檢查 Ollama 是否安裝
where ollama >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo [錯誤] 您的系統尚未安裝 Ollama。
    echo 請至官網下載並安裝: https://ollama.com/download
    echo 安裝完畢後，確保 Ollama 系統列圖示有出現（服務執行中），再重新執行此腳本。
    pause
    exit /b 1
)

echo [✔] Ollama 已安裝，準備下載所需的模型...
echo 這可能會花費數分鐘時間，取決於您的網路速度。
echo -----------------------------------------------------------

:: 2. 下載語意分析模型 (llama3)
echo 📥 正在下載語意分析模組 (llama3)...
ollama pull llama3
if %ERRORLEVEL% neq 0 (
    echo [錯誤] llama3 下載失敗，請檢查網路連線或 Ollama 服務狀態。
    pause
    exit /b 1
)

echo -----------------------------------------------------------

:: 3. 下載視覺分析模型 (qwen2.5-vl)
echo 📥 正在下載視窗/圖片分析模組 (qwen2.5vl:latest)...
ollama pull qwen2.5vl:latest
if %ERRORLEVEL% neq 0 (
    echo [錯誤] qwen2.5vl 下載失敗，請檢查網路連線或 Ollama 服務狀態。
    pause
    exit /b 1
)

echo ===========================================================
echo 🎉 模型全部下載完畢！您現在可以直接啟動 Aegis Guardian 的 Python 後端引擎了。
echo 啟動指令： venv\Scripts\activate ^&^& python core\main.py
pause
