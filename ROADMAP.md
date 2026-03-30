# Aegis Guardian 發展路線圖 (Roadmap)

本文件概述了 Aegis Guardian 的未來發展方向與計畫。

## 📍 當前進度：Phase 4 完成 - v1.1.0 已發布

### ✅ 已完成里程碑
- [x] **Phase 1**：Python 監控原型（6 個監控模組、AI 腦核、Telegram 通報）
- [x] **Phase 2**：Tauri v2 外殼與 Sidecar 整合（15 個 Rust Commands、IPC 即時推送）
- [x] **Phase 3**：主動緩解增強（Python MitigationManager + Rust kill/isolate/quarantine）
- [x] **Phase 4**：macOS .dmg 安裝包封裝（`AI Security Guardian_1.1.0_aarch64.dmg`）

### 🔴 緊急 Bug 修復 (Hotfix)
- [ ] **白名單重啟清空問題**：`whitelist.rs` 的 `DROP TABLE` 應改為 `CREATE TABLE IF NOT EXISTS`，避免每次重啟清除使用者白名單設定。
- [ ] **config.yaml 覆寫問題**：`update_config` Rust Command 會破壞 Python 側的 `behavior_firewall` / `terminal_rules` 欄位，需改為合併更新而非全量覆寫。

### 🟢 近期目標 (Short-term)
- [ ] **檔案完整性強化**：`file_integrity.rs` 加入 SHA-256 checksum 比對，取代「24 小時修改」的粗糙判斷，降低誤報率。
- [ ] **Telegram 安全驗證**：callback 處理加入 HMAC-SHA256 簽名驗證，防止偽造 callback 觸發遠端指令。
- [ ] **Windows 封裝**：補完 Windows `.exe` / `.msi` 的自動化建構流程。

### 🟡 中期目標 (Mid-term)
- [ ] **語義過濾器增強**：提供更精準的威脅識別，減少 AI 分析誤報。
- [ ] **多模型支援**：優化對 OmniParser-v2.0 與 Qwen2.5-VL 等視覺模型的支援。
- [ ] **外掛系統**：允許第三方開發者自定義監控規則與動作。

### 🔵 長期願景 (Long-term)
- [ ] **分散式通報網絡**：支援多個守門員之間的威脅情報共享。
- [ ] **物理隔離強化**：與專屬硬體結合，提供完全脫網的資安防護方案。

## 🤝 參與貢獻
如果您對以上任何項目感興趣，請查看 [CONTRIBUTING.md](CONTRIBUTING.md) 並尋找帶有 `help wanted` 或 `good first issue` 標籤的 Issue！
