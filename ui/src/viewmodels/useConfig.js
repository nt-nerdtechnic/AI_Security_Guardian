import { useState, useCallback, useEffect } from 'react';
import { TauriApi } from '../models/tauriApi';

export function useConfig() {
  const [mode, setMode] = useState('Silent Monitor');
  const [modules, setModules] = useState({
    visual: true,
    clipboard: true,
    network: true
  });
  const [configLoading, setConfigLoading] = useState(true);

  const fetchConfig = useCallback(async () => {
    try {
      const config = await TauriApi.getConfig();
      setMode(config.mode);
      setModules(config.modules);
      setConfigLoading(false);
    } catch (e) {
      console.error("Failed to fetch config", e);
      setConfigLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchConfig();
  }, [fetchConfig]);

  const updateConfig = async (newMode, newModules) => {
    try {
      await TauriApi.updateConfig(newMode, newModules);
    } catch (e) {
      console.error("Failed to update config", e);
    }
  };

  const handleModeChange = (m) => {
    setMode(m);
    updateConfig(m, modules);
  };

  const toggleModule = (mod) => {
    const next = { ...modules, [mod]: !modules[mod] };
    setModules(next);
    updateConfig(mode, next);
  };

  return { mode, modules, configLoading, handleModeChange, toggleModule };
}
