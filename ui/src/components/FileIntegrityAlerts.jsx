import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { FileCheck, AlertTriangle, ShieldCheck, Info, Plus, Trash2, Settings2 } from 'lucide-react';
import { useLanguage } from '../i18n/LanguageContext';

const FileIntegrityAlerts = ({ darkMode, t: theme }) => {
    const { t } = useLanguage();
    const [alerts, setAlerts] = useState([]);
    const [config, setConfig] = useState(null);
    const [isManaging, setIsManaging] = useState(false);
    const [newPath, setNewPath] = useState('');

    useEffect(() => {
        const fetchConfig = async () => {
            try {
                const cfg = await invoke('get_config');
                setConfig(cfg);
            } catch (e) {
                console.error("Failed to fetch config", e);
            }
        };
        fetchConfig();
    }, []);

    useEffect(() => {
        const fetchAlerts = async () => {
            try {
                const data = await invoke('check_file_integrity');
                setAlerts(data);
            } catch (e) {
                console.error("Failed to fetch file integrity", e);
            }
        };

        fetchAlerts();
        const interval = setInterval(fetchAlerts, 10000);
        return () => clearInterval(interval);
    }, []);

    const handleAddPath = async () => {
        if (!newPath || !config) return;
        
        try {
            const updatedConfig = { ...config };
            if (!updatedConfig.file_integrity) {
                updatedConfig.file_integrity = { custom_paths: [] };
            }
            if (!updatedConfig.file_integrity.custom_paths.includes(newPath)) {
                updatedConfig.file_integrity.custom_paths.push(newPath);
                await invoke('update_config', { mode: config.mode, modules: config.modules, fileIntegrity: updatedConfig.file_integrity });
                setConfig(updatedConfig);
                setNewPath('');
                // 立即重新抓一次檔案完整性
                const data = await invoke('check_file_integrity');
                setAlerts(data);
            }
        } catch (e) {
            console.error("Failed to add path", e);
        }
    };

    const handleRemovePath = async (pathToRemove) => {
        if (!config) return;
        try {
            const updatedConfig = { ...config };
            if (updatedConfig.file_integrity && updatedConfig.file_integrity.custom_paths) {
                updatedConfig.file_integrity.custom_paths = updatedConfig.file_integrity.custom_paths.filter(p => p !== pathToRemove);
                await invoke('update_config', { mode: config.mode, modules: config.modules, fileIntegrity: updatedConfig.file_integrity });
                setConfig(updatedConfig);
                // 更新清單
                const data = await invoke('check_file_integrity');
                setAlerts(data);
            }
        } catch (e) {
            console.error("Failed to remove path", e);
        }
    };

    const formatTime = (timestamp) => {
        if (timestamp === "N/A" || !timestamp) return "N/A";
        const d = new Date(parseInt(timestamp) * 1000);
        const now = new Date();
        const diffInMinutes = Math.floor((now - d) / 60000);

        if (diffInMinutes < 60) return t('file_integrity.time_mins_ago', { n: diffInMinutes });
        if (diffInMinutes < 1440) return t('file_integrity.time_hours_ago', { n: Math.floor(diffInMinutes / 60) });
        return d.toLocaleDateString();
    };

    const hasWarning = alerts.some(a => a.status === 'WARNING');

    return (
        <div className={`${theme.surface} rounded-3xl border ${theme.border} p-8 flex flex-col relative overflow-hidden group h-full transition-colors duration-300`}>
            <div className="absolute top-0 right-0 w-64 h-64 bg-blue-500/5 rounded-full blur-3xl -mr-32 -mt-32 opacity-50" />

            <div className="flex justify-between items-center mb-6 relative z-10">
                <div className="space-y-1">
                    <h3 className={`text-sm font-bold ${darkMode ? 'text-slate-300' : 'text-slate-600'} uppercase tracking-widest flex items-center`}>
                        <FileCheck className="w-4 h-4 mr-2 text-blue-500" /> {t('file_integrity.title')}
                    </h3>
                </div>
                <div className="flex items-center space-x-3">
                    <button
                        onClick={() => setIsManaging(!isManaging)}
                        className={`p-1.5 rounded-lg border transition-colors ${
                            isManaging 
                                ? (darkMode ? 'bg-blue-500/20 border-blue-500/40 text-blue-400' : 'bg-blue-50 border-blue-200 text-blue-600')
                                : (darkMode ? 'bg-slate-800/50 border-slate-700 text-slate-400 hover:text-slate-300 hover:border-slate-600' : 'bg-slate-50 border-slate-200 text-slate-500 hover:text-slate-700')
                        }`}
                        title={t('file_integrity.manage_paths') || 'Manage Paths'}
                    >
                        <Settings2 className="w-3.5 h-3.5" />
                    </button>
                    <div className={`px-3 py-1 rounded-lg border text-[10px] font-mono ${
                        hasWarning
                            ? 'bg-red-500/10 border-red-500/40 text-red-400'
                            : darkMode
                                ? 'bg-slate-800/50 border-slate-700 text-blue-400'
                                : 'bg-blue-50 border-blue-200 text-blue-500'
                    }`}>
                        {hasWarning ? t('file_integrity.status_warning') : t('file_integrity.status_monitoring')}
                    </div>
                </div>
            </div>

            {isManaging && config && (
                <div className={`mb-4 p-4 rounded-xl border ${darkMode ? 'bg-slate-800/40 border-slate-700/60' : 'bg-white/60 border-slate-200'} relative z-10`}>
                    <h4 className={`text-xs font-bold uppercase tracking-wider mb-3 ${darkMode ? 'text-slate-400' : 'text-slate-500'}`}>
                        {t('file_integrity.manage_paths') || 'Custom Guard Paths'}
                    </h4>
                    
                    <div className="space-y-2 mb-4">
                        {(config.file_integrity?.custom_paths || []).map((path, idx) => (
                            <div key={idx} className={`flex justify-between items-center p-2 rounded border ${darkMode ? 'bg-slate-800/80 border-slate-700' : 'bg-slate-50 border-slate-200'}`}>
                                <span className={`text-xs font-mono truncate mr-2 ${darkMode ? 'text-slate-300' : 'text-slate-600'}`}>{path}</span>
                                <button
                                    onClick={() => handleRemovePath(path)}
                                    className="p-1 text-red-400 hover:bg-red-500/10 rounded transition-colors"
                                    title={t('file_integrity.remove_path') || 'Remove'}
                                >
                                    <Trash2 className="w-3.5 h-3.5" />
                                </button>
                            </div>
                        ))}
                    </div>

                    <div className="flex space-x-2">
                        <input
                            type="text"
                            value={newPath}
                            onChange={(e) => setNewPath(e.target.value)}
                            placeholder={t('file_integrity.placeholder_path') || "Absolute path (e.g. /Users/...)"}
                            className={`flex-grow text-xs rounded border px-3 py-1.5 focus:outline-none focus:ring-1 ${
                                darkMode 
                                    ? 'bg-slate-900/50 border-slate-700 text-slate-300 focus:border-blue-500/50 focus:ring-blue-500/50' 
                                    : 'bg-white border-slate-300 text-slate-700 focus:border-blue-500 focus:ring-blue-500'
                            }`}
                        />
                        <button
                            onClick={handleAddPath}
                            disabled={!newPath}
                            className={`px-3 py-1.5 rounded flex items-center text-xs font-bold transition-all ${
                                !newPath 
                                    ? 'opacity-50 cursor-not-allowed bg-slate-500/20 text-slate-500' 
                                    : 'bg-blue-500 hover:bg-blue-600 text-white shadow-lg shadow-blue-500/20'
                            }`}
                        >
                            <Plus className="w-3.5 h-3.5 mr-1" /> {t('file_integrity.add_path') || 'Add'}
                        </button>
                    </div>
                </div>
            )}

            <div className="flex-grow space-y-3 overflow-y-auto relative z-10 pr-2 scrollbar-thin scrollbar-thumb-slate-800">
                {alerts.filter(a => !a.ignored).map((alert, idx) => (
                    <div key={idx} className={`flex flex-col p-3 border rounded-xl space-y-3 ${
                        alert.status === 'WARNING'
                            ? 'bg-red-500/5 border-red-500/20'
                            : darkMode ? 'bg-slate-800/30 border-slate-700/50' : 'bg-slate-50 border-slate-200'
                    }`}>
                        <div className="flex justify-between items-center">
                            <div className="flex flex-col">
                                <span className={`text-xs font-mono ${darkMode ? 'text-slate-300' : 'text-slate-600'}`} title={alert.file_path}>
                                    {alert.file_path.length > 25 ? '...' + alert.file_path.slice(-25) : alert.file_path}
                                </span>
                                <span className={`text-[10px] ${darkMode ? 'text-slate-400' : 'text-slate-500'} mt-1 uppercase`}>{alert.message}</span>
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

                        {alert.status === 'WARNING' && (
                            <div className="flex justify-end space-x-2 pt-2 border-t border-red-500/10">
                                <button
                                    onClick={async () => {
                                        if (window.confirm(t('file_integrity.confirm_quarantine', { path: alert.file_path }))) {
                                            try {
                                                const res = await invoke('move_to_quarantine', { file_path: alert.file_path });
                                                console.log("Quarantine result:", res);
                                                const data = await invoke('check_file_integrity');
                                                setAlerts(data);
                                            } catch (err) {
                                                console.error("Failed to quarantine file:", err);
                                            }
                                        }
                                    }}
                                    className="px-3 py-1 bg-amber-500/10 hover:bg-amber-500/20 border border-amber-500/30 rounded text-[10px] font-bold text-amber-500 uppercase tracking-wider transition-colors"
                                >
                                    {t('file_integrity.btn_quarantine')}
                                </button>
                                <button
                                    onClick={() => {
                                        if (window.confirm(t('file_integrity.confirm_ignore', { path: alert.file_path }))) {
                                            setAlerts(prev => prev.map((a, i) =>
                                                i === idx ? { ...a, ignored: true } : a
                                            ));
                                        }
                                    }}
                                    className={`px-3 py-1 ${darkMode ? 'bg-slate-800/50 hover:bg-slate-700/50 border-slate-700 text-slate-400' : 'bg-slate-100 hover:bg-slate-200 border-slate-300 text-slate-500'} border rounded text-[10px] font-bold uppercase tracking-wider transition-colors`}
                                >
                                    {t('file_integrity.btn_ignore')}
                                </button>
                            </div>
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
};

export default FileIntegrityAlerts;
