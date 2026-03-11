import React, { useState, useEffect } from 'react';
import { TauriApi } from '../models/tauriApi';

const MitigationPanel = ({ darkMode }) => {
  const [threats, setThreats] = useState([]);
  const [loadingAction, setLoadingAction] = useState(null);

  useEffect(() => {
    const fetchThreats = async () => {
      try {
        const data = await TauriApi.getThreatProcesses();
        setThreats(data || []);
      } catch (e) {
        console.error("Failed to fetch threats", e);
      }
    };
    
    fetchThreats();
    const interval = setInterval(fetchThreats, 2000);
    return () => clearInterval(interval);
  }, []);

  const handleAction = async (pid, action) => {
    try {
      setLoadingAction(`${pid}-${action}`);
      await TauriApi.mitigateProcess(pid, action);
      
      // 樂觀更新或馬上重拉資料
      const updated = await TauriApi.getThreatProcesses();
      setThreats(updated || []);
    } catch (e) {
      console.error(`Mitigation failed: ${action} on ${pid}`, e);
      alert(`執行失敗：${e}`);
    } finally {
      setLoadingAction(null);
    }
  };

  return (
    <div className={`p-4 xl:p-6 rounded-2xl shadow-lg border ${darkMode ? 'bg-slate-800/80 border-slate-700/60' : 'bg-white border-slate-200'} transition-colors duration-300 mt-6`}>
      <div className="flex justify-between items-center mb-5">
        <h3 className={`text-lg font-bold flex items-center ${darkMode ? 'text-white' : 'text-slate-800'}`}>
          <span className="mr-2 text-xl">🛡️</span> 即時威脅防禦 (Active Mitigations)
        </h3>
        <span className="flex h-3 w-3 relative">
          {threats.length > 0 ? (
            <>
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-rose-400 opacity-75"></span>
              <span className="relative inline-flex rounded-full h-3 w-3 bg-rose-500"></span>
            </>
          ) : (
            <span className="relative inline-flex rounded-full h-3 w-3 bg-emerald-500"></span>
          )}
        </span>
      </div>
      
      {threats.length === 0 ? (
        <div className={`text-sm py-6 text-center rounded-xl bg-emerald-500/10 border border-emerald-500/20 ${darkMode ? 'text-emerald-400' : 'text-emerald-600'}`}>
          <span className="block text-2xl mb-2">✨</span>
          所有系統運行正常。尚未偵測到高風險程序。
        </div>
      ) : (
        <div className="space-y-3 max-h-[36rem] overflow-y-auto pr-1">
          {threats.map((proc) => {
            const isKillLoading = loadingAction === `${proc.pid}-kill`;
            const isIsolateLoading = loadingAction === `${proc.pid}-isolate`;
            const isResumeLoading = loadingAction === `${proc.pid}-resume`;
            
            return (
              <div key={proc.pid} className={`flex flex-col xl:flex-row items-start xl:items-center justify-between p-3.5 rounded-xl border ${darkMode ? 'bg-slate-900/50 border-slate-700' : 'bg-slate-50 border-slate-200'} transition-all hover:shadow-md`}>
                
                <div className="flex-1 mb-3 xl:mb-0">
                  <div className={`font-semibold text-sm xl:text-base flex items-center ${darkMode ? 'text-slate-200' : 'text-slate-800'}`}>
                    {proc.name} 
                    <span className={`ml-2 px-2 py-0.5 text-xs rounded-md border ${darkMode ? 'bg-slate-800 border-slate-600 text-slate-400' : 'bg-white border-slate-300 text-slate-500'}`}>
                      PID: {proc.pid}
                    </span>
                  </div>
                  <div className="flex flex-wrap gap-x-4 gap-y-1 mt-1.5 text-xs">
                    <span className="text-rose-500 font-medium">CPU: {proc.cpu_usage.toFixed(1)}%</span>
                    <span className="text-blue-500 font-medium">RAM: {proc.memory_mb.toFixed(1)} MB</span>
                    <span className={`${darkMode ? 'text-amber-400/80' : 'text-amber-600'} italic`}>Status: {proc.status}</span>
                  </div>
                </div>
                
                <div className="flex space-x-2 w-full xl:w-auto justify-end">
                  <button 
                    disabled={loadingAction !== null}
                    onClick={() => handleAction(proc.pid, 'isolate')} 
                    className="flex-1 xl:flex-none px-3 py-1.5 bg-amber-500/10 text-amber-500 hover:bg-amber-500 hover:text-white rounded-lg transition-colors text-xs font-semibold border border-amber-500/30 flex justify-center items-center" 
                    title="Isolate (Suspend)"
                  >
                    {isIsolateLoading ? <span className="animate-spin mr-1">⏳</span> : <span className="mr-1">🧊</span>} 凍結
                  </button>
                  <button 
                    disabled={loadingAction !== null}
                    onClick={() => handleAction(proc.pid, 'resume')} 
                    className="flex-1 xl:flex-none px-3 py-1.5 bg-emerald-500/10 text-emerald-500 hover:bg-emerald-500 hover:text-white rounded-lg transition-colors text-xs font-semibold border border-emerald-500/30 flex justify-center items-center" 
                    title="Resume"
                  >
                    {isResumeLoading ? <span className="animate-spin mr-1">⏳</span> : <span className="mr-1">▶️</span>} 恢復
                  </button>
                  <button 
                    disabled={loadingAction !== null}
                    onClick={() => handleAction(proc.pid, 'kill')} 
                    className="flex-1 xl:flex-none px-3 py-1.5 bg-rose-500/10 text-rose-500 hover:bg-rose-500 hover:text-white rounded-lg transition-colors text-xs font-semibold border border-rose-500/30 flex justify-center items-center" 
                    title="Terminate (Kill)"
                  >
                   {isKillLoading ? <span className="animate-spin mr-1">⏳</span> : <span className="mr-1">🛑</span>} 終止
                  </button>
                </div>

              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};

export default MitigationPanel;
