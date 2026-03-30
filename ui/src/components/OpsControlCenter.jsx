import React, { useCallback, useEffect, useMemo, useState } from 'react';
import {
  X, RefreshCw, Server, Activity, CalendarClock,
  Play, Pause, Terminal, Trash2,
} from 'lucide-react';
import { TauriApi } from '../models/tauriApi';
import { useLanguage } from '../i18n/LanguageContext';

// ── Confirmation dialog ───────────────────────────────────────────────────────
function ConfirmDialog({ name, onConfirm, onCancel, t }) {
  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onCancel} />
      <div className="relative bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-[22px] shadow-2xl p-6 mx-4 max-w-sm w-full">
        <h4 className="text-sm font-bold text-slate-800 dark:text-slate-100 mb-1">{t('ops_center.confirm_title')}</h4>
        <p className="text-[12px] text-slate-500 mb-4 break-all">{name}</p>
        <div className="flex items-center justify-end gap-2">
          <button
            onClick={onCancel}
            className="px-3 py-1.5 rounded-lg text-[11px] font-bold border border-slate-200 dark:border-slate-700 text-slate-500 hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors"
          >{t('ops_center.confirm_cancel')}</button>
          <button
            onClick={() => { onConfirm(); onCancel(); }}
            className="px-3 py-1.5 rounded-lg text-[11px] font-bold bg-rose-500 text-white hover:bg-rose-600 transition-colors"
          >{t('ops_center.confirm_ok')}</button>
        </div>
      </div>
    </div>
  );
}

// ── KPI card ─────────────────────────────────────────────────────────────────
function KpiCard({ label, value, color, onClick }) {
  return (
    <button
      onClick={onClick}
      className="bg-slate-50 dark:bg-slate-900/20 border border-slate-200 dark:border-slate-800 rounded-[22px] p-4 shadow-sm text-center transition-all hover:scale-[1.02] hover:shadow-md hover:bg-white dark:hover:bg-slate-800/40 group active:scale-95 flex-1"
    >
      <div className={`text-2xl font-black ${color} group-hover:drop-shadow-[0_0_8px_rgba(99,102,241,0.3)] transition-all`}>{value}</div>
      <div className="text-[10px] text-slate-500 mt-0.5 tracking-wide group-hover:text-slate-700 dark:group-hover:text-slate-300 transition-colors">{label}</div>
    </button>
  );
}

// ── Filter tab strip ──────────────────────────────────────────────────────────
function FilterTabs({ options, current, onChange, accentClass }) {
  return (
    <div className="flex items-center gap-1">
      {options.map(({ key, label, Icon }) => {
        const active = current === key;
        return (
          <button
            key={key}
            onClick={() => onChange(key)}
            title={label}
            className={`flex items-center gap-1 px-2 py-0.5 rounded-lg text-[9px] font-bold transition-all border ${
              active ? `${accentClass} text-white` : 'bg-white dark:bg-slate-900 text-slate-400 border-slate-100 dark:border-slate-800'
            }`}
          >
            <Icon size={8} />
            <span className="hidden sm:inline">{label}</span>
          </button>
        );
      })}
    </div>
  );
}

// ── Section container ─────────────────────────────────────────────────────────
function Section({ id, accentColor, children }) {
  return (
    <div
      id={id}
      className="bg-slate-50 dark:bg-slate-900/20 border border-slate-200 dark:border-slate-800 rounded-[28px] shadow-sm overflow-hidden scroll-mt-2"
    >
      <div style={{ height: '2px', background: `linear-gradient(to right,transparent,${accentColor},transparent)` }} />
      <div className="p-5 space-y-2">{children}</div>
    </div>
  );
}

// ── Section header row ────────────────────────────────────────────────────────
function SectionHeader({ icon, title, count, filterSlot, loading, onRefresh, lastScanned }) {
  return (
    <div className="flex items-center gap-2 mb-3">
      {icon}
      <span className="text-[10px] font-black uppercase tracking-[0.18em] text-slate-600 dark:text-slate-400">{title}</span>
      <span className="text-[9px] text-slate-400">{count}</span>
      <div className="flex items-center gap-1 ml-auto">
        {filterSlot}
        <div className="w-px h-3 bg-slate-200 dark:bg-slate-700 mx-1" />
        <button
          onClick={onRefresh}
          title={lastScanned ? `上次掃描: ${lastScanned.toLocaleTimeString('zh-TW', { hour: '2-digit', minute: '2-digit', second: '2-digit' })}` : '刷新'}
          className="p-1.5 rounded-lg border border-slate-200 dark:border-slate-700 text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 transition-all"
        >
          <RefreshCw size={9} className={loading ? 'animate-spin' : ''} />
        </button>
      </div>
    </div>
  );
}

