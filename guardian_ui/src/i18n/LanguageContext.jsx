import React, { createContext, useContext, useState, useCallback } from 'react';
import { getTranslation, getNextLang, STORAGE_KEY, DEFAULT_LANG } from './index.js';

const LanguageContext = createContext(null);

export function LanguageProvider({ children }) {
  const [language, setLanguageState] = useState(() => {
    return localStorage.getItem(STORAGE_KEY) || DEFAULT_LANG;
  });

  const setLanguage = useCallback((lang) => {
    localStorage.setItem(STORAGE_KEY, lang);
    setLanguageState(lang);
  }, []);

  const cycleLang = useCallback(() => {
    setLanguage(getNextLang(language));
  }, [language, setLanguage]);

  const t = useCallback((key, vars) => {
    return getTranslation(language, key, vars);
  }, [language]);

  return (
    <LanguageContext.Provider value={{ language, setLanguage, cycleLang, t }}>
      {children}
    </LanguageContext.Provider>
  );
}

export function useLanguage() {
  const ctx = useContext(LanguageContext);
  if (!ctx) throw new Error('useLanguage must be used within LanguageProvider');
  return ctx;
}
