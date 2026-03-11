import { invoke } from '@tauri-apps/api/core';

/**
 * Tauri API Wrapper (Model)
 * 統一管理所有的 Rust Backend 呼叫，隔離依賴。
 */
export const TauriApi = {
  getConfig: () => invoke('get_config'),
  
  updateConfig: (mode, modules) => invoke('update_config', { mode, modules }),
  
  getRealActivities: () => invoke('get_real_activities'),
  
  // (如有其他 API 也統一放這)
};
