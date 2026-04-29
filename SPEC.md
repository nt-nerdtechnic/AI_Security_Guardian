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
- **AI 大腦**：預設 `llama3`（語義分析）/ `qwen2.5vl:latest`（視覺多模態）透過 Ollama 本地推理，可由 `config.yaml` 的 `ai.models` 或 `AEGIS_SEMANTIC_MODEL` / `AEGIS_VISUAL_MODEL` 覆寫。
- **語義判定 schema**：`guardian_brain.py` 以 JSON schema 解析 `verdict`、`confidence`、`category`、`reason`、`recommended_action`，並保留舊版 YES/NO fallback。
- **白名單持久化**：SQLite (`rusqlite` bundled)，存於 `~/.aegis-guardian/whitelist.db`
- **檔案完整性**：SHA-256 baseline 存於 `~/.aegis-guardian/file_checksums.json`，支援遞迴目錄 hash、接受單一變更與重建 baseline。
- **Telegram 遠端操作防護**：Inline callback 使用 sender 驗證與 HMAC-SHA256 簽名；遠端終止與隔離會寫入審計事件。
- **開發語言**：Python 3.10+ / Rust 1.77+ (Tauri v2) / React + Vite (JSX + Tailwind CSS)

## 4. 階段開發目標 (Milestones)
- **Phase 1** ✅：建立 Python 原型驗證（所有監控模組、AI 腦核、Telegram 通報均完成）。
- **Phase 2** ✅：**Tauri 外殼與 Sidecar 整合**。Python Sidecar 啟動、stdout/stderr 轉發、Rust 背景執行緒輪詢 incidents.json 並即時 emit AI 告警至 React 前端。
- **Phase 3** ✅：**主動緩解增強**。Python `MitigationManager` 自動防禦（清空剪貼簿、終止進程、隔離檔案）與 Rust Commands（kill / isolate / resume / quarantine）均已實作完成。
- **Phase 4** ✅：**編譯與安裝檔封裝 (Production Build)**。已產出 `AI Security Guardian_1.1.0_aarch64.dmg` 與 `.app` bundle（macOS Apple Silicon），`tauri.conf.json` 已設定 `externalBin` 打包 Sidecar binary。

### 已知待改善項目
- `whitelist.rs` 已修正。之前因為 `cleanup_stale_whitelist` 邏輯過於激進導致重啟後清空，現已改為持久化。
- `file_integrity.rs` 已改用 checksum baseline；仍需擴充更多權限不足、檔案不存在與跨平台路徑測試。
- Telegram callback 已加入 HMAC 與 sender 驗證；仍需在真實 Bot 環境做端到端驗證。
- `update_config` 已改為合併更新並抽出路徑解析；仍需在 packaged app 與 Windows sidecar 場景驗證。
- Windows 支援目前為 experimental：安裝器 target 已加入，但 `lsof`、`kill`、`HOME`、LaunchAgents 與 macOS 特定監控仍需平台分支。

### v1.1.1 建議範圍
- 安全補強：config 合併更新、Telegram callback 閉環、process terminate 保護清單。
- 產品化補強：file integrity baseline 管理、UI 操作、差異報告。
- 工程品質：Python / Rust / UI 測試與 CI matrix。
- AI 中期能力只納入薄切片：語義 JSON schema parser 與 config/env 驅動模型選擇；完整多模型路由、OmniParser adapter、rule pack schema 與分散式通報仍排入後續 milestone。

---
*Updated for Current Architecture on 2026-04-28*
