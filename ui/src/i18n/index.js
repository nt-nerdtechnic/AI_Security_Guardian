import zhTW from './locales/zh-TW.json';
import zhCN from './locales/zh-CN.json';
import en from './locales/en.json';

const LOCALES = { 'zh-TW': zhTW, 'zh-CN': zhCN, en };
const LANG_CYCLE = ['zh-TW', 'zh-CN', 'en'];
const STORAGE_KEY = 'aegis-language';
const DEFAULT_LANG = 'zh-TW';

export { LOCALES, LANG_CYCLE, STORAGE_KEY, DEFAULT_LANG };

/**
 * 從巢狀語系物件中取得翻譯字串，支援 dot notation。
 * 例如：t('network.btn_deny') => "拒絕（終止）"
 * 支援插值佔位符，例如：t('network.total_ports', { count: 5 }) => "共 5 個 Port"
 */
export function getTranslation(lang, key, vars = {}) {
  const locale = LOCALES[lang] || LOCALES[DEFAULT_LANG];
  const keys = key.split('.');
  let value = locale;
  for (const k of keys) {
    value = value?.[k];
    if (value === undefined) break;
  }
  if (typeof value !== 'string') {
    // fallback: try English
    let fallback = LOCALES['en'];
    for (const k of keys) {
      fallback = fallback?.[k];
      if (fallback === undefined) break;
    }
    value = typeof fallback === 'string' ? fallback : key;
  }
  // Replace {var} placeholders
  return value.replace(/\{(\w+)\}/g, (_, v) => vars[v] ?? `{${v}}`);
}

/** 取得下一個語言代碼（循環切換） */
export function getNextLang(currentLang) {
  const idx = LANG_CYCLE.indexOf(currentLang);
  return LANG_CYCLE[(idx + 1) % LANG_CYCLE.length];
}
