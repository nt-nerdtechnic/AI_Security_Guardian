# Aegis Guardian 發展路線圖 (Roadmap)

本文件概述了 Aegis Guardian 的未來發展方向與計畫。

## 📍 當前進度：Phase 2 - Rust 核心整合中

### 🟢 近期目標 (Short-term)
- [ ] **Rust 核心引擎遷移**：逐步將 Python 監控模塊遷移至 Rust，以實現亞秒級響應速度。
- [ ] **編譯與安裝檔封裝**：實作 macOS (.dmg) 與 Windows (.exe) 的自動化建構流程。
- [ ] **UI 效能優化**：減少前端與 Rust 後端之間的資料傳輸開銷。

### 🟡 中期目標 (Mid-term)
- [ ] **語義過濾器增強**：對齊 NT 資安主權協議，提供更精準的威脅識別。
- [ ] **多模型支援**：優化對 OmniParser-v2.0 與 Qwen2.5-VL 等視覺模型的支援。
- [ ] **外掛系統**：允許第三方開發者自定義監控規則與動作。

### 🔵 長期願景 (Long-term)
- [ ] **分散式通報網絡**：支援多個守門員之間的威脅情報共享。
- [ ] **物理隔離強化**：與專屬硬體結合，提供完全脫網的資安防護方案。

## 🤝 參與貢獻
如果您對以上任何項目感興趣，請查看 [CONTRIBUTING.md](CONTRIBUTING.md) 並尋找帶有 `help wanted` 或 `good first issue` 標籤的 Issue！
