import React, { useState, useEffect } from 'react';
import { FileCheck, AlertTriangle, ShieldCheck, Info } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

const FileIntegrityAlerts = () => {
    const [alerts, setAlerts] = useState([]);

    useEffect(() => {
        fetchAlerts();
        const interval = setInterval(fetchAlerts, 10000);
        return () => clearInterval(interval);
    }, []);

    const fetchAlerts = async () => {
        try {
            const data = await invoke('check_file_integrity');
            setAlerts(data);
        } catch (e) {
            console.error("Failed to fetch file integrity", e);
        }
    };

    const formatTime = (timestamp) => {
        if (timestamp === "N/A" || !timestamp) return "N/A";
        const d = new Date(parseInt(timestamp) * 1000);
        const now = new Date();
        const diffInMinutes = Math.floor((now - d) / 60000);
        
        if (diffInMinutes < 60) return `${diffInMinutes} mins ago`;
        if (diffInMinutes < 1440) return `${Math.floor(diffInMinutes / 60)} hours ago`;
        return d.toLocaleDateString();
    };

    return (
        <div className="bg-slate-900/40 rounded-3xl border border-slate-800/60 p-8 flex flex-col relative overflow-hidden group h-full">
            <div className="absolute top-0 right-0 w-64 h-64 bg-blue-500/5 rounded-full blur-3xl -mr-32 -mt-32 opacity-50" />
            
            <div className="flex justify-between items-center mb-6 relative z-10">
                <div className="space-y-1">
                    <h3 className="text-sm font-bold text-slate-300 uppercase tracking-widest flex items-center">
                        <FileCheck className="w-4 h-4 mr-2 text-blue-500" /> File Integrity
                    </h3>
                </div>
                <div className={`px-3 py-1 rounded-lg border text-[10px] font-mono ${alerts.some(a => a.status === 'WARNING') ? 'bg-red-500/10 border-red-500/40 text-red-400' : 'bg-slate-800/50 border-slate-700 text-blue-400'}`}>
                    {alerts.some(a => a.status === 'WARNING') ? 'WARNING' : 'MONITORING'}
                </div>
            </div>

            <div className="flex-grow space-y-3 overflow-y-auto relative z-10 pr-2 scrollbar-thin scrollbar-thumb-slate-800">
                {alerts.map((alert, idx) => (
                    <div key={idx} className={`flex justify-between items-center p-3 border rounded-xl ${alert.status === 'WARNING' ? 'bg-red-500/5 border-red-500/20' : 'bg-slate-800/30 border-slate-700/50'}`}>
                        <div className="flex flex-col">
                            <span className="text-xs font-mono text-slate-300" title={alert.file_path}>
                                {alert.file_path.length > 25 ? '...' + alert.file_path.slice(-25) : alert.file_path}
                            </span>
                            <span className="text-[10px] text-slate-400 mt-1 uppercase">{alert.message}</span>
                        </div>
                        {alert.status === 'WARNING' ? (
                            <AlertTriangle className="w-4 h-4 text-red-500/80" />
                        ) : alert.status === 'OK' ? (
                            <div className="flex flex-col items-end">
                                <ShieldCheck className="w-3 h-3 text-emerald-500/60 mb-1" />
                                <span className="text-[9px] font-bold uppercase text-emerald-500/60">
                                    {formatTime(alert.last_modified)}
                                </span>
                            </div>
                        ) : (
                            <Info className="w-4 h-4 text-slate-500/60" />
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
};

export default FileIntegrityAlerts;
