import React from 'react';
import { useLanguage } from '../i18n/LanguageContext';

const MODE_KEYS = ['Silent Monitor', 'Advisory', 'Strict Blocking'];

const ModeSwitch = ({ modes, currentMode, onModeChange, darkMode, t: theme }) => {
  const { t } = useLanguage();

  const modeLabel = {
    'Silent Monitor':  t('modes.silent_monitor'),
    'Advisory':        t('modes.advisory'),
    'Strict Blocking': t('modes.strict_blocking'),
  };

  const modeDesc = {
    'Silent Monitor':  t('modes.silent_desc'),
    'Advisory':        t('modes.advisory_desc'),
    'Strict Blocking': t('modes.strict_desc'),
  };

  return (
    <div className={`${theme.surface} rounded-3xl border ${theme.border} p-6 space-y-4 shadow-xl transition-colors duration-300`}>
      <h2 className={`flex items-center text-[10px] font-black ${theme.textMuted} uppercase tracking-[0.2em]`}>
        <span className="w-3 h-3 mr-2 text-emerald-500">🛡️</span> {t('modes.panel_title')}
      </h2>
      <div className="space-y-2">
        <p className={`text-[9px] ${theme.textMuted} font-medium mb-4 italic leading-relaxed border-l ${darkMode ? 'border-slate-800' : 'border-slate-300'} pl-3`}>
          {t('modes.panel_desc')}
        </p>
        {modes.map(m => (
          <div key={m} className="space-y-1">
            <button
              onClick={() => onModeChange(m)}
              className={`w-full py-3 px-4 rounded-2xl text-[11px] font-bold transition-all duration-300 border uppercase tracking-widest ${
                currentMode === m
                ? 'bg-emerald-500/10 border-emerald-500/40 text-emerald-400 shadow-[0_0_20px_rgba(16,185,129,0.05)] translate-x-1'
                : darkMode
                  ? 'bg-slate-800/20 border-transparent text-slate-500 hover:bg-slate-800/40 hover:text-slate-400'
                  : 'bg-slate-100 border-transparent text-slate-400 hover:bg-slate-200 hover:text-slate-600'
              }`}
            >
              {modeLabel[m] ?? m}
            </button>
            {currentMode === m && (
              <p className="text-[8px] text-emerald-500/60 font-medium px-4 py-1 italic">
                {modeDesc[m]}
              </p>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

export default ModeSwitch;
