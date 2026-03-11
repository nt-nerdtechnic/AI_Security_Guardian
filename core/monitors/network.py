import time
import subprocess
import logging
import threading
from core.models.incident import IncidentLogger

logger = logging.getLogger('Aegis_Guardian')

i18n = None

class NetworkMonitor(threading.Thread):
    def __init__(self, config, notifier, mitigator=None):
        super().__init__()
        self.config = config.get('network_monitor', {})
        self.notifier = notifier
        self.mitigator = mitigator
        self.daemon = True
        self.running = True
        self.interval = self.config.get('check_interval', 5)
        self.threshold = self.config.get('high_bandwidth_threshold_mb', 50) * 1024 * 1024
        self.suspicious_ports = self.config.get('suspicious_ports', [])

    def run(self):
        logger.info(i18n.get('network_monitor_started') if i18n else "Network monitor started")
        while self.running:
            try:
                output = subprocess.check_output(['lsof', '-i', '-nP'], stderr=subprocess.STDOUT).decode()
                lines = output.splitlines()
                if len(lines) > 1:
                    for line in lines[1:]:
                        parts = line.split()
                        if len(parts) < 9: continue
                        
                        command = parts[0]
                        pid = parts[1]
                        remote_info = parts[8]
                        if '->' in remote_info:
                            remote_addr = remote_info.split('->')[1]
                            if ':' in remote_addr:
                                port_str = remote_addr.split(':')[-1]
                                try:
                                    port = int(port_str)
                                    if port in self.suspicious_ports:
                                        msg = i18n.get('network_alert', port=port, command=command, pid=pid) if i18n else f"Suspicious port: {port}"
                                        logger.warning(msg)
                                        IncidentLogger.record(
                                            module="NetworkMonitor",
                                            severity="WARNING",
                                            message="Suspicious outbound port detected",
                                            metadata={"port": port, "pid": pid, "command": command}
                                        )
                                        
                                        if self.mitigator:
                                            self.mitigator.auto_mitigate_incident(
                                                module="NetworkMonitor",
                                                severity="WARNING",
                                                metadata={"port": port, "pid": pid, "command": command}
                                            )

                                        self.notifier.send_interactive_alert(msg, [
                                            [
                                                {"text": "🛡️ Quarantine", "callback_data": f"quarantine|{parts[8]}"},
                                                {"text": "🛑 Terminate", "callback_data": f"terminate|{pid}"}
                                            ],
                                            [
                                                {"text": "🙈 Ignore", "callback_data": f"ignore|{pid}"}
                                            ]
                                        ])
                                except:
                                    continue
            except Exception as e:
                pass
            
            time.sleep(self.interval)

    def stop(self):
        self.running = False
