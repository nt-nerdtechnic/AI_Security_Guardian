import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Globe, ShieldAlert, ChevronDown, ChevronUp } from 'lucide-react';
import FileIntegrityAlerts from './FileIntegrityAlerts';
import { useLanguage } from '../i18n/LanguageContext';

// ── Port 用途描述表 key 對應 ─────────────────────────────────────
const PORT_RISK_KEYS = {
  21:    { name: 'FTP',        riskKey: 'ftp_risk' },
  22:    { name: 'SSH',        riskKey: 'ssh_risk' },
  23:    { name: 'Telnet',     riskKey: 'telnet_risk' },
  25:    { name: 'SMTP',       riskKey: 'smtp_risk' },
  53:    { name: 'DNS',        riskKey: 'dns_risk' },
  80:    { name: 'HTTP',       riskKey: 'http_risk' },
  443:   { name: 'HTTPS',      riskKey: 'https_risk' },
  1433:  { name: 'MSSQL',      riskKey: 'mssql_risk' },
  3000:  { name: 'Dev Server', riskKey: 'dev_server_risk' },
  3306:  { name: 'MySQL',      riskKey: 'mysql_risk' },
  5432:  { name: 'PostgreSQL', riskKey: 'postgresql_risk' },
  5900:  { name: 'VNC',        riskKey: 'vnc_risk' },
  6379:  { name: 'Redis',      riskKey: 'redis_risk' },
  8080:  { name: 'HTTP Alt',   riskKey: 'http_alt_risk' },
  8443:  { name: 'HTTPS Alt',  riskKey: 'https_alt_risk' },
  27017: { name: 'MongoDB',    riskKey: 'mongodb_risk' },
};

