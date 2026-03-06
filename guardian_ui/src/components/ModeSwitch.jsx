import React from 'react';

const ModeSwitch = ({ modes, currentMode, onModeChange }) => {
  return (
    <div className="bg-slate-900/40 rounded-3xl border border-slate-800/60 p-6 space-y-4 shadow-xl">
      <h2 className="flex items-center text-[10px] font-black text-slate-500 uppercase tracking-[0.2em]">
        <span className="w-3 h-3 mr-2 text-emerald-500">🛡️</span> Protection Level
      </h2>
      <div className="space-y-2">
        <p className="text-[9px] text-slate-500 font-medium mb-4 italic leading-relaxed border-l border-slate-800 pl-3">
          Select protection level to define how strictly the system handles detected anomalies.
        </p>
        {modes.map(m => (
          <div key={m} className="space-y-1">
            <button
              onClick={() => onModeChange(m)}
              className={`w-full py-3 px-4 rounded-2xl text-[11px] font-bold transition-all duration-300 border uppercase tracking-widest ${
                currentMode === m 
                ? 'bg-emerald-500/10 border-emerald-500/40 text-emerald-400 shadow-[0_0_20px_rgba(16,185,129,0.05)] translate-x-1' 
                : 'bg-slate-800/20 border-transparent text-slate-500 hover:bg-slate-800/40 hover:text-slate-400'
              }`}
            >
              {m}
            </button>
            {currentMode === m && (
              <p className="text-[8px] text-emerald-500/60 font-medium px-4 py-1 italic">
                {m === 'Silent Monitor' && 'Logging anomalies only. No intervention.'}
                {m === 'Advisory' && 'Prompting user for risky actions.'}
                {m === 'Strict Blocking' && 'Automatically terminating high-risk processes.'}
              </p>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

export default ModeSwitch;
