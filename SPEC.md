# AI Security Guardian (AI 資安守門員) - 獨立軟體規格書 v1.0

## 1. 產品定義 (Product Definition)
這是一個**輕量級、獨立運行**的桌面資安工具。它不依賴 Nerty 或其他大型 AI 平台，而是作為一個獨立的 Binary/腳本，專注於透過本地模型提供「即時螢幕監控」與「敏感行為攔截」。

## 2. 核心功能 (Standalone Features)

### A. 獨立視覺哨兵 (Isolated Visual Sentry)
- **零依賴運行**：獨立進程，不與其他 AI 系統共用權限，專注監控桌面。
- **特權視窗偵測**：當系統彈出權限請求 (Sudo/TCC) 或密碼輸入框時，立即捕捉快照並存證。

### B. 本地行為防火牆 (Local Behavior Firewall)
- **剪貼簿安全**：監控 Clipboard 內容，偵測到 API Key 或明文密碼時，主動提醒用戶清理。
- **終端指令預審**：針對 Shell 執行的高風險關鍵字進行亞秒級攔截提醒。

### C. 物理通報網關
- **Direct Webhook**：直接透過實體網卡發送 Telegram 警報，繞過所有中轉代理。

## 3. 技術棧 (Tech Stack) - [對齊 2026-03-04 指令]
- **核心引擎 (The Kernel)**：**Rust-Kernel** (負責物理級監控、進程管理與亞秒級攔截)
- **AI 橋接層**：**PyO3** (將 **Python-Brain** AI 大腦嵌入 Rust 核心)
- **視覺引擎**：OmniParser-v2.0 / Florence-2 / Qwen2.5-VL
- **推理後端**：Ollama (Token Zero 本地推理)
- **開發語言**：**Rust (核心)** / Python (AI 邏輯)
- **介面層**：Tauri (基於 Rust 的跨平台 UI 框架)
- **通報渠道**：Telegram Physical Channel (物理網關直連)

## 4. 階段開發目標 (Milestones)
- **Phase 1**：建立 Python 原型驗證 (已完成：剪貼簿、視覺、指令攔截)。
- **Phase 2 (衝刺中)**：**Rust 核心點火**。將 Python 驗證過的邏輯使用 Rust 重新實作，並透過 PyO3 建立「鋼鐵外骨骼」。
- **Phase 3**：實作語義過濾器，對齊 NT 資安主權協議。
- **Phase 4 (當前最高優先級)**：**編譯與安裝檔封裝 (Production Build)**。實作跨平台封裝邏輯，產出可執行的 `.dmg` (macOS) 與 `.exe` (Windows) 安裝檔，並整合所有本地 AI 權重依賴。

---
*Generated for Standalone Development by Nerty on 2026-03-03*