const ActivityDashboard = ({ stats, events = { threats: [], keys: [], visual: [] }, darkMode, t: theme }) => {
  const { t } = useLanguage();
  const [exposedPorts, setExposedPorts] = useState([]);

  const getPortLabel = useCallback((port) => {
    const entry = PORT_RISK_KEYS[port];
    if (entry) return { name: entry.name, risk: t(`port_risks.${entry.riskKey}`) };
    return { name: `Port ${port}`, risk: t('port_risks.system_service_risk') };
  }, [t]);

  // 後端已在 get_exposed_ports 內根據 DB 白名單標記 ignored，直接使用即可
  const fetchExposedPorts = useCallback(async () => {
    try {
      const ports = await invoke('get_exposed_ports');
      setExposedPorts(ports);
    } catch (e) {
      console.error("Failed to fetch exposed ports", e);
    }
  }, []);

  useEffect(() => {
    fetchExposedPorts();
    const interval = setInterval(fetchExposedPorts, 10000);
    return () => clearInterval(interval);
  }, [fetchExposedPorts]);

  const panel = `${theme.surface} rounded-3xl border ${theme.border} p-8 flex flex-col relative overflow-hidden group transition-colors duration-300`;

  return (
    <div className="flex flex-col space-y-6">
      <div className="grid grid-cols-3 gap-4">
        <StatCard
          title={t('stats.threats_blocked')}
          subtitle={t('stats.threats_blocked_sub')}
          value={stats.total_blocked}
          icon={<span className="text-amber-500">⚠️</span>}
          events={events.threats}
          darkMode={darkMode} theme={theme}
        />
        <StatCard
          title={t('stats.keys_redacted')}
          subtitle={t('stats.keys_redacted_sub')}
          value={stats.sensitive_keys}
          icon={<span className="text-emerald-500">📋</span>}
          events={events.keys}
          darkMode={darkMode} theme={theme}
        />
        <StatCard
          title={t('stats.visual_alerts')}
          subtitle={t('stats.visual_alerts_sub')}
          value={stats.visual_alerts}
          icon={<span className="text-blue-500">👁️</span>}
          events={events.visual}
          darkMode={darkMode} theme={theme}
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left: System Status */}
        <div className={panel}>
          <div className="absolute top-0 right-0 w-64 h-64 bg-emerald-500/5 rounded-full blur-3xl -mr-32 -mt-32 opacity-50" />

          <div className="flex justify-between items-center mb-6 relative z-10">
            <div className="space-y-1">
              <h3 className={`text-sm font-bold ${darkMode ? 'text-slate-300' : 'text-slate-600'} uppercase tracking-widest flex items-center`}>
                <span className="w-4 h-4 mr-2 text-emerald-500">📊</span> {t('system_stream.title')}
              </h3>
            </div>
            <div className={`px-3 py-1 ${darkMode ? 'bg-slate-800/50 border-slate-700 text-emerald-400' : 'bg-emerald-50 border-emerald-200 text-emerald-600'} rounded-lg border text-[10px] font-mono`}>
              {t('system_stream.status_secure')}
            </div>
          </div>

          <div className="w-full flex-grow flex flex-col justify-start space-y-6 relative z-10">
            <ProgressBar
              label={t('system_stream.process_scan')}
              status={t('system_stream.status_active')}
              color="emerald" width="w-[85%]" darkMode={darkMode}
            />
            <ProgressBar
              label={t('system_stream.memory_buffer')}
              status={t('system_stream.status_stable')}
              color="blue" width="w-[42%]" darkMode={darkMode}
            />

            <div className={`pt-4 border-t ${darkMode ? 'border-slate-800/50' : 'border-slate-200'}`}>
              <p className={`text-[9px] ${theme.textMuted} uppercase font-black mb-2`}>{t('system_stream.last_heartbeat')}</p>
              <p className={`text-xs font-mono ${darkMode ? 'text-slate-300' : 'text-slate-600'}`}>
                {t('system_stream.heartbeat_active')} • 2026-03-06 18:55
              </p>
            </div>
          </div>
        </div>

        {/* Center: Network Exposure */}
        <NetworkExposurePanel
          exposedPorts={exposedPorts}
          setExposedPorts={setExposedPorts}
          fetchExposedPorts={fetchExposedPorts}
          panel={panel}
          darkMode={darkMode}
          theme={theme}
          getPortLabel={getPortLabel}
          t={t}
        />

        {/* Right: File Integrity Alerts */}
        <FileIntegrityAlerts darkMode={darkMode} t={theme} />
      </div>
    </div>
  );
};

