# Aegis Guardian 發展路線圖 (Roadmap)

本文件概述了 Aegis Guardian 的未來發展方向與計畫。

## 📍 當前進度：Phase 4 完成 - v1.1.0 已發布

### ✅ 已完成里程碑
- [x] **Phase 1**：Python 監控原型（6 個監控模組、AI 腦核、Telegram 通報）
- [x] **Phase 2**：Tauri v2 外殼與 Sidecar 整合（15 個 Rust Commands、IPC 即時推送）
- [x] **Phase 3**：主動緩解增強（Python MitigationManager + Rust kill/isolate/quarantine）
- [x] **Phase 4**：macOS .dmg 安裝包封裝（`AI Security Guardian_1.1.0_aarch64.dmg`）

### 🔴 緊急 Bug 修復 (Hotfix)
- [x] **白名單重啟清空問題**：修正 `cleanup_stale_whitelist` 邏輯，從過於激進的自動清理改為持久化，確保服務重開後白名單依然有效。
- [x] **config.yaml 覆寫問題**：`update_config` 已改為合併更新，只寫入 `mode`、`modules`、`file_integrity.custom_paths`，並保留 Python 側 `behavior_firewall` / `terminal_rules` / `network_monitor` 等欄位；config 路徑解析支援 `AEGIS_CONFIG_PATH`、repo root fallback 與 reload marker 同目錄寫入。
- [x] **Telegram callback 安全閉環**：callback 已加入 sender 驗證與 HMAC-SHA256 簽名；`terminate` 加入 PID 0/1、自身程序與核心程序保護，成功/失敗會寫入 `logs/incidents.json`；`quarantine` 只允許處理 incidents metadata 中已記錄的檔案路徑。

### 🟢 近期目標 (Short-term)
- [x] **檔案完整性強化**：`file_integrity.rs` 已加入 SHA-256 baseline、遞迴目錄 hash、重建 baseline、接受單一變更與匯出差異報告的 UI 操作。
- [ ] **Windows 封裝**：`tauri.conf.json` 已加入 `nsis` / `msi` targets，CI 已加入 macOS/Windows Rust check；Windows sidecar binary 與平台差異仍列為 experimental。
- [ ] **v1.1.1 Hotfix 發布**：建議範圍限於安全補強、測試、CI 與文件同步，不混入語義模型、外掛系統等中期功能。

### 🟡 中期目標 (Mid-term)
- [x] **語義過濾器增強（薄切片）**：`guardian_brain.py` 已新增 JSON schema parser，輸出 `verdict`、`confidence`、`category`、`reason`、`recommended_action`，並保留舊版 YES/NO 相容解析。
- [x] **多模型支援（薄切片）**：Ollama URL、語義模型、視覺模型已可由 `config.yaml` 的 `ai.models` 或 `AEGIS_*` 環境變數驅動；OmniParser-v2.0 adapter 仍列為後續研究。
- [ ] **外掛系統**：先保留 `ai.rule_packs` 的資料型 rule pack 入口，不載入任意程式碼；後續需定義 YAML/JSON rule pack schema 與驗證器。

### 🔵 長期願景 (Long-term)
- [ ] **分散式通報網絡**：支援多個守門員之間的威脅情報共享。
- [ ] **物理隔離強化**：與專屬硬體結合，提供完全脫網的資安防護方案。

## 🤝 參與貢獻
如果您對以上任何項目感興趣，請查看 [CONTRIBUTING.md](CONTRIBUTING.md) 並尋找帶有 `help wanted` 或 `good first issue` 標籤的 Issue！
