import React from 'react';

const ActivityDashboard = ({ stats }) => {
  return (
    <div className="flex flex-col space-y-6 flex-grow overflow-hidden">
      <div className="grid grid-cols-3 gap-4">
        <StatCard 
          title="Threats Blocked" 
          value={stats.total_blocked} 
          icon={<span className="text-amber-500">⚠️</span>} 
        />
        <StatCard 
          title="Keys Redacted" 
          value={stats.sensitive_keys} 
          icon={<span className="text-emerald-500">📋</span>} 
        />
        <StatCard 
          title="Visual Alerts" 
          value={stats.visual_alerts} 
          icon={<span className="text-blue-500">👁️</span>} 
        />
      </div>
      
      <div className="flex-grow bg-slate-900/40 rounded-3xl border border-slate-800/60 p-8 flex flex-col relative overflow-hidden group">
        <div className="absolute top-0 right-0 w-64 h-64 bg-emerald-500/5 rounded-full blur-3xl -mr-32 -mt-32 transition-opacity group-hover:opacity-100 opacity-50" />
        <div className="absolute bottom-0 left-0 w-64 h-64 bg-blue-500/5 rounded-full blur-3xl -ml-32 -mb-32 transition-opacity group-hover:opacity-100 opacity-50" />

        <div className="flex justify-between items-center mb-6 relative z-10">
            <div className="space-y-1">
                <h3 className="text-sm font-bold text-slate-300 uppercase tracking-widest flex items-center">
                    <span className="w-4 h-4 mr-2 text-emerald-500">📊</span> System Activity Stream
                </h3>
                <p className="text-[10px] text-slate-500 font-medium italic">Autonomous Monitoring Active</p>
            </div>
            <div className="px-3 py-1 bg-slate-800/50 rounded-lg border border-slate-700 text-[10px] font-mono text-emerald-400">
                STATUS: OPTIMAL
            </div>
        </div>

        <div className="w-full flex-grow flex flex-col justify-center space-y-8 relative z-10">
            <ProgressBar label="Background Process Scan" status="ACTIVE • 100% SECURE" color="emerald" width="w-[85%]" />
            <ProgressBar label="Memory Entropy Buffer" status="STABLE • 1.4ms LATENCY" color="blue" width="w-[42%]" />
            <ProgressBar label="System Load Check" status="STABLE • NORMAL LOAD" color="amber" width="w-[67%]" />

            <div className="grid grid-cols-2 gap-8">
                <div className="bg-slate-800/20 rounded-2xl p-4 border border-slate-700/30">
                    <p className="text-[9px] text-slate-500 uppercase font-black mb-2">Last Scan</p>
                    <p className="text-xs font-mono text-slate-300">2026-03-05 23:55:02</p>
                </div>
                <div className="bg-slate-800/20 rounded-2xl p-4 border border-slate-700/30">
                    <p className="text-[9px] text-slate-500 uppercase font-black mb-2">Active Rules</p>
                    <p className="text-xs font-mono text-slate-300">12 Core / 8 Custom</p>
                </div>
            </div>
        </div>
      </div>
    </div>
  );
};

function StatCard({ title, value, icon }) {
  return (
    <div className="bg-slate-900/40 rounded-2xl border border-slate-800/60 p-5 space-y-3 transition-transform hover:scale-[1.02] cursor-default group shadow-lg">
      <div className="flex justify-between items-center">
        <span className="text-[9px] font-black text-slate-500 uppercase tracking-[0.15em]">{title}</span>
        <div className="p-1.5 bg-slate-800/50 rounded-lg group-hover:bg-slate-800 transition-colors">
            {icon}
        </div>
      </div>
      <div className="text-3xl font-black text-slate-100 tracking-tight">{value}</div>
    </div>
  );
}

function ProgressBar({ label, status, color, width }) {
    const colors = {
        emerald: "from-emerald-600 to-emerald-400 shadow-[0_0_15px_rgba(16,185,129,0.3)]",
        blue: "from-blue-600 to-blue-400 shadow-[0_0_15px_rgba(59,130,246,0.3)]",
        amber: "from-amber-600 to-amber-400 shadow-[0_0_15px_rgba(245,158,11,0.3)]"
    };
    return (
        <div className="space-y-3">
            <div className="flex justify-between text-[10px] text-slate-400 uppercase font-bold tracking-tighter">
                <span>{label}</span>
                <span className={`text-${color}-400`}>{status}</span>
            </div>
            <div className="w-full bg-slate-800/50 h-2 rounded-full overflow-hidden border border-slate-700/50">
                <div className={`bg-gradient-to-r ${colors[color]} h-full ${width} ${color === 'emerald' ? 'animate-pulse' : ''}`} />
            </div>
        </div>
    );
}

export default ActivityDashboard;