// ── Empty placeholder ─────────────────────────────────────────────────────────
function Empty({ icon, text }) {
  return (
    <div className="flex flex-col items-center justify-center py-10 text-slate-400">
      <div className="mb-2 opacity-25">{icon}</div>
      <span className="text-sm">{text}</span>
    </div>
  );
}

// ── Status badge ──────────────────────────────────────────────────────────────
function Badge({ active, activeLabel, inactiveLabel, activeClass = 'bg-emerald-50 dark:bg-emerald-950/40 text-emerald-600 dark:text-emerald-400 border-emerald-200 dark:border-emerald-800/40' }) {
  return (
    <span className={`shrink-0 text-[9px] font-bold px-1.5 py-0.5 rounded-md border ${
      active ? activeClass : 'bg-slate-100 dark:bg-slate-800 text-slate-400 border-slate-200 dark:border-slate-700'
    }`}>
      {active ? activeLabel : inactiveLabel}
    </span>
  );
}

// ── Sessions section ──────────────────────────────────────────────────────────
function SessionsSection({ sessions, loading, onRefresh, lastScanned, t }) {
  const [filter, setFilter] = useState('all');

  const filterOpts = [
    { key: 'all', label: t('ops_center.filter_all'), Icon: Activity },
    { key: 'high', label: t('ops_center.filter_high_cpu'), Icon: Play },
  ];

  const visible = useMemo(() => {
    if (filter === 'high') return sessions.filter(s => s.cpu_usage > 5);
    return sessions;
  }, [sessions, filter]);

  return (
    <Section id="ops-sessions-section" accentColor="rgba(99,102,241,0.5)">
      <SectionHeader
        icon={<Activity size={12} className="text-indigo-500" />}
        title={t('ops_center.sessions_title')}
        count={`${visible.length} / ${sessions.length}`}
        loading={loading}
        onRefresh={onRefresh}
        lastScanned={lastScanned}
        filterSlot={
          <FilterTabs
            options={filterOpts}
            current={filter}
            onChange={setFilter}
            accentClass="bg-indigo-500 border-indigo-500"
          />
        }
      />

      {sessions.length === 0 ? (
        <Empty icon={<Activity size={28} />} text={t('ops_center.empty_sessions')} />
      ) : visible.length === 0 ? (
        <Empty icon={<Activity size={28} />} text={t('ops_center.empty_sessions_filter')} />
      ) : (
        <div className="space-y-1.5 max-h-[360px] overflow-y-auto pr-0.5">
          {visible.map((item) => (
            <div
              key={item.session_id}
              className={`rounded-2xl border px-3.5 py-2.5 transition-all ${
                item.cpu_usage > 5
                  ? 'border-violet-200 dark:border-violet-800/40 bg-violet-50/30 dark:bg-violet-950/10'
                  : 'border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900/50'
              }`}
            >
              <div className="flex items-center gap-2">
                <div className={`shrink-0 w-5 h-5 rounded-lg flex items-center justify-center ${
                  item.cpu_usage > 5 ? 'bg-violet-100 dark:bg-violet-900/40' : 'bg-slate-100 dark:bg-slate-800/40'
                }`}>
                  {item.cpu_usage > 5
                    ? <div className="w-2 h-2 rounded-full bg-violet-500 animate-pulse" />
                    : <Activity size={10} className="text-slate-500" />}
                </div>
                <div className="flex-1 min-w-0">
                  <span className="block text-[12px] font-semibold text-slate-800 dark:text-slate-100 truncate">{item.name}</span>
                  <span className="block text-[10px] text-slate-400 font-mono">
                    PID {item.pid} · {item.status}
                  </span>
                </div>
                <div className="flex items-center gap-2 shrink-0 text-[10px] font-mono">
                  <span className={item.cpu_usage > 10 ? 'text-amber-500 font-bold' : 'text-slate-400'}>
                    CPU {item.cpu_usage.toFixed(1)}%
                  </span>
                  <span className="text-blue-400">RAM {item.memory_mb.toFixed(0)} MB</span>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </Section>
  );
}

// ── LaunchAgents section ──────────────────────────────────────────────────────
function LaunchAgentsSection({ agents, loading, onRefresh, lastScanned, t }) {
  const [filter, setFilter] = useState('running');

  const filterOpts = [
    { key: 'all', label: t('ops_center.filter_all'), Icon: Activity },
    { key: 'running', label: t('ops_center.filter_running'), Icon: Play },
    { key: 'stopped', label: t('ops_center.filter_stopped'), Icon: Pause },
  ];

  const visible = useMemo(() => {
    if (filter === 'running') return agents.filter(a => a.loaded);
    if (filter === 'stopped') return agents.filter(a => !a.loaded);
    return agents;
  }, [agents, filter]);

  return (
    <Section id="ops-agents-section" accentColor="rgba(16,185,129,0.45)">
      <SectionHeader
        icon={<Server size={12} className="text-emerald-500" />}
        title={t('ops_center.launch_agents_title')}
        count={`${agents.filter(a => a.loaded).length} / ${agents.length} ${t('ops_center.running_suffix')}`}
        loading={loading}
        onRefresh={onRefresh}
        lastScanned={lastScanned}
        filterSlot={
          <FilterTabs
            options={filterOpts}
            current={filter}
            onChange={setFilter}
            accentClass="bg-emerald-500 border-emerald-500"
          />
        }
      />

      {agents.length === 0 ? (
        <Empty icon={<Server size={22} />} text={t('ops_center.empty_launch_agents')} />
      ) : visible.length === 0 ? (
        <Empty icon={<Server size={22} />} text={t('ops_center.empty_launch_agents_filter')} />
      ) : (
        <div className="space-y-1.5 max-h-[360px] overflow-y-auto pr-0.5">
          {visible.map((agent) => (
            <div
              key={agent.plist_path}
              className={`rounded-xl border px-3 py-2.5 transition-all ${
                agent.loaded
                  ? 'border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900/50'
                  : 'border-slate-100 dark:border-slate-800 bg-slate-50/40 dark:bg-slate-900/20 opacity-60'
              }`}
            >
              <div className="flex items-center gap-2">
                <Badge
                  active={agent.loaded}
                  activeLabel={t('ops_center.status_loaded')}
                  inactiveLabel={t('ops_center.status_unloaded')}
                  activeClass="bg-emerald-50 dark:bg-emerald-950/40 text-emerald-600 dark:text-emerald-400 border-emerald-200 dark:border-emerald-800/40"
                />
                <span className="flex-1 min-w-0 text-[11px] font-semibold text-slate-800 dark:text-slate-100 truncate">{agent.label}</span>
              </div>
              <div className="mt-1 flex items-center gap-2 text-[9px] text-slate-400 flex-wrap">
                <span className="font-mono truncate">{agent.program || '-'}</span>
                {(agent.run_at_load || agent.keep_alive) && (
                  <>
                    <span className="opacity-40">·</span>
                    <span className="font-mono text-violet-400/80">
                      {agent.keep_alive ? t('ops_center.keep_alive') : t('ops_center.run_at_load')}
                    </span>
                  </>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </Section>
  );
}

// ── Crontab section ───────────────────────────────────────────────────────────
function CrontabSection({ entries, loading, onRefresh, lastScanned, t }) {
  const [filter, setFilter] = useState('all');

  const filterOpts = [
    { key: 'all', label: t('ops_center.filter_all'), Icon: Activity },
    { key: 'normal', label: t('ops_center.filter_normal'), Icon: Play },
    { key: 'scheduled', label: t('ops_center.filter_scheduled'), Icon: CalendarClock },
  ];

  const visible = useMemo(() => {
    if (filter === 'normal') return entries.filter(e => !e.schedule.startsWith('@'));
    if (filter === 'scheduled') return entries.filter(e => e.schedule.startsWith('@'));
    return entries;
  }, [entries, filter]);

  return (
    <Section id="ops-crontab-section" accentColor="rgba(245,158,11,0.45)">
      <SectionHeader
        icon={<Terminal size={12} className="text-amber-500" />}
        title={t('ops_center.crontab_title')}
        count={`${entries.length} ${t('ops_center.entries_suffix')}`}
        loading={loading}
        onRefresh={onRefresh}
        lastScanned={lastScanned}
        filterSlot={
          <FilterTabs
            options={filterOpts}
            current={filter}
            onChange={setFilter}
            accentClass="bg-amber-500 border-amber-500"
          />
        }
      />

      {entries.length === 0 ? (
        <Empty icon={<Terminal size={22} />} text={t('ops_center.empty_crontab')} />
      ) : visible.length === 0 ? (
        <Empty icon={<Terminal size={22} />} text={t('ops_center.empty_crontab_filter')} />
      ) : (
        <div className="space-y-1.5 max-h-[360px] overflow-y-auto pr-0.5">
          {visible.map((item, idx) => (
            <div
              key={`${item.schedule}-${idx}`}
              className="rounded-xl border px-3 py-2.5 transition-all border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900/50"
            >
              <div className="flex items-center gap-2">
                <span className="shrink-0 text-[9px] font-mono font-bold px-1.5 py-0.5 rounded-md border bg-amber-50 dark:bg-amber-950/40 text-amber-600 dark:text-amber-400 border-amber-200 dark:border-amber-800/40">
                  {item.schedule}
                </span>
                <span className="flex-1 min-w-0 text-[11px] font-semibold text-slate-800 dark:text-slate-100 truncate">
                  {item.command.split('/').slice(-1)[0] || item.command}
                </span>
              </div>
              <div className="mt-1 text-[9px] text-slate-400 font-mono truncate opacity-60">
                {item.command}
              </div>
            </div>
          ))}
        </div>
      )}
    </Section>
  );
}

// ── Main drawer ───────────────────────────────────────────────────────────────
const OpsControlCenter = ({ darkMode, theme, onClose }) => {
  const { t } = useLanguage();

  const [crontab, setCrontab]         = useState([]);
  const [launchAgents, setLaunchAgents] = useState([]);
  const [sessions, setSessions]       = useState([]);

  const [systemLoading, setSystemLoading]   = useState(false);
  const [sessionLoading, setSessionLoading] = useState(false);
  const [initLoading, setInitLoading]       = useState(true);

  const [lastSystem, setLastSystem]   = useState(null);
  const [lastSession, setLastSession] = useState(null);
  const [error, setError]             = useState('');

  const loadSystem = useCallback(async () => {
    try {
      const [cronData, launchData] = await Promise.all([
        TauriApi.opsListSystemCrontab(),
        TauriApi.opsListLaunchAgents(),
      ]);
      setCrontab(Array.isArray(cronData)   ? cronData   : []);
      setLaunchAgents(Array.isArray(launchData) ? launchData : []);
      setLastSystem(new Date());
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const loadSessions = useCallback(async () => {
    try {
      const data = await TauriApi.opsListActiveSessions();
      setSessions(Array.isArray(data) ? data : []);
      setLastSession(new Date());
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const loadAll = useCallback(async () => {
    setError('');
    await Promise.all([loadSystem(), loadSessions()]);
    setInitLoading(false);
  }, [loadSystem, loadSessions]);

  useEffect(() => { loadAll(); }, [loadAll]);

  // crontab & launchagents: every 30s
  useEffect(() => {
    const id = setInterval(() => { loadSystem().catch(() => {}); }, 30000);
    return () => clearInterval(id);
  }, [loadSystem]);

  // sessions: every 5s
  useEffect(() => {
    const id = setInterval(() => { loadSessions().catch(() => {}); }, 5000);
    return () => clearInterval(id);
  }, [loadSessions]);

  const scrollToSection = (id) => {
    document.getElementById(id)?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  };

  const kpi = useMemo(() => ({
    sessions:     sessions.length,
    highCpu:      sessions.filter(s => s.cpu_usage > 5).length,
    agentsRunning: launchAgents.filter(a => a.loaded).length,
    agentsTotal:   launchAgents.length,
    crontabCount:  crontab.length,
  }), [sessions, launchAgents, crontab]);

  return (
    <>
      {/* Backdrop */}
      <div className="fixed inset-0 bg-black/40 backdrop-blur-[1px] z-40" onClick={onClose} />

      {/* Drawer */}
      <aside className={`fixed top-0 right-0 h-full w-[min(1040px,96vw)] z-50 border-l ${theme.border} ${theme.bg} shadow-2xl flex flex-col`}>

        {/* Header */}
        <div className={`px-6 py-4 border-b ${theme.border} flex items-center justify-between shrink-0`}>
          <div>
            <h2 className={`text-sm font-black uppercase tracking-[0.2em] ${theme.textMuted}`}>{t('ops_center.title')}</h2>
            <p className="text-[11px] mt-0.5 text-slate-500">{t('ops_center.subtitle')}</p>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={async () => { setSystemLoading(true); await loadAll(); setSystemLoading(false); }}
              className={`flex items-center gap-1.5 px-3 py-2 rounded-xl border text-[11px] font-bold uppercase tracking-wider ${darkMode ? 'border-slate-700 text-slate-300 hover:bg-slate-800' : 'border-slate-300 text-slate-600 hover:bg-slate-100'} transition-colors`}
            >
              <RefreshCw size={13} className={systemLoading ? 'animate-spin' : ''} />
              {t('ops_center.refresh')}
            </button>
            <button
              onClick={onClose}
              className={`p-2 rounded-lg border ${darkMode ? 'border-slate-700 text-slate-400 hover:bg-slate-800' : 'border-slate-300 text-slate-600 hover:bg-slate-100'} transition-colors`}
            >
              <X size={16} />
            </button>
          </div>
        </div>

        {/* KPI row (scroll anchors) */}
        <div className={`px-6 py-3 border-b ${theme.border} shrink-0`}>
          <div className="flex gap-3">
            <KpiCard
              label={t('ops_center.kpi_sessions')}
              value={kpi.sessions}
              color="text-indigo-600 dark:text-indigo-400"
              onClick={() => scrollToSection('ops-sessions-section')}
            />
            <KpiCard
              label={t('ops_center.kpi_high_cpu')}
              value={kpi.highCpu}
              color={kpi.highCpu > 0 ? 'text-amber-500' : 'text-emerald-600 dark:text-emerald-400'}
              onClick={() => scrollToSection('ops-sessions-section')}
            />
            <KpiCard
              label={t('ops_center.kpi_agents')}
              value={`${kpi.agentsRunning}/${kpi.agentsTotal}`}
              color={kpi.agentsRunning < kpi.agentsTotal ? 'text-amber-500' : 'text-emerald-600 dark:text-emerald-400'}
              onClick={() => scrollToSection('ops-agents-section')}
            />
            <KpiCard
              label={t('ops_center.kpi_crontab')}
              value={kpi.crontabCount}
              color="text-amber-600 dark:text-amber-400"
              onClick={() => scrollToSection('ops-crontab-section')}
            />
          </div>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto px-6 py-5 space-y-5">
          {initLoading ? (
            <div className="flex items-center justify-center h-48 text-slate-400">
              <RefreshCw size={22} className="animate-spin" />
            </div>
          ) : (
            <>
              {error && (
                <div className="text-xs p-3 rounded-xl border bg-rose-500/10 border-rose-500/30 text-rose-400">
                  {t('ops_center.load_error')}: {error}
                </div>
              )}

              {/* Sessions: full-width */}
              <SessionsSection
                sessions={sessions}
                loading={sessionLoading}
                onRefresh={async () => { setSessionLoading(true); await loadSessions(); setSessionLoading(false); }}
                lastScanned={lastSession}
                t={t}
              />

              {/* LaunchAgents + Crontab: 2-col grid */}
              <div className="grid grid-cols-1 xl:grid-cols-2 gap-5">
                <LaunchAgentsSection
                  agents={launchAgents}
                  loading={systemLoading}
                  onRefresh={async () => { setSystemLoading(true); await loadSystem(); setSystemLoading(false); }}
                  lastScanned={lastSystem}
                  t={t}
                />
                <CrontabSection
                  entries={crontab}
                  loading={systemLoading}
                  onRefresh={async () => { setSystemLoading(true); await loadSystem(); setSystemLoading(false); }}
                  lastScanned={lastSystem}
                  t={t}
                />
              </div>
            </>
          )}
        </div>
      </aside>
    </>
  );
};

export default OpsControlCenter;
