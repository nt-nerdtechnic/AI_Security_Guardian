import React, { useState, useEffect } from 'react';
import { Shield, RefreshCw } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import ModeSwitch from './components/ModeSwitch';
import SettingsPanel from './components/SettingsPanel';
import ActivityDashboard from './components/ActivityDashboard';

function App() {
  const [mode, setMode] = useState('Silent Monitor');
  const [modules, setModules] = useState({
    visual: true,
    clipboard: true,
    network: true
  });
  const [stats, setStats] = useState({
    total_blocked: 0,
    sensitive_keys: 0,
    visual_alerts: 0
  });
  const [loading, setLoading] = useState(true);

  const modes = ['Silent Monitor', 'Advisory', 'Strict Blocking'];

  useEffect(() => {
    initApp();
    const interval = setInterval(fetchStats, 5000);
    return () => clearInterval(interval);
  }, []);

  const initApp = async () => {
    setLoading(true);
    await fetchConfig();
    await fetchStats();
    setLoading(false);
  };

  const fetchConfig = async () => {
    try {
      const config = await invoke('get_config');
      setMode(config.mode);
      setModules(config.modules);
    } catch (e) {
      console.error("Failed to fetch config", e);
    }
  };

  const fetchStats = async () => {
    try {
      const logData = await invoke('get_incident_stats');
      setStats(logData);
    } catch (e) {
      console.error("Failed to fetch stats", e);
    }
  };

  const updateConfig = async (newMode, newModules) => {
    try {
      await invoke('update_config', { mode: newMode, modules: newModules });
    } catch (e) {
      console.error("Failed to update config", e);
    }
  };

  const handleModeChange = (m) => {
    setMode(m);
    updateConfig(m, modules);
  };

  const toggleModule = (mod) => {
    const next = { ...modules, [mod]: !modules[mod] };
    setModules(next);
    updateConfig(mode, next);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen bg-slate-950 text-emerald-500">
        <RefreshCw className="animate-spin w-8 h-8" />
      </div>
    );
  }

  return (
    <div className="flex flex-col h-screen bg-slate-950 p-6 space-y-6 text-slate-100 overflow-hidden font-sans">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div className="flex items-center space-x-3">
          <Shield className="text-emerald-500 w-8 h-8" />
          <h1 className="text-2xl font-bold tracking-tight">Aegis Guardian</h1>
        </div>
        <div className="flex items-center space-x-2 bg-slate-900/50 px-3 py-1 rounded-full border border-slate-800 shadow-lg shadow-emerald-500/5">
          <div className={`w-2 h-2 rounded-full animate-pulse ${mode === 'Strict Blocking' ? 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.8)]' : 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.8)]'}`} />
          <span className="text-[10px] font-bold text-slate-300 uppercase tracking-widest">{mode}</span>
        </div>
      </div>

      {/* Main Content */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6 flex-grow overflow-hidden">
        {/* Left Column: Dashboard Stats */}
        <div className="md:col-span-3 flex flex-col space-y-6 overflow-y-auto pr-2">
          <ActivityDashboard stats={stats} />
        </div>

        {/* Right Column: Sidebar Controls */}
        <div className="space-y-6 flex flex-col h-full">
          <ModeSwitch modes={modes} currentMode={mode} onModeChange={handleModeChange} />
          <SettingsPanel modules={modules} onToggleModule={toggleModule} />
        </div>
      </div>

      {/* Footer */}
      <div className="text-[9px] text-slate-700 font-bold uppercase tracking-[0.3em] text-center pt-2 flex items-center justify-center space-x-4">
        <span>Autonomous R&D Unit</span>
        <span className="w-1 h-1 bg-slate-800 rounded-full" />
        <span className="text-slate-600 italic">v1.1.0-Aegis</span>
        <span className="w-1 h-1 bg-slate-800 rounded-full" />
        <span>NerdTechnic</span>
      </div>
    </div>
  );
}

export default App;