import React from 'react';

const ModuleToggle = ({ label, active, onToggle, icon }) => {
  return (
    <div className="flex items-center justify-between p-4 bg-slate-800/10 rounded-2xl border border-slate-800/30 transition-colors hover:border-slate-700/50">
      <div className="flex items-center space-x-4">
        <div className={`transition-colors duration-300 ${active ? 'text-emerald-400 drop-shadow-[0_0_5px_rgba(16,185,129,0.5)]' : 'text-slate-600'}`}>{icon}</div>
        <span className={`text-[11px] font-bold uppercase tracking-wider transition-colors duration-300 ${active ? 'text-slate-200' : 'text-slate-600'}`}>{label}</span>
      </div>
      <button 
        onClick={onToggle}
        className={`relative inline-flex h-5 w-9 items-center rounded-full transition-all duration-500 focus:outline-none ${active ? 'bg-emerald-500' : 'bg-slate-700'}`}
      >
        <span className={`inline-block h-3 w-3 transform rounded-full bg-white shadow-md transition-transform duration-300 ${active ? 'translate-x-5' : 'translate-x-1'}`} />
      </button>
    </div>
  );
};

const SettingsPanel = ({ modules, onToggleModule }) => {
  return (
    <div className="bg-slate-900/40 rounded-3xl border border-slate-800/60 p-6 space-y-4 shadow-xl flex-grow">
      <h2 className="flex items-center text-[10px] font-black text-slate-500 uppercase tracking-[0.2em]">
        <span className="w-3 h-3 mr-2 text-blue-500">⚙️</span> Intelligence Nodes
      </h2>
      <div className="space-y-3">
        <ModuleToggle label="Visual Sentry" active={modules.visual} onToggle={() => onToggleModule('visual')} icon={<span>👁️</span>} />
        <ModuleToggle label="Clipboard Guard" active={modules.clipboard} onToggle={() => onToggleModule('clipboard')} icon={<span>📋</span>} />
        <ModuleToggle label="Network Shield" active={modules.network} onToggle={() => onToggleModule('network')} icon={<span>🌐</span>} />
      </div>
      
      <div className="mt-8 p-4 bg-slate-950/50 rounded-2xl border border-slate-800/50">
          <div className="flex items-center space-x-2 text-[9px] text-slate-600 font-bold uppercase mb-2">
              <span>Background Sync</span>
          </div>
          <div className="flex items-center space-x-2">
              <div className="w-1 h-1 bg-emerald-500 rounded-full animate-ping" />
              <span className="text-[10px] font-mono text-slate-400">guardian.py: ONLINE</span>
          </div>
      </div>
    </div>
  );
};

export default SettingsPanel;
