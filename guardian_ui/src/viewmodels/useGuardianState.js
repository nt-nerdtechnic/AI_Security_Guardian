import { useState, useCallback, useEffect } from 'react';
import { TauriApi } from '../models/tauriApi';

export function useGuardianState(t) {
  const [stats, setStats] = useState({
    total_blocked: 0,
    sensitive_keys: 0,
    visual_alerts: 0
  });
  
  const [events, setEvents] = useState({ threats: [], keys: [], visual: [] });
  const [loading, setLoading] = useState(true);

  const fetchStats = useCallback(async () => {
    try {
      const realData = await TauriApi.getRealActivities();
      setStats(realData.stats);
      
      const sortDesc = (a, b) => b._ts - a._ts;
      setEvents({
        threats: realData.threats.sort(sortDesc).slice(0, 50),
        keys:    realData.keys.sort(sortDesc).slice(0, 50),
        visual:  realData.visual.sort(sortDesc).slice(0, 50),
      });
      setLoading(false);
    } catch (e) {
      console.error("Failed to fetch stats", e);
      setLoading(false);
    }
  }, [t]);

  // 定期輪詢
  useEffect(() => {
    fetchStats();
    const interval = setInterval(fetchStats, 5000);
    return () => clearInterval(interval);
  }, [fetchStats]);

  return { stats, events, loading };
}