function NetworkExposurePanel({ exposedPorts, setExposedPorts, fetchExposedPorts, panel, darkMode, theme, getPortLabel, t }) {
  const [safeExpanded, setSafeExpanded] = useState(false);
  const [allowedExpanded, setAllowedExpanded] = useState(false);

  // 風險：所有 is_risky 項目 (不管是否被 ignore)
  const allRisky = exposedPorts.filter(p => p.is_risky);
  const activeRisky = allRisky.filter(p => !p.ignored);
  const ignoredRisky = allRisky.filter(p => p.ignored);

  // 安全：純正常服務
  const safe = exposedPorts.filter(p => !p.is_risky);

  const safeGrouped = React.useMemo(() => {
    const groups = {};
    safe.forEach(item => {
      const pName = item.process_name || t('network.unknown_process');
      if (!groups[pName]) groups[pName] = [];
      groups[pName].push(item);
    });
    return Object.entries(groups).map(([name, items]) => ({ name, items })).sort((a, b) => b.items.length - a.items.length);
  }, [safe, t]);

  const visible = exposedPorts;
  const hasRisky = activeRisky.length > 0;

  const threatLabel = activeRisky.length === 1
    ? t('network.threat')
    : t('network.threats');

  return (
    <div className={panel}>
      <div className="absolute top-0 right-0 w-64 h-64 bg-amber-500/5 rounded-full blur-3xl -mr-32 -mt-32 opacity-50" />

      {/* Header */}
      <div className="flex justify-between items-center mb-4 relative z-10">
        <h3 className={`text-sm font-bold ${darkMode ? 'text-slate-300' : 'text-slate-600'} uppercase tracking-widest flex items-center`}>
          <span className="mr-2">🌐</span> {t('network.title')}
        </h3>
        <div className={`px-3 py-1 rounded-lg border text-[10px] font-mono ${
          hasRisky
            ? 'bg-red-500/10 border-red-500/40 text-red-400 animate-pulse'
            : darkMode ? 'bg-slate-800/50 border-slate-700 text-slate-400'
                       : 'bg-slate-100 border-slate-300 text-slate-500'
        }`}>
          {hasRisky ? `⚠ ${activeRisky.length} ${threatLabel}` : exposedPorts.length > 0 ? t('network.status_monitoring') : t('network.status_protected')}
        </div>
      </div>

      {/* Summary counts */}
      <div className="flex items-center flex-wrap gap-2 mb-4 relative z-10">
        <span className="text-[10px] font-bold text-red-400 bg-red-500/10 border border-red-500/20 px-2 py-0.5 rounded-full flex items-center">
          <span className="mr-1">🔴</span> {activeRisky.length} {t('network.risky_count')}
        </span>
        {ignoredRisky.length > 0 && (
          <span className={`text-[10px] font-bold text-emerald-500 ${darkMode ? 'bg-emerald-500/10 border-emerald-500/20' : 'bg-emerald-50 border-emerald-200'} border px-2 py-0.5 rounded-full flex items-center`}>
            <span className="mr-1">🛡️</span> {ignoredRisky.length} {t('network.allowed_count')}
          </span>
        )}
        <span className={`text-[10px] font-bold text-emerald-400 ${darkMode ? 'bg-emerald-500/10 border-emerald-500/20' : 'bg-emerald-50 border-emerald-200'} border px-2 py-0.5 rounded-full flex items-center`}>
          <span className="mr-1">🟢</span> {safe.length} {t('network.safe_count')}
        </span>
        <span className={`text-[10px] ${darkMode ? 'text-slate-500' : 'text-slate-400'} ml-1`}>
          {t('network.total_ports', { count: visible.length })}
        </span>
      </div>

      <div className="flex-grow space-y-2 overflow-y-auto relative z-10 pr-1">

        {/* ── 風險 Ports ── */}
        {activeRisky.length > 0 && (
          <div className="space-y-2">
            {activeRisky.map((item, i) => {
              const label = getPortLabel(item.port);
              return (
                <div key={`risky-${item.port}-${i}`}
                  className="flex flex-col p-3 border rounded-xl space-y-2 bg-red-500/5 border-red-500/20">
                  <div className="flex justify-between items-start">
                    <div className="flex items-center space-x-2">
                      <ShieldAlert className="w-4 h-4 flex-shrink-0 text-red-500" />
                      <div>
                        <div className="flex items-center space-x-2">
                          <span className="text-xs font-bold text-red-400">{label.name}</span>
                          <span className="text-[10px] font-mono text-red-300/60">:{item.port}</span>
                        </div>
                        <p className="text-[10px] mt-0.5 text-red-300/50">{label.risk}</p>
                        <p className={`text-[10px] font-mono ${darkMode ? 'text-slate-500' : 'text-slate-400'} mt-0.5`}>
                          {item.process_name || t('network.unknown_process')} · PID {item.pid}
                        </p>
                      </div>
                    </div>
                    <span className="text-[9px] font-bold uppercase mt-0.5 flex-shrink-0 text-red-500/60">
                      {t('network.exposed_label')}
                    </span>
                  </div>
                  <div className="flex justify-end space-x-2 pt-1 border-t border-red-500/10">
                    <button
                      onClick={async () => {
                        try { await invoke('terminate_process', { pid: item.pid }); fetchExposedPorts(); }
                        catch (err) { console.error(err); }
                      }}
                      className="px-2.5 py-1 bg-red-500/10 hover:bg-red-500/25 border border-red-500/30 rounded text-[10px] font-bold text-red-400 uppercase tracking-wider transition-colors"
                    >{t('network.btn_deny')}</button>
                    <button
                      onClick={async () => {
                        try {
                          await invoke('add_network_whitelist', { port: item.port, pid: item.pid, processName: item.process_name || '' });
                          setExposedPorts(prev => prev.map(p => p.port === item.port ? { ...p, ignored: true } : p));
                        } catch (err) {
                          console.error('Failed to add whitelist', err);
                        }
                      }}
                      className={`px-2.5 py-1 ${darkMode ? 'bg-slate-800/50 hover:bg-slate-700/50 border-slate-700 text-slate-400' : 'bg-slate-100 hover:bg-slate-200 border-slate-300 text-slate-500'} border rounded text-[10px] font-bold uppercase tracking-wider transition-colors`}
                    >{t('network.btn_approve')}</button>
                  </div>
                </div>
              );
            })}
          </div>
        )}

        {/* ── 已允許 Ports (折疊) ── */}
        {ignoredRisky.length > 0 && (
          <div className={`rounded-xl border ${darkMode ? 'border-emerald-500/20 bg-emerald-500/5' : 'border-emerald-200 bg-emerald-50'} overflow-hidden mt-4`}>
            <button
              onClick={() => setAllowedExpanded(v => !v)}
              className={`w-full flex justify-between items-center px-3 py-2.5 text-left transition-colors ${darkMode ? 'hover:bg-emerald-500/10' : 'hover:bg-emerald-100'}`}
            >
              <span className={`text-[10px] font-bold ${darkMode ? 'text-emerald-400' : 'text-emerald-600'} uppercase tracking-widest flex items-center space-x-2`}>
                <span>🛡️</span>
                <span>{t('network.allowed_count')}: {ignoredRisky.length}</span>
              </span>
              {allowedExpanded
                ? <ChevronUp className={`w-3.5 h-3.5 ${darkMode ? 'text-emerald-500' : 'text-emerald-600'}`} />
                : <ChevronDown className={`w-3.5 h-3.5 ${darkMode ? 'text-emerald-500' : 'text-emerald-600'}`} />
              }
            </button>

            {allowedExpanded && (
              <div className="p-2 space-y-2">
                {ignoredRisky.map((item, i) => {
                  const label = getPortLabel(item.port);
                  return (
                    <div key={`ignored-${item.port}-${i}`}
                      className={`flex flex-col p-3 border rounded-xl space-y-2 ${darkMode ? 'bg-slate-800/40 border-slate-700/50' : 'bg-white border-slate-200'} shadow-sm`}>
                      <div className="flex justify-between items-start">
                        <div className="flex items-center space-x-2">
                          <ShieldAlert className="w-4 h-4 flex-shrink-0 text-emerald-500" />
                          <div>
                            <div className="flex items-center space-x-2">
                              <span className="text-xs font-bold text-emerald-500">{label.name}</span>
                              <span className={`text-[10px] font-mono ${darkMode ? 'text-emerald-400/60' : 'text-emerald-600/60'}`}>:{item.port}</span>
                            </div>
                            <p className={`text-[10px] mt-0.5 ${darkMode ? 'text-emerald-400/50' : 'text-emerald-600/50'}`}>{label.risk}</p>
                            <p className={`text-[10px] font-mono ${darkMode ? 'text-slate-500' : 'text-slate-400'} mt-0.5`}>
                              {item.process_name || t('network.unknown_process')} · PID {item.pid}
                            </p>
                          </div>
                        </div>
                        <span className="text-[9px] font-bold uppercase mt-0.5 flex-shrink-0 text-emerald-500/60">
                          {t('network.whitelisted_label')}
                        </span>
                      </div>
                      <div className={`flex justify-between pt-1 border-t ${darkMode ? 'border-slate-700/50' : 'border-slate-100'}`}>
                        <button
                          onClick={async () => {
                            try {
                              await invoke('remove_network_whitelist', { port: item.port });
                              setExposedPorts(prev => prev.map(p => p.port === item.port ? { ...p, ignored: false } : p));
                            } catch (err) {
                              console.error('Failed to remove whitelist', err);
                            }
                          }}
                          className={`px-2.5 py-1 rounded text-[10px] font-bold uppercase transition-colors ${darkMode ? 'bg-slate-700 hover:bg-slate-600 text-slate-300' : 'bg-slate-200 hover:bg-slate-300 text-slate-600'}`}
                        >
                          {t('network.btn_revoke')}
                        </button>
                        <button
                          onClick={async () => {
                            try { await invoke('terminate_process', { pid: item.pid }); fetchExposedPorts(); }
                            catch (err) { console.error(err); }
                          }}
                          className="px-2.5 py-1 bg-red-500/10 hover:bg-red-500/25 border border-red-500/30 rounded text-[10px] font-bold text-red-400 uppercase tracking-wider transition-colors"
                        >{t('network.btn_kill')}</button>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        )}

        {/* ── 安全 Ports (折疊) ── */}
        {safe.length > 0 && (
          <div className={`rounded-xl border ${darkMode ? 'border-slate-700/50 bg-slate-800/20' : 'border-slate-200 bg-slate-50'} overflow-hidden mt-4`}>
            <button
              onClick={() => setSafeExpanded(v => !v)}
              className={`w-full flex justify-between items-center px-3 py-2.5 text-left transition-colors ${darkMode ? 'hover:bg-slate-800/40' : 'hover:bg-slate-100'}`}
            >
              <span className={`text-[10px] font-bold ${darkMode ? 'text-slate-400' : 'text-slate-500'} uppercase tracking-widest flex items-center space-x-2`}>
                <Globe className="w-3 h-3" />
                <span>{t('network.system_listening', { count: safe.length })}</span>
              </span>
              {safeExpanded
                ? <ChevronUp className={`w-3.5 h-3.5 ${darkMode ? 'text-slate-500' : 'text-slate-400'}`} />
                : <ChevronDown className={`w-3.5 h-3.5 ${darkMode ? 'text-slate-500' : 'text-slate-400'}`} />
              }
            </button>

            {safeExpanded && (
              <div className={`divide-y ${darkMode ? 'divide-slate-700/50' : 'divide-slate-200'}`}>
                {safeGrouped.map((group, i) => (
                  <SafeProcessGroup key={`safe-group-${i}`} group={group} getPortLabel={getPortLabel} darkMode={darkMode} t={t} fetchExposedPorts={fetchExposedPorts} />
                ))}
              </div>
            )}
          </div>
        )}

        {/* 空狀態 */}
        {visible.length === 0 && (
          <div className="flex flex-col items-center justify-center py-10 opacity-30 text-slate-500 space-y-2">
            <span className="text-3xl">🛡️</span>
            <p className="text-[10px] font-bold uppercase tracking-widest">{t('network.no_ports')}</p>
          </div>
        )}
      </div>
    </div>
  );
}

function SafeProcessGroup({ group, getPortLabel, darkMode, t, fetchExposedPorts }) {
  const [expanded, setExpanded] = useState(false);
  const multiple = group.items.length > 1;

  return (
    <div className={`flex flex-col ${darkMode ? 'hover:bg-slate-800/30' : 'hover:bg-slate-100/50'} transition-colors`}>
      <div 
        onClick={() => multiple ? setExpanded(!expanded) : null}
        className={`flex items-center justify-between px-3 py-2 ${multiple ? 'cursor-pointer' : ''}`}
      >
        <div className="flex items-center justify-between w-full">
          <div className="flex items-center space-x-2">
            <Globe className={`w-3 h-3 ${darkMode ? 'text-slate-500' : 'text-slate-400'} flex-shrink-0`} />
            <span className={`text-[11px] font-bold ${darkMode ? 'text-slate-300' : 'text-slate-600'}`}>
              {group.name}
            </span>
            {multiple && (
              <span className={`text-[9px] px-1.5 py-0.5 rounded-full ${darkMode ? 'bg-slate-700 text-slate-400' : 'bg-slate-200 text-slate-500'}`}>
                {group.items.length}
              </span>
            )}
          </div>
          
          {multiple && (
            <div className="flex items-center space-x-2">
              {expanded 
                ? <ChevronUp className={`w-3.5 h-3.5 ${darkMode ? 'text-slate-500' : 'text-slate-400'}`} />
                : <ChevronDown className={`w-3.5 h-3.5 ${darkMode ? 'text-slate-500' : 'text-slate-400'}`} />
              }
            </div>
          )}
        </div>
      </div>
      
      {(expanded || !multiple) && (
        <div className={`pl-8 pr-3 pb-2 space-y-1 ${!multiple ? 'pt-0' : 'pt-1 border-t ' + (darkMode ? 'border-slate-700/50' : 'border-slate-200')}`}>
          {group.items.map((item, i) => {
            const label = getPortLabel(item.port);
            return (
              <div key={i} className="flex justify-between items-center py-1">
                <div>
                  <span className={`text-[10px] font-mono ${darkMode ? 'text-emerald-400/80' : 'text-emerald-600/80'}`}>
                    {label.name} <span className="opacity-60">:{item.port}</span>
                  </span>
                  <p className={`text-[9px] ${darkMode ? 'text-slate-500' : 'text-slate-400'} font-mono`}>
                    PID {item.pid}
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <span className={`text-[9px] font-bold uppercase ${darkMode ? 'text-emerald-500/40' : 'text-emerald-500/60'}`}>
                    {t('network.listen_label')}
                  </span>
                  <button
                    onClick={async (e) => {
                      e.stopPropagation();
                      try { await invoke('terminate_process', { pid: item.pid }); fetchExposedPorts(); }
                      catch (err) { console.error(err); }
                    }}
                    className="px-2 py-0.5 bg-red-500/10 hover:bg-red-500/25 border border-red-500/30 rounded text-[9px] font-bold text-red-400 uppercase tracking-wider transition-colors"
                  >{t('network.btn_kill')}</button>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

function StatCard({ title, subtitle, value, icon, darkMode, theme, events = [] }) {
  const [open, setOpen] = useState(false);

  // ESC 關閉 Modal
  useEffect(() => {
    if (!open) return;
    const handler = (e) => { if (e.key === 'Escape') setOpen(false); };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [open]);

  return (
    <>
      <div
        onClick={() => setOpen(true)}
        className={`${theme.surface} rounded-2xl border ${theme.border} p-5 space-y-1 transition-all hover:scale-[1.02] cursor-pointer group shadow-lg duration-300 relative`}
        title="點擊查看事件明細"
      >
        <div className="flex justify-between items-center">
          <span className={`text-xs font-bold ${darkMode ? 'text-slate-300' : 'text-slate-500'} uppercase tracking-wider`}>{title}</span>
          <div className={`p-1.5 ${darkMode ? 'bg-slate-800/50 group-hover:bg-slate-800' : 'bg-slate-100 group-hover:bg-slate-200'} rounded-lg transition-colors`}>
            {icon}
          </div>
        </div>
        <div className={`text-3xl font-black ${theme.text} tracking-tight`}>{value}</div>
        <div className="flex items-center justify-between mt-1">
          {subtitle && (
            <p className={`text-[9px] ${darkMode ? 'text-slate-600' : 'text-slate-400'} uppercase tracking-wider`}>{subtitle}</p>
          )}
          <div className={`text-[9px] ${darkMode ? 'text-slate-600' : 'text-slate-400'} flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity`}>
            <span>點擊查看明細</span>
            {events.length > 0 && <span className="inline-block w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />}
          </div>
        </div>
      </div>

      {/* Modal */}
      {open && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
          onClick={() => setOpen(false)}
        >
          <div
            className={`${darkMode ? 'bg-slate-900 border-slate-700' : 'bg-white border-slate-200'} border rounded-2xl shadow-2xl w-full max-w-md mx-4 overflow-hidden`}
            onClick={e => e.stopPropagation()}
          >
            {/* Modal Header */}
            <div className={`flex justify-between items-center px-6 py-4 border-b ${darkMode ? 'border-slate-800' : 'border-slate-100'}`}>
              <div className="flex items-center gap-2">
                {icon}
                <div>
                  <h2 className={`text-sm font-bold ${darkMode ? 'text-slate-100' : 'text-slate-800'} uppercase tracking-wider`}>{title}</h2>
                  <p className={`text-[10px] ${darkMode ? 'text-slate-500' : 'text-slate-400'}`}>{subtitle} · 共 {value} 筆</p>
                </div>
              </div>
              <button
                onClick={() => setOpen(false)}
                className={`text-lg leading-none ${darkMode ? 'text-slate-500 hover:text-slate-300' : 'text-slate-400 hover:text-slate-600'} transition-colors`}
              >✕</button>
            </div>

            {/* Event List */}
            <div className="max-h-80 overflow-y-auto px-6 py-4 space-y-2">
              {events.length > 0 ? (
                events.map((ev, i) => (
                  <div key={i} className={`flex flex-col gap-1.5 p-3 rounded-xl ${darkMode ? 'bg-slate-800/50' : 'bg-slate-50'} border ${darkMode ? 'border-slate-700/50' : 'border-slate-200'}`}>
                    <div className="flex justify-between items-center">
                      <span className={`text-[10px] font-bold px-2 py-0.5 rounded uppercase tracking-wider ${darkMode ? 'bg-slate-700 text-slate-300' : 'bg-slate-200 text-slate-700'}`}>
                        {ev.source || 'System Event'}
                      </span>
                      <span className={`text-[10px] font-mono flex-shrink-0 ${darkMode ? 'text-slate-500' : 'text-slate-400'}`}>
                        {ev.time}
                      </span>
                    </div>
                    <p className={`text-xs mt-1 ${darkMode ? 'text-slate-100' : 'text-slate-700'}`}>{ev.desc}</p>
                  </div>
                ))
              ) : (
                <div className={`flex flex-col items-center justify-center py-10 space-y-2 ${darkMode ? 'text-slate-600' : 'text-slate-400'} opacity-60`}>
                  <span className="text-3xl">📭</span>
                  <p className="text-xs font-bold uppercase tracking-widest">尚無事件記錄</p>
                  <p className="text-[10px]">系統啟動後新增的事件將會在這裡顯示</p>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </>
  );
}

function ProgressBar({ label, status, color, width, darkMode }) {
  const colors = {
    emerald: "from-emerald-600 to-emerald-400 shadow-[0_0_15px_rgba(16,185,129,0.3)]",
    blue:    "from-blue-600 to-blue-400 shadow-[0_0_15px_rgba(59,130,246,0.3)]",
    amber:   "from-amber-600 to-amber-400 shadow-[0_0_15px_rgba(245,158,11,0.3)]"
  };
  return (
    <div className="space-y-3">
      <div className={`flex justify-between text-[10px] ${darkMode ? 'text-slate-400' : 'text-slate-500'} uppercase font-bold tracking-tighter`}>
        <span>{label}</span>
        <span className={`text-${color}-400`}>{status}</span>
      </div>
      <div className={`w-full ${darkMode ? 'bg-slate-800/50 border-slate-700/50' : 'bg-slate-200 border-slate-300'} h-2 rounded-full overflow-hidden border`}>
        <div className={`bg-gradient-to-r ${colors[color]} h-full ${width} ${color === 'emerald' ? 'animate-pulse' : ''}`} />
      </div>
    </div>
  );
}

export default ActivityDashboard;