# 貢獻指南 (Contributing Guide)

歡迎對 Aegis Guardian 做出貢獻！以下是本地開發環境的建置說明。

## 本地開發環境建置

### 後端 Python 引擎

```bash
# 1. 建立虛擬環境
python3 -m venv venv
source venv/bin/activate

# 2. 安裝依賴
pip install -r requirements.txt

# 3. 確認 Ollama 已安裝並執行，然後安裝所需模型
./install_models.sh

# 4. 啟動後端
python core/main.py
```

### 前端 UI（Tauri + React）

```bash
cd ui

# 安裝 Node.js 依賴
npm install

# 啟動開發伺服器（熱更新）
npm run dev
```

> **注意**：Tauri 的 Rust 後端需要安裝 Rust 工具鏈。請參考 [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites/) 進行安裝。若只需修改前端 React 元件，直接執行 `npm run dev` 在瀏覽器中開發即可。

## 提交規範 (Commit Convention)

請遵循 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
feat: 新增剪貼簿正則規則支援
fix: 修正視覺模型回傳結果解析錯誤
docs: 更新 README 安裝步驟
refactor: 重構 NetworkMonitor 模組
```

## 如何新增語言支援 (i18n)

在 `locales/` 目錄下，複製現有的 `zh-TW.json`，修改對應的字串並以目標語系命名（例如 `de.json`），然後在 `core/i18n.py` 中加入對應的語系代碼。

## 如何新增行為規則

1. 開啟 `config.yaml`
2. 在 `behavior_firewall.regex_rules` 下新增規則名稱與正規表達式
3. 在 `terminal_rules.high_risk_keywords` 下新增關鍵字

不需要修改任何程式碼，直接更動 `config.yaml` 後重啟 `core/main.py` 即可生效。

## 如何尋找任務

我們使用 GitHub Labels 來標示適合貢獻的 Issue：
- `good first issue`：適合對專案不熟悉的新手，通常是簡單的 bug 修復或文檔更新。
- `help wanted`：維護者目前無法獨自處理，需要社群支援的任務。

在開始工作前，請先在對應的 Issue 下留言以避免重複勞動。

## Bug 回報

請至 GitHub Issues 開啟新的 Issue，並包含：
- 作業系統版本
- Python / Ollama 版本
- 重現步驟
- 錯誤日誌（可從 `logs/incidents.json` 取得）
