import time
import psutil
import logging
import threading
from core.models.incident import IncidentLogger

logger = logging.getLogger('Aegis_Guardian')

i18n = None

class SystemResourceMonitor(threading.Thread):
    def __init__(self, config, notifier, mitigator=None):
        super().__init__()
        self.config = config.get('system_resource_monitor', {})
        self.notifier = notifier
        self.mitigator = mitigator
        self.daemon = True
        self.running = True
        
        self.interval = self.config.get('check_interval', 10)
        self.cpu_threshold = self.config.get('cpu_threshold_percent', 90)
        self.ram_threshold = self.config.get('ram_threshold_percent', 90)
        self.disk_threshold = self.config.get('disk_threshold_percent', 90)

    def run(self):
        logger.info(i18n.get('system_resource_monitor_started') if i18n else "System Resource Monitor started")
        
        while self.running:
            try:
                # CPU
                cpu_percent = psutil.cpu_percent(interval=None)
                if cpu_percent >= self.cpu_threshold:
                    msg = i18n.get('resource_cpu_high', cpu=cpu_percent) if i18n else f"High CPU: {cpu_percent}%"
                    self._alert_and_log("CPU", msg, cpu_percent)
                
                # RAM
                mem = psutil.virtual_memory()
                if mem.percent >= self.ram_threshold:
                    msg = i18n.get('resource_ram_high', ram=mem.percent) if i18n else f"High RAM: {mem.percent}%"
                    self._alert_and_log("RAM", msg, mem.percent)
                
                # Disk (Root partition)
                disk = psutil.disk_usage('/')
                if disk.percent >= self.disk_threshold:
                    msg = i18n.get('resource_disk_high', disk=disk.percent) if i18n else f"High Disk: {disk.percent}%"
                    self._alert_and_log("Disk", msg, disk.percent)
                    
            except Exception as e:
                err_msg = i18n.get('system_resource_monitor_error', error=e) if i18n else f"Error in monitor: {e}"
                logger.error(err_msg, exc_info=True)
                
            time.sleep(self.interval)

    def _alert_and_log(self, resource_type, msg, value):
        logger.warning(msg)
        IncidentLogger.record(
            module="SystemResourceMonitor",
            severity="WARNING",
            message=f"High {resource_type} usage detected",
            metadata={"resource": resource_type, "value_percent": value}
        )
        
        if self.mitigator:
            self.mitigator.auto_mitigate_incident(
                module="SystemResourceMonitor",
                severity="WARNING",
                metadata={"resource": resource_type, "value_percent": value}
            )

        # Send Telegram alert without interactive buttons for simple resource alerts
        self.notifier.send_text_alert(msg)

    def stop(self):
        self.running = False
