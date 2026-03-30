import { invoke } from '@tauri-apps/api/core';

/**
 * Tauri API Wrapper (Model)
 * 統一管理所有的 Rust Backend 呼叫，隔離依賴。
 */
export const TauriApi = {
  getConfig: () => invoke('get_config'),
  
  updateConfig: (mode, modules, fileIntegrity) => invoke('update_config', { mode, modules, fileIntegrity }),
  
  getRealActivities: () => invoke('get_real_activities'),
  
  getSystemResources: () => invoke('get_system_resources'),
  
  getThreatProcesses: () => invoke('get_threat_processes'),
  
  mitigateProcess: (pid, action) => invoke('mitigate_process', { pid, action }),

  // Ops Center — read
  opsListSystemCrontab: () => invoke('ops_list_system_crontab'),
  opsListLaunchAgents: () => invoke('ops_list_launch_agents'),
  opsListActiveSessions: () => invoke('ops_list_active_sessions'),

  // Ops Center — actions
  opsKillSession:         (pid) => invoke('ops_kill_session', { pid }),
  opsToggleLaunchAgent:   (plistPath, loaded) => invoke('ops_toggle_launch_agent', { plistPath, loaded }),
  opsDeleteLaunchAgent:   (plistPath) => invoke('ops_delete_launch_agent', { plistPath }),
  opsToggleCrontab:       (raw) => invoke('ops_toggle_crontab', { raw }),
  opsDeleteCrontab:       (raw) => invoke('ops_delete_crontab', { raw }),

  // (如有其他 API 也統一放這)
};
