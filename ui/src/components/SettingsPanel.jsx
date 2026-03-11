import React from 'react';
import { useLanguage } from '../i18n/LanguageContext';

const ModuleToggle = ({ label, active, onToggle, icon, darkMode }) => {
  return (
    <div className={`flex items-center justify-between p-4 ${darkMode ? 'bg-slate-800/10 border-slate-800/30 hover:border-slate-700/50' : 'bg-slate-50 border-slate-200 hover:border-slate-300'} rounded-2xl border transition-colors`}>
      <div className="flex items-center space-x-4">
        <div className={`transition-colors duration-300 ${active ? 'text-emerald-400 drop-shadow-[0_0_5px_rgba(16,185,129,0.5)]' : darkMode ? 'text-slate-600' : 'text-slate-400'}`}>{icon}</div>
        <span className={`text-[11px] font-bold uppercase tracking-wider transition-colors duration-300 ${active ? (darkMode ? 'text-slate-200' : 'text-slate-700') : (darkMode ? 'text-slate-600' : 'text-slate-400')}`}>{label}</span>
      </div>
      <button
        onClick={onToggle}
        className={`relative inline-flex h-5 w-9 items-center rounded-full transition-all duration-500 focus:outline-none ${active ? 'bg-emerald-500' : darkMode ? 'bg-slate-700' : 'bg-slate-300'}`}
      >
        <span className={`inline-block h-3 w-3 transform rounded-full bg-white shadow-md transition-transform duration-300 ${active ? 'translate-x-5' : 'translate-x-1'}`} />
      </button>
    </div>
  );
};

const SettingsPanel = ({ modules, onToggleModule, darkMode, t: theme }) => {
  const { t } = useLanguage();
  return (
    <div className={`${theme.surface} rounded-3xl border ${theme.border} p-6 space-y-4 shadow-xl flex-grow transition-colors duration-300`}>
      <h2 className={`flex items-center text-[10px] font-black ${theme.textMuted} uppercase tracking-[0.2em]`}>
        <span className="w-3 h-3 mr-2 text-blue-500">⚙️</span> {t('settings.panel_title')}
      </h2>
      <div className="space-y-3">
        <ModuleToggle label={t('settings.visual_sentry')}   active={modules.visual}     onToggle={() => onToggleModule('visual')}     icon={<span>👁️</span>}  darkMode={darkMode} />
        <ModuleToggle label={t('settings.clipboard_guard')} active={modules.clipboard}  onToggle={() => onToggleModule('clipboard')}  icon={<span>📋</span>}  darkMode={darkMode} />
        <ModuleToggle label={t('settings.network_shield')}  active={modules.network}    onToggle={() => onToggleModule('network')}    icon={<span>🌐</span>}  darkMode={darkMode} />
      </div>

      <div className={`mt-8 p-4 ${darkMode ? 'bg-slate-950/50 border-slate-800/50' : 'bg-slate-100 border-slate-200'} rounded-2xl border`}>
        <div className={`flex items-center space-x-2 text-[9px] ${theme.textMuted} font-bold uppercase mb-2`}>
          <span>{t('settings.background_sync')}</span>
        </div>
        <div className="flex items-center space-x-2">
          <div className="w-1 h-1 bg-emerald-500 rounded-full animate-ping" />
          <span className={`text-[10px] font-mono ${darkMode ? 'text-slate-400' : 'text-slate-500'}`}>{t('settings.guardian_online')}</span>
        </div>
      </div>
    </div>
  );
};

export default SettingsPanel;
