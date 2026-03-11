# 🛡️ Aegis Guardian（AI 資安守門員）

> A lightweight, privacy-first AI security monitor for your desktop. It leverages **local AI models (via Ollama)** to detect threats — no data leaves your machine.

[繁體中文](#繁體中文說明) | [English](#english-guide)

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Platform: macOS](https://img.shields.io/badge/Platform-macOS-lightgrey)
![AI Engine: Ollama](https://img.shields.io/badge/AI-Ollama%20(Local)-green)

---

## 繁體中文說明

### 功能特色

| 功能 | 說明 |
|------|------|
| 🖥️ **視覺哨兵** | 偵測螢幕上出現的特權彈窗（sudo、密碼輸入框）並自動截圖存證 |
| 📋 **剪貼簿防火牆** | 即時掃描剪貼簿，偵測 API Key、私鑰、明文密碼 |
| 💻 **終端指令預審** | 攔截高風險 Shell 指令關鍵字（如 `rm -rf /`、`nc -e`）|
| 🌐 **網路異常監控** | 偵測可疑連線埠與大流量進出 |
| 📲 **Telegram 即時通報** | 威脅事件自動推送 Telegram |

### 系統需求

- **作業系統**: macOS 12+ (Monterey 以上)
- **Python**: 3.10+
- **Node.js**: 18+（UI 開發時需要）
- **Ollama**: 最新版本（見下方安裝說明）

### 快速開始 (Quick Start)

#### Step 1：安裝 Ollama

前往 [ollama.com/download](https://ollama.com/download) 下載並安裝 Ollama，安裝後確認系統列有 Ollama 圖示並處於執行中狀態。

#### Step 2：一鍵安裝 AI 模型

```bash
# 複製專案
git clone https://github.com/YOUR_USERNAME/AI_Security_Guardian.git
cd AI_Security_Guardian

# macOS / Linux
chmod +x install_models.sh
./install_models.sh

# Windows
# 雙擊 install_models.bat 執行
```

此腳本會自動下載：
- `llama3` — 語意分析模型（偵測高危指令與文字）
- `qwen2.5vl:latest` — 多模態視覺模型（偵測螢幕特權彈窗）

#### Step 3：設定通報渠道（可選）

編輯 `config.yaml`，填入您的 Telegram Bot Token 與 Chat ID：

```yaml
webhook:
  telegram:
    bot_token: 'YOUR_BOT_TOKEN'
    chat_id: 'YOUR_CHAT_ID'
```

#### Step 4：安裝 Python 依賴並啟動

```bash
# 建立虛擬環境
python3 -m venv venv
source venv/bin/activate        # Windows: venv\Scripts\activate

# 安裝依賴
pip install -r requirements.txt

# 啟動後端監控引擎
python core/main.py
```

#### Step 5：啟動 UI 儀表板（可選）

```bash
cd ui
npm install
npm run dev
```

---

## English Guide

### What is Aegis Guardian?

A real-time desktop security monitor powered by **local AI (Ollama)**. It watches your clipboard, terminal inputs, active windows, and network traffic for suspicious activity — completely offline.

### Installation

1. Install [Ollama](https://ollama.com/download)
2. Clone this repo and run `./install_models.sh` (or `install_models.bat` on Windows)
3. `pip install -r requirements.txt`
4. `python core/main.py`

### Architecture

See [ARCH.md](./ARCH.md) for the full technical architecture breakdown.

---

## License

This project is licensed under the **MIT License** — see [LICENSE](./LICENSE) for details.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for how to get involved.
