# Aegis Guardian - 物理架構規格書 v3.1 (Python-first + Tauri Sidecar)

## 1. 實際物理路徑結構 (Current State - 2026-03-13)

```text
~/Desktop/AI_Security_Guardian/
├── core/
│   ├── main.py             # [Python] 主協調程序 (Sidecar Daemon)
│   ├── models/             #   數據模型與設定
│   │   ├── config.py       #     YAML 配置解析
│   │   ├── i18n.py         #     多語系管理員
│   │   └── incident.py     #     日誌記錄器 (IncidentLogger)
│   ├── viewmodels/         #   核心業務邏輯 (MVVM)
│   │   ├── ai_client.py    #     AI 分析介面 (AiBrainViewModel)
│   │   ├── mitigator.py    #     主動防禦動作 (MitigationManager)
│   │   └── notifier.py     #     Telegram 通報 (TelegramNotifierViewModel)
│   └── monitors/           #   監控模組執行緒
│       ├── clipboard.py    #     剪貼簿監控
│       ├── active_window.py#     視窗監控
│       ├── network.py      #     網路監控
│       └── ...
├── ui/                     # [Tauri + React] 系統外殼與 UI
│   ├── src/                #   React 前端 (儀表板)
│   └── src-tauri/          #   Tauri Rust 後端 (Sidecar 管理 + 系統動作)
├── guardian_brain.py       # [Python] AI 分析引擎實作
├── config.yaml             # 全域資安配置
├── logs/                   # 事件共享區 (incidents.json)
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

## 3. 技術棧

| 層級              | 技術               | 職責                        |
|-------------------|--------------------|----------------------------|
| AI 分析           | Python + Ollama    | 視覺/語義威脅判定            |
| 核心邏輯層        | Python (MVVM)      | 監控管理、自動防禦、通報     |
| UI 殼層           | Rust (Tauri)       | Sidecar 管理、系統權限操作    |
| UI 前端           | React + Vite       | 儀表板、即時告警顯示         |
| 資料媒介          | NDJSON (logs)      | 跨語言進程資料共享           |

## 4. 開發 SOP

- **Sidecar 優先**：所有核心監控與 AI 判讀應保留在 Python 側，方便快速迭代測試。
- **共享為準**：Tauri UI 不應直接維護狀態，應以 `logs/incidents.json` 的內容作為單一事實來源。
- **權限分工**：需要高權限的系統動作（如 macOS TCC 授權動作）優先考慮交由 Rust 或特定 Sidecar 執行。
