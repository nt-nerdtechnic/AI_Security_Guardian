# Aegis Guardian - 物理架構規格書 v4.0 (Python-first + Tauri Sidecar)

## 1. 實際物理路徑結構 (Current State - 2026-03-30)

```text
~/Desktop/AI_Security_Guardian/
├── core/
│   ├── main.py             # [Python] 主協調程序 (Sidecar Daemon)
│   ├── models/             #   數據模型與設定
│   │   ├── config.py       #     YAML 配置解析
│   │   ├── i18n.py         #     多語系管理員
│   │   └── incident.py     #     日誌記錄器 (IncidentLogger, NDJSON)
│   ├── viewmodels/         #   核心業務邏輯 (MVVM)
│   │   ├── ai_client.py    #     AI 分析介面 (AiBrainViewModel)
│   │   ├── mitigator.py    #     主動防禦動作 (MitigationManager)
│   │   └── notifier.py     #     Telegram 通報 (TelegramNotifierViewModel)
│   └── monitors/           #   監控模組執行緒
│       ├── clipboard.py    #     剪貼簿監控 (Regex + AI 語義雙層)
│       ├── active_window.py#     視窗監控 (osascript + screencapture)
│       ├── keystroke.py    #     終端指令監控 (pynput)
│       ├── network.py      #     網路監控 (lsof)
│       ├── system_resource.py#   系統資源監控 (CPU/RAM/Disk)
│       └── heartbeat.py    #     健康心跳 (含記憶體洩漏自保)
├── ui/                     # [Tauri + React] 系統外殼與 UI
│   ├── src/                #   React 前端 (儀表板)
│   │   ├── components/     #     UI 元件
│   │   │   ├── ActivityDashboard.jsx  # 事件統計 + 埠口監控 + AI 告警
│   │   │   ├── MitigationPanel.jsx    # 即時威脅進程操作面板
│   │   │   ├── FileIntegrityAlerts.jsx# 系統檔案完整性監控
│   │   │   ├── ModeSwitch.jsx         # 防禦模式切換
│   │   │   └── SettingsPanel.jsx      # 模組開關設定
│   │   ├── viewmodels/     #     React ViewModel (MVVM)
│   │   └── models/tauriApi.js  #  Tauri invoke 統一封裝
│   └── src-tauri/          #   Tauri Rust 後端
│       ├── src/
│       │   ├── main.rs         # 主程式：Commands + Sidecar + IPC + config 合併更新
│       │   ├── network.rs      # NetworkSentinel：lsof 掃描監聽埠
│       │   ├── whitelist.rs    # SQLite 持久化白名單
│       │   ├── file_integrity.rs # SHA-256 baseline + 遞迴目錄完整性偵測
│       │   ├── quarantine.rs   # 檔案隔離移動 (含 cross-device fallback)
│       │   └── process_control.rs # SIGTERM/SIGKILL 進程終止
│       └── binaries/
│           └── aegis-core-daemon-aarch64-apple-darwin  # 已編譯 Sidecar binary
├── guardian_brain.py       # [Python] AI 分析引擎 (llama3 + qwen2.5vl)
├── config.yaml             # 全域資安配置 (含 behavior_firewall / terminal_rules)
├── logs/                   # 事件共享區 (incidents.json, NDJSON)
└── requirements.txt        # Python 依賴
```

## 2. 核心運作協議 (Execution Protocol)

1. **[啟動]** Tauri 啟動後，透過 **Sidecar** 機制執行 `python core/main.py` 作為背景服務。
2. **[感知]** Python 監控模組 (monitors) 分散執行緒，即時攔截剪貼簿、視窗與網路事件。
3. **[AI 判定]** 若規則未命中，Python 呼叫 `AiBrainViewModel` 進行語義或視覺二次判定。
4. **[共享]** 威脅事件統一寫入 `logs/incidents.json`。
5. **[通報]** Python `TelegramNotifierViewModel` 獨立發送遠端告警。
6. **[UI 呈現]** Tauri 後端 (Rust) 輪詢 `incidents.json`，並透過 IPC (`emit`) 推送至 React 前端。
7. **[主動緩解]**
    - **自動**：Python `MitigationManager` 根據配置執行攔截。
    - **手動**：用戶在 UI 點擊動作，觸發 Tauri Rust Command (如 `mitigate_process`)。
