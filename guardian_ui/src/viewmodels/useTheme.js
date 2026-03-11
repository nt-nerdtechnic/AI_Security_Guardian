import { useState, useEffect } from 'react';

export function useTheme() {
  const [darkMode, setDarkMode] = useState(() => {
    const saved = localStorage.getItem('aegis-theme');
    return saved ? saved === 'dark' : true;
  });

  useEffect(() => {
    localStorage.setItem('aegis-theme', darkMode ? 'dark' : 'light');
  }, [darkMode]);

  const toggleTheme = () => setDarkMode(prev => !prev);

  const themeTokens = darkMode ? {
    bg:           'bg-slate-950',
    surface:      'bg-slate-900/40',
    border:       'border-slate-800/60',
    text:         'text-slate-100',
    textMuted:    'text-slate-500',
    statusBg:     'bg-slate-900/50',
    statusBorder: 'border-slate-800',
    toggleBg:     'bg-slate-800 hover:bg-slate-700',
    toggleText:   'text-amber-400',
    footerText:   'text-slate-700',
    footerDot:    'bg-slate-800',
    footerSub:    'text-slate-600',
  } : {
    bg:           'bg-slate-100',
    surface:      'bg-white/80',
    border:       'border-slate-200',
    text:         'text-slate-900',
    textMuted:    'text-slate-400',
    statusBg:     'bg-white/70',
    statusBorder: 'border-slate-300',
    toggleBg:     'bg-slate-200 hover:bg-slate-300',
    toggleText:   'text-slate-700',
    footerText:   'text-slate-400',
    footerDot:    'bg-slate-300',
    footerSub:    'text-slate-400',
  };

  return { darkMode, toggleTheme, th: themeTokens };
}
