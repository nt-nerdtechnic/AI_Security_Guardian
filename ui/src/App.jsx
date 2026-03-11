import React from 'react';
import { Shield, RefreshCw, Sun, Moon, Globe } from 'lucide-react';

import ModeSwitch from './components/ModeSwitch';
import SettingsPanel from './components/SettingsPanel';
import ActivityDashboard from './components/ActivityDashboard';

// I18n
import { useLanguage } from './i18n/LanguageContext';
import { LOCALES } from './i18n/index.js';

// ViewModels (MVVM)
import { useGuardianState } from './viewmodels/useGuardianState';
import { useConfig } from './viewmodels/useConfig';
import { useTheme } from './viewmodels/useTheme';

function App() {
  // [View] 主要負責 UI 渲染，所有邏輯與狀態管理皆委派給 ViewModels
  
  const { language, cycleLang, t } = useLanguage();
  const currentLangName = LOCALES[language]?.lang_name ?? language;

  const { darkMode, toggleTheme, th } = useTheme();
  
  const { 
    mode, modules, configLoading, 
    handleModeChange, toggleModule 
  } = useConfig();
  
  const { stats, events, sysResources, loading: stateLoading } = useGuardianState(t);

  const isLoading = configLoading || stateLoading;

  const MODE_KEYS = ['Silent Monitor', 'Advisory', 'Strict Blocking'];
  const MODE_DISPLAY = {
    'Silent Monitor': t('modes.silent_monitor'),
    'Advisory':       t('modes.advisory'),
    'Strict Blocking': t('modes.strict_blocking'),
  };

  if (isLoading) {
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
          {/* Language Switch */}
          <button
            onClick={cycleLang}
            className={`flex items-center space-x-1.5 px-3 py-1.5 rounded-full ${th.toggleBg} text-emerald-400 transition-all duration-300 shadow-sm text-[10px] font-bold uppercase tracking-widest`}
            title={t('app.language_button_title')}
          >
            <Globe className="w-3.5 h-3.5" />
            <span>{currentLangName}</span>
          </button>

          {/* Theme Switch */}
          <button
            onClick={toggleTheme}
            className={`p-2 rounded-full ${th.toggleBg} ${th.toggleText} transition-all duration-300 shadow-sm`}
            title={darkMode ? t('app.toggle_light') : t('app.toggle_dark')}
          >
            {darkMode ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
          </button>

          {/* Status Indicator */}
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
          <ActivityDashboard stats={stats} events={events} sysResources={sysResources} darkMode={darkMode} t={th} />
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
        <span className={`${th.footerSub} italic`}>v2.0.0-MVVM</span>
        <span className={`w-1 h-1 ${th.footerDot} rounded-full`} />
        <span>{t('app.footer_brand')}</span>
      </div>
    </div>
  );
}

export default App;
