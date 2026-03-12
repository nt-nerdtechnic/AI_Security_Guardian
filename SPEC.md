# Aegis Guardian (AI 資安守門員) - 獨立軟體規格書 v1.1

## 1. 產品定義 (Product Definition)
這是一個**輕量級、獨立運行**的桌面資安工具。它透過 Python 監控引擎 (Core) 與 Tauri (UI 殼層) 的協作，提供「即時螢幕監控」與「敏感行為攔截」。

## 2. 核心功能 (Standalone Features)

### A. 獨立視覺哨兵 (Isolated Visual Sentry)
- **零依賴運行**：由 Python Sidecar 獨立進程監控桌面，不干擾主系統運行。
- **特權視窗偵測**：當偵測到敏感視窗（如 Terminal、Settings）時，啟動 AI 語義視覺分析。

### B. 本地行為防火牆 (Local Behavior Firewall)
- **剪貼簿安全**：即時監控剪貼簿，偵測 API Key 或密碼，並可由 `MitigationManager` 自動清空。
- **網路行為監控**：攔截非預期的埠口開啟或敏感連線。

### C. 物理通報網關
- **Direct Bot Notification**：直接透過 Telegram Bot API 發送告警，無需中轉。

## 3. 技術棧 (Tech Stack) - [2026-03-13 同步]
- **監控引擎 (Core Engine)**：**Python (MVVM)** (負責行為監控、AI 調度與自動緩解)
- **UI & 系統外殼**：**Tauri (Rust)** (負責進程生命週期管理、UI 呈現、高權限動作協作)
- **通訊機制**：**Sidecar + NDJSON Logs** (Python 作為 Sidecar 執行，透過日誌文件共享事件)
- **AI 大腦**：OmniParser / Florence-2 (透過 Ollama 本地推理)
- **開發語言**：Python 3.10+ / Rust (Tauri) / TypeScript (React)

## 4. 階段開發目標 (Milestones)
- **Phase 1**：建立 Python 原型驗證 (已完成)。
- **Phase 2 (現況)**：**Tauri 外殼與 Sidecar 整合**。完成 Python 監控引擎作為 Tauri Sidecar 運行，並透過日誌與 UI 即時溝通。
- **Phase 3**：**主動緩解增強**。完整實現 `MitigationManager` 的各種自動防禦策略（如網路流量快照、檔案隔離）。
- **Phase 4**：**編譯與安裝檔封裝 (Production Build)**。產出 `.dmg` (macOS) 與 `.exe` (Windows) 安裝檔，整合所有環境依賴。

---
*Updated for Current Architecture on 2026-03-13*
