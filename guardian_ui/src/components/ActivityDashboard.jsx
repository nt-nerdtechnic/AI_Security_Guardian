import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

const ActivityDashboard = ({ stats }) => {
  const [exposedPorts, setExposedPorts] = useState([]);

  useEffect(() => {
    fetchExposedPorts();
    const interval = setInterval(fetchExposedPorts, 10000);
    return () => clearInterval(interval);
  }, []);

  const fetchExposedPorts = async () => {
    try {
      const ports = await invoke('get_exposed_ports');
      setExposedPorts(ports);
    } catch (e) {
      console.error("Failed to fetch exposed ports", e);
    }
  };

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
      
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 flex-grow overflow-hidden">
          {/* Left: System Status */}
          <div className="bg-slate-900/40 rounded-3xl border border-slate-800/60 p-8 flex flex-col relative overflow-hidden group">
            <div className="absolute top-0 right-0 w-64 h-64 bg-emerald-500/5 rounded-full blur-3xl -mr-32 -mt-32 opacity-50" />
            
            <div className="flex justify-between items-center mb-6 relative z-10">
                <div className="space-y-1">
                    <h3 className="text-sm font-bold text-slate-300 uppercase tracking-widest flex items-center">
                        <span className="w-4 h-4 mr-2 text-emerald-500">📊</span> System Stream
                    </h3>
                </div>
                <div className="px-3 py-1 bg-slate-800/50 rounded-lg border border-slate-700 text-[10px] font-mono text-emerald-400">
                    SECURE
                </div>
            </div>

            <div className="w-full flex-grow flex flex-col justify-center space-y-6 relative z-10">
                <ProgressBar label="Process Scan" status="ACTIVE" color="emerald" width="w-[85%]" />
                <ProgressBar label="Memory Buffer" status="STABLE" color="blue" width="w-[42%]" />
                
                <div className="pt-4 border-t border-slate-800/50">
                    <p className="text-[9px] text-slate-500 uppercase font-black mb-2">Last Heartbeat</p>
                    <p className="text-xs font-mono text-slate-300">Active • 2026-03-06 18:55</p>
                </div>
            </div>
          </div>

          {/* Right: Network Exposure (v1.0.7 Feature) */}
          <div className="bg-slate-900/40 rounded-3xl border border-slate-800/60 p-8 flex flex-col relative overflow-hidden group">
            <div className="absolute top-0 right-0 w-64 h-64 bg-amber-500/5 rounded-full blur-3xl -mr-32 -mt-32 opacity-50" />
            
            <div className="flex justify-between items-center mb-6 relative z-10">
                <div className="space-y-1">
                    <h3 className="text-sm font-bold text-slate-300 uppercase tracking-widest flex items-center">
                        <span className="w-4 h-4 mr-2 text-amber-500">🌐</span> Network Exposure
                    </h3>
                </div>
                <div className={`px-3 py-1 rounded-lg border text-[10px] font-mono ${exposedPorts.some(p => p.is_risky) ? 'bg-red-500/10 border-red-500/40 text-red-400' : 'bg-slate-800/50 border-slate-700 text-slate-400'}`}>
                    {exposedPorts.some(p => p.is_risky) ? 'WARNING' : (exposedPorts.length > 0 ? 'MONITORING' : 'PROTECTED')}
                </div>
            </div>

            <div className="flex-grow space-y-3 overflow-y-auto relative z-10 pr-2 scrollbar-thin scrollbar-thumb-slate-800">
                {exposedPorts.length > 0 ? (
                    exposedPorts.map((item, index) => (
                        <div key={`${item.port}-${index}`} className={`flex justify-between items-center p-3 border rounded-xl ${item.is_risky ? 'bg-red-500/5 border-red-500/20' : 'bg-slate-800/30 border-slate-700/50'}`}>
                            <div className="flex flex-col">
                                <span className={`text-xs font-mono ${item.is_risky ? 'text-red-400' : 'text-slate-300'}`}>Port {item.port}</span>
                                {item.process_name && <span className="text-[10px] text-slate-400 font-mono mt-1">{item.process_name} (PID: {item.pid})</span>}
                            </div>
                            <span className={`text-[9px] font-bold uppercase ${item.is_risky ? 'text-red-500/60' : 'text-slate-500'}`}>Exposed</span>
                        </div>
                    ))
                ) : (
                    <div className="flex flex-col items-center justify-center h-full opacity-30 text-slate-500 space-y-2">
                        <span className="text-3xl">🛡️</span>
                        <p className="text-[10px] font-bold uppercase tracking-widest">No Ports Exposed</p>
                    </div>
                )}
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