8. **[遠端審批]** Telegram Inline callback 需通過 sender 與 HMAC-SHA256 驗證；`terminate` 套用保護清單，`quarantine` 只接受 incidents metadata 中已記錄的檔案路徑。

## 3. 技術棧

| 層級              | 技術                        | 職責                                          |
|-------------------|-----------------------------|-----------------------------------------------|
| AI 分析           | Python + Ollama             | config/env 驅動模型，語義回傳 JSON schema     |
| 核心邏輯層        | Python (MVVM)               | 監控管理、自動防禦、Telegram 通報              |
| UI 殼層           | Rust (Tauri v2)             | Sidecar 管理、SQLite 白名單、系統動作 Commands |
| UI 前端           | React + Vite + Tailwind CSS | 儀表板、即時告警顯示、設定管理                 |
| 資料媒介          | NDJSON (logs/incidents.json)| 跨語言進程資料共享，Rust 以 tail-read 方式輪詢 |
| 白名單持久化      | SQLite (rusqlite, bundled)  | 網路埠口白名單，持久化至 ~/.aegis-guardian/     |
| 完整性 baseline   | JSON + SHA-256              | `~/.aegis-guardian/file_checksums.json`，支援遞迴目錄 hash |
| Callback 驗證     | HMAC-SHA256                 | Telegram 遠端操作簽名與 sender 驗證             |

## 4. Rust Commands 清單

| Command | 功能 |
|---------|------|
| `get_config` | 讀取防禦模式與模組設定 |
| `update_config` | 合併更新設定並寫回 config.yaml，保留 Python 側欄位 |
| `get_incident_stats` | 從 incidents.json 統計威脅數量 |
| `get_real_activities` | 解析 incidents.json 取得分類事件列表 |
| `get_ai_alerts` | 取得歷史 AI_Brain 告警 |
| `get_exposed_ports` | 掃描本地監聽埠（含白名單過濾）|
| `add_network_whitelist` | 新增埠口白名單 |
| `remove_network_whitelist` | 移除埠口白名單 |
| `get_network_whitelist` | 列出已核准埠口 |
| `check_file_integrity` | 以 SHA-256 baseline 偵測系統敏感檔案與目錄變更 |
| `rebuild_file_integrity_baseline` | 重建所有監控路徑的 checksum baseline |
| `accept_file_integrity_change` | 接受單一路徑目前 hash 作為新 baseline |
| `export_file_integrity_report` | 匯出目前完整性檢查報告 |
| `move_to_quarantine` | 隔離可疑檔案至 quarantine/ |
| `terminate_process` | SIGTERM/SIGKILL 終止指定進程 |
| `get_system_resources` | 取得 CPU/RAM/Disk 即時用量 |
| `get_threat_processes` | 取得高 CPU 占用進程列表 |
| `mitigate_process` | kill / isolate (SIGSTOP) / resume (SIGCONT) |

## 5. 開發 SOP

- **Sidecar 優先**：所有核心監控與 AI 判讀保留在 Python 側，方便快速迭代測試。
- **共享為準**：Tauri UI 不直接維護狀態，以 `logs/incidents.json` 為單一事實來源；Rust 以 tail-read 輪詢並 `emit("ai-alert")` 推送前端。
- **權限分工**：需要高權限的系統動作（如進程終止、檔案隔離）交由 Rust Commands 執行。
- **IPC 模式**：Python→Rust 透過 NDJSON 檔案；Rust→React 透過 Tauri `emit` 事件（即時）或 `invoke`（輪詢）。
- **設定寫回**：Rust UI 只管轄 `mode`、`modules`、`file_integrity.custom_paths`，不得覆寫 Python 側安全規則。config 解析優先順序為 `AEGIS_CONFIG_PATH`、repo root fallback、legacy `../config.yaml`。
- **封裝狀態**：macOS 為主要支援平台；Windows `nsis` / `msi` target 已加入，但 sidecar binary 命名、`lsof` / `kill` / LaunchAgents 等平台差異仍為 experimental。
- **AI 設定**：`config.yaml` 的 `ai.ollama_url`、`ai.models.semantic`、`ai.models.visual` 可覆寫預設模型；`ai.rule_packs` 只接受資料型 YAML/JSON rule pack 路徑，尚未載入任意程式碼外掛。
