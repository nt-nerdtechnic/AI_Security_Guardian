# Aegis Guardian - 物理架構規格書 v2.0 (對齊 2026-03-04 指令)

## 1. 物理路徑結構 (Folder Tree)
```text
~/Desktop/AI_Security_Guardian/
├── guardian_core/          # [Rust] 核心鋼鐵外骨骼 (Kernel)
│   ├── Cargo.toml          # 依賴定義 (pyo3, tokio, sysinfo)
│   ├── src/
│   │   ├── main.rs         # 程式入口：啟動背景服務 (Daemon)
│   │   ├── interceptor/    # 攔截模組 (網絡、進程、視窗)
│   │   ├── sensor/         # 採集模組 (物理截圖、鍵盤、剪貼簿)
│   │   └── bridge/         # PyO3 橋接：呼叫 Python AI 大腦
├── guardian_brain/         # [Python] AI 語義大腦 (Brain)
│   ├── brain_server.py     # 接收 Rust 傳來的數據進行 AI 判定
│   ├── models/             # 本地權重 (OmniParser-v2.0, Florence-2)
│   └── rules/              # 治安規則庫 (Regex, YARA)
├── config.yaml             # 全域資安配置
├── logs/                   # 治安事件簿 (JSON + 證據截圖)
└── scripts/                # 打包與部署腳本 (.dmg / .exe)
```

## 2. 核心運作協議 (Execution Protocol)
1. **[感知] Rust Sensor** 每秒 2 次掃描物理狀態 (螢幕/剪貼簿/進程)。
2. **[初篩] Rust Core** 過濾無效異動。若發現「疑似風險」，將封包傳送至 **Bridge**。
3. **[判定] Python Brain** (透過 PyO3 內嵌或 IPC) 執行 AI 語義分析。
4. **[執行] Rust Interceptor** 根據判定結果執行「硬攔截」(Kill Process / Redact Text / Block Net)。
5. **[存證]** 全自動產出 JSON 治安日誌並回傳物理網關。

## 3. 開發 SOP (監軍執行)
- **Rust First**：所有物理監控邏輯必須寫在 `guardian_core` 之下。
- **Brain as Service**：Python 僅負責「大腦級」決策邏輯。
- **No Direct Logic**：禁止在 Brain 層直接執行系統級修改，統一交由 Core 執行。
