import time
import os
import logging
import psutil
import threading

logger = logging.getLogger('Aegis_Guardian')
i18n = None

class SystemHeartbeat(threading.Thread):
    """
    (Controller)
    系統健康檢查心跳，每 1 分鐘紀錄一次運行狀態。
    """
    def __init__(self, monitors):
        super().__init__()
        self.monitors = monitors
        self.daemon = True
        self.running = True

    def run(self):
        logger.info(i18n.get('heartbeat_started') if i18n else "Heartbeat started")
        process = psutil.Process(os.getpid())
        while self.running:
            active_modules = [m.__class__.__name__ for m in self.monitors if m.is_alive()]
            
            mem_info = process.memory_info()
            cpu_usage = process.cpu_percent(interval=None)
            mem_mb = mem_info.rss / 1024 / 1024
            
            if i18n:
                status_msg = i18n.get('heartbeat_status', module_count=len(active_modules), cpu=cpu_usage, mem=round(mem_mb, 1))
            else:
                status_msg = f"Heartbeat: {len(active_modules)} modules active, CPU {cpu_usage}%, RAM {round(mem_mb, 1)}MB"
            logger.info(status_msg)
            
            MAX_MEM_MB = 200 
            MAX_CPU_PERCENT = 80

            if mem_mb > MAX_MEM_MB:
                msg = i18n.get('mem_leak_shutdown', mem=round(mem_mb, 1)) if i18n else f"Memory leak detected: {round(mem_mb,1)}MB"
                logger.critical(msg)
                os._exit(1)

            if cpu_usage > MAX_CPU_PERCENT:
                msg = i18n.get('resource_cpu_high', cpu=cpu_usage) if i18n else f"High CPU: {cpu_usage}%"
                logger.warning(msg)
                
            time.sleep(60)

    def stop(self):
        self.running = False
