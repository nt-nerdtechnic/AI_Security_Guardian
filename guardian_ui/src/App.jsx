import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Shield, RefreshCw, Sun, Moon, Globe } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import ModeSwitch from './components/ModeSwitch';
import SettingsPanel from './components/SettingsPanel';
import ActivityDashboard from './components/ActivityDashboard';
import { useLanguage } from './i18n/LanguageContext';
import { LOCALES } from './i18n/index.js';

function App() {
  const { language, cycleLang, t } = useLanguage();
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
  const [events, setEvents]   = useState({ threats: [], keys: [], visual: [] });
  const [loading, setLoading] = useState(true);
  const [darkMode, setDarkMode] = useState(() => {
    const saved = localStorage.getItem('aegis-theme');
    return saved ? saved === 'dark' : true; // 預設 dark
  });

  // 語系對應的 mode 名稱（對應 config 內部 key 用英文，顯示用語系字串）
  const MODE_KEYS = ['Silent Monitor', 'Advisory', 'Strict Blocking'];
  const MODE_DISPLAY = {
    'Silent Monitor': t('modes.silent_monitor'),
    'Advisory':       t('modes.advisory'),
    'Strict Blocking': t('modes.strict_blocking'),
  };



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

  const fetchStats = useCallback(async () => {
    try {
      const realData = await invoke('get_real_activities');
      setStats(realData.stats);
      
      const sortDesc = (a, b) => b._ts - a._ts;
      setEvents({
        threats: realData.threats.sort(sortDesc).slice(0, 50),
        keys:    realData.keys.sort(sortDesc).slice(0, 50),
        visual:  realData.visual.sort(sortDesc).slice(0, 50),
      });
    } catch (e) {
      console.error("Failed to fetch stats", e);
    }
  }, [t]);

  useEffect(() => {
    initApp();
    const interval = setInterval(fetchStats, 5000);
    return () => clearInterval(interval);
  }, [fetchStats]);

  useEffect(() => {
    localStorage.setItem('aegis-theme', darkMode ? 'dark' : 'light');
  }, [darkMode]);

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

  // ── Theme tokens ───────────────────────────────────────────────
  const th = darkMode ? {
    bg:           'bg-slate-950',
    surface:      'bg-slate-900/40',
    border:       'border-slate-800/60',
    text:         'text-slate-100',
    textMuted:    'text-slate-500',
    statusBg:     'bg-slate-900/50',
    statusBorder: 'border-slate-800',
    toggleBg:     'bg-slate-800 hover:bg-slate-700',
    toggleText:   'text-amber-400',
    footerText:   'text-slate-700',
    footerDot:    'bg-slate-800',
    footerSub:    'text-slate-600',
  } : {
    bg:           'bg-slate-100',
    surface:      'bg-white/80',
    border:       'border-slate-200',
    text:         'text-slate-900',
    textMuted:    'text-slate-400',
    statusBg:     'bg-white/70',
    statusBorder: 'border-slate-300',
    toggleBg:     'bg-slate-200 hover:bg-slate-300',
    toggleText:   'text-slate-700',
    footerText:   'text-slate-400',
    footerDot:    'bg-slate-300',
    footerSub:    'text-slate-400',
  };

  // 目前語言的顯示名稱
  const currentLangName = LOCALES[language]?.lang_name ?? language;

  if (loading) {
    return (
      <div className={`flex items-center justify-center h-screen ${th.bg} text-emerald-500`}>
        <RefreshCw className="animate-spin w-8 h-8" />
      </div>
    );
  }

  return (
    <div className={`flex flex-col min-h-screen ${th.bg} p-6 space-y-6 ${th.text} overflow-y-auto font-sans transition-colors duration-300`}>
      {/* Header */}
      <div className="flex justify-between items-center">
        <div className="flex items-center space-x-3">
          <Shield className="text-emerald-500 w-8 h-8" />
          <h1 className="text-2xl font-bold tracking-tight">{t('app.title')}</h1>
        </div>

        <div className="flex items-center space-x-3">
          {/* 語言切換按鈕 */}
          <button
            onClick={cycleLang}
            className={`flex items-center space-x-1.5 px-3 py-1.5 rounded-full ${th.toggleBg} text-emerald-400 transition-all duration-300 shadow-sm text-[10px] font-bold uppercase tracking-widest`}
            title={t('app.language_button_title')}
          >
            <Globe className="w-3.5 h-3.5" />
            <span>{currentLangName}</span>
          </button>

          {/* Dark / Light 切換按鈕 */}
          <button
            onClick={() => setDarkMode(prev => !prev)}
            className={`p-2 rounded-full ${th.toggleBg} ${th.toggleText} transition-all duration-300 shadow-sm`}
            title={darkMode ? t('app.toggle_light') : t('app.toggle_dark')}
          >
            {darkMode ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
          </button>

          {/* 狀態指示器 */}
          <div className={`flex items-center space-x-2 ${th.statusBg} px-3 py-1 rounded-full border ${th.statusBorder} shadow-lg shadow-emerald-500/5`}>
            <div className={`w-2 h-2 rounded-full animate-pulse ${mode === 'Strict Blocking' ? 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.8)]' : 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.8)]'}`} />
            <span className={`text-[10px] font-bold ${th.textMuted} uppercase tracking-widest`}>{MODE_DISPLAY[mode] ?? mode}</span>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6 flex-grow">
        {/* Left Column: Dashboard Stats */}
        <div className="md:col-span-3 flex flex-col space-y-6 pr-2">
          <ActivityDashboard stats={stats} events={events} darkMode={darkMode} t={th} />
        </div>

        {/* Right Column: Sidebar Controls */}
        <div className="space-y-6 flex flex-col h-full">
          <ModeSwitch modes={MODE_KEYS} currentMode={mode} onModeChange={handleModeChange} darkMode={darkMode} t={th} />
          <SettingsPanel modules={modules} onToggleModule={toggleModule} darkMode={darkMode} t={th} />
        </div>
      </div>

      {/* Footer */}
      <div className={`text-[9px] ${th.footerText} font-bold uppercase tracking-[0.3em] text-center pt-2 flex items-center justify-center space-x-4`}>
        <span>{t('app.footer_unit')}</span>
        <span className={`w-1 h-1 ${th.footerDot} rounded-full`} />
        <span className={`${th.footerSub} italic`}>v1.1.0-Aegis</span>
        <span className={`w-1 h-1 ${th.footerDot} rounded-full`} />
        <span>{t('app.footer_brand')}</span>
      </div>
    </div>
  );
}

export default App;
