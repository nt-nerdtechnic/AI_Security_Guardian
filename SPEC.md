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

## 3. 技術棧 (Tech Stack) - [2026-03-30 同步]
- **監控引擎 (Core Engine)**：**Python (MVVM)** (負責行為監控、AI 調度與自動緩解)
- **UI & 系統外殼**：**Tauri v2 (Rust)** (負責 Sidecar 管理、UI 呈現、高權限動作執行)
- **通訊機制**：**Sidecar + NDJSON Logs + Tauri IPC emit** (Python 寫入日誌，Rust tail-read 並推送前端)
- **AI 大腦**：`llama3`（語義分析）/ `qwen2.5vl:latest`（視覺多模態）透過 Ollama 本地推理
- **白名單持久化**：SQLite (`rusqlite` bundled)，存於 `~/.aegis-guardian/whitelist.db`
- **開發語言**：Python 3.10+ / Rust 1.77+ (Tauri v2) / React + Vite (JSX + Tailwind CSS)

## 4. 階段開發目標 (Milestones)
- **Phase 1** ✅：建立 Python 原型驗證（所有監控模組、AI 腦核、Telegram 通報均完成）。
- **Phase 2** ✅：**Tauri 外殼與 Sidecar 整合**。Python Sidecar 啟動、stdout/stderr 轉發、Rust 背景執行緒輪詢 incidents.json 並即時 emit AI 告警至 React 前端。
- **Phase 3** ✅：**主動緩解增強**。Python `MitigationManager` 自動防禦（清空剪貼簿、終止進程、隔離檔案）與 Rust Commands（kill / isolate / resume / quarantine）均已實作完成。
- **Phase 4** ✅：**編譯與安裝檔封裝 (Production Build)**。已產出 `AI Security Guardian_1.1.0_aarch64.dmg` 與 `.app` bundle（macOS Apple Silicon），`tauri.conf.json` 已設定 `externalBin` 打包 Sidecar binary。

### 已知待改善項目
- `whitelist.rs` 每次啟動執行 `DROP TABLE`，重啟後白名單會被清空，需改為 `CREATE TABLE IF NOT EXISTS`。
- `file_integrity.rs` 以「24 小時內修改」為 WARNING 判斷，缺乏 checksum 機制，誤報率偏高。
- Telegram callback 處理未驗證來源 (HMAC-SHA256)，存在偽造 callback 的遠端控制風險。
- `update_config` Rust 側會完全覆寫 `config.yaml`，破壞 Python 側的 `behavior_firewall` / `terminal_rules` 等欄位。

---
*Updated for Current Architecture on 2026-03-30*
