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

## 3. 技術目標 (Technical Goals)
- **低佔用**：平常以靜默進程存在，僅在偵測到變動時喚醒 AI 判定。
- **純本地**：所有視覺與語義判定均由本地推理引擎完成，數據不出機。

## 4. 交付物 (Deliverables)
- `guardian.py`：核心啟動腳本。
- `config.yaml`：資安攔截規則配置。
- `logs/`：治安事件紀錄與證據快照。

---
*Generated for Standalone Development by Nerty on 2026-03-03*
