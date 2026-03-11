# Aegis Guardian - 物理架構規格書 v3.0 (Python-first，2026-03-11 整頓)

## 1. 實際物理路徑結構 (Current State)

```text
~/Desktop/AI_Security_Guardian/
├── guardian.py             # [Python] 主協調程序 (監控管理員 + 主入口)
├── guardian_brain.py       # [Python] AI 分析引擎 (直接 import)
├── guardian_core/          # [Python] 共用工具套件
│   ├── i18n.py             #   多語系管理員
│   └── actions/
│       └── mitigation.py   #   程序終止/隔離動作
├── ui/                     # [Tauri + React] 桌面 UI
│   ├── src/                #   React 前端 (JSX 元件)
│   └── src-tauri/          #   Tauri Rust 後端 (IPC + 威脅分析 Command)
├── locales/                # 多語系資源 (zh-TW, zh-CN, en, ja)
├── config.yaml             # 全域資安配置
├── logs/                   # 事件日誌 (incidents.json, snapshot_*.png)
└── requirements.txt        # Python 依賴
```

## 2. 核心運作協議 (Execution Protocol)

1. **[啟動]** `python guardian.py` 作為主背景服務啟動所有監控模組。
2. **[感知]** `ClipboardMonitor`、`ActiveWindowMonitor`、`KeystrokeMonitor`、`NetworkMonitor` 各自在獨立執行緒中監控系統狀態。
3. **[初篩]** 每個監控模組使用 Regex 或規則清單進行第一道過濾。
4. **[AI 判定]** 若 Regex 未命中，`AiBrainClient` 呼叫 `guardian_brain.py` 的分析函數（透過本地 Ollama 推理）進行語義或視覺二次判定。
5. **[存證]** 威脅事件寫入 `logs/incidents.json`（NDJSON 格式），截圖存為 PNG。
6. **[通報]** `TelegramNotifier` 推送告警訊息或截圖至 Telegram。
7. **[UI 即時顯示]** Tauri 後端背景執行緒輪詢 `incidents.json`，即時 emit 事件至 React 前端。

## 3. 技術棧 (Python-first)

| 層級              | 技術               | 職責                        |
|-------------------|--------------------|----------------------------|
| AI 分析           | Python + Ollama    | 視覺/語義威脅判定            |
| 協調層            | Python (threading) | 監控模組管理                |
| UI 後端           | Rust (Tauri)       | IPC 橋接、白名單、進程管理  |
| UI 前端           | React (JSX)        | 儀表板、告警呈現            |
| 多語系            | JSON + I18nManager | zh-TW / zh-CN / en / ja    |
| 通報渠道          | Telegram Bot API   | 遠端告警通知                |

## 4. 開發 SOP

- **Python First**：所有監控邏輯與 AI 分析以 Python 實作為主。
- **Brain as Import**：`guardian_brain.py` 只提供分析函數供 `guardian.py` 直接 import，不啟動 HTTP Server。
- **Tauri 為 UI 橋**：Tauri Rust 後端只負責 UI 所需的資料提供與系統動作（進程終止、檔案隔離），不重複實作監控邏輯。
- **共用日誌**：`logs/incidents.json` 為 Python 監控服務與 Tauri UI 之間的唯一資料共享媒介。
