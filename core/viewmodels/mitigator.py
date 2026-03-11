import psutil
import logging
import subprocess
import shutil
import time
from pathlib import Path
from core.models.incident import IncidentLogger

logger = logging.getLogger('Aegis_Guardian')

# 由 main.py 在初始化時注入
config_ref = {}


class MitigationManager:
    """
    (ViewModel)
    統一管理所有主動防禦動作（Mitigation Actions）。
    根據 incident 的 module/severity 決定採取何種動作。
    """

    def __init__(self, config: dict):
        self.cfg = config.get('mitigation', {})
        self.auto_mitigate = self.cfg.get('auto_mitigate', False)
        self.quarantine_dir = Path(self.cfg.get('quarantine_dir', 'quarantine'))
        self.quarantine_dir.mkdir(parents=True, exist_ok=True)
        self.actions_map = self.cfg.get('actions', {})

    # ─── 公開入口：自動根據 incident 判斷動作 ─────────────────────────────
    def auto_mitigate_incident(self, module: str, severity: str, metadata: dict) -> str:
        """
        根據 module 與 severity 自動執行防禦。
        僅在 config.mitigation.auto_mitigate = true 時生效。
        回傳描述已執行動作的字串。
        """
        if not self.auto_mitigate:
            return "auto_mitigate disabled"

        action = ""
        result = "no action taken"

        if "NetworkMonitor" in module:
            action = self.actions_map.get("network_threat", "kill_process")
        elif "SystemResourceMonitor" in module:
            action = self.actions_map.get("cpu_overload", "isolate_top_process")
        elif "Clipboard" in module:
            action = self.actions_map.get("clipboard_threat", "clear_clipboard")

        if action == "kill_process":
            pid = metadata.get("pid")
            if pid:
                ok = self.kill_process_by_pid(int(pid))
                result = f"kill_process (PID {pid}): {'ok' if ok else 'failed'}"

        elif action == "isolate_top_process":
            proc = self._get_top_cpu_process()
            if proc:
                ok = self.isolate_process(proc.pid)
                result = f"isolate_process (PID {proc.pid}, {proc.name()}): {'ok' if ok else 'failed'}"

        elif action == "clear_clipboard":
            ok = self.clear_clipboard()
            result = f"clear_clipboard: {'ok' if ok else 'failed'}"

        logger.info(f"[MitigationManager] auto_mitigate result: {result}")
        IncidentLogger.record(
            module="MitigationManager",
            severity="INFO",
            message=f"Auto-mitigation executed: {result}",
            metadata={"trigger_module": module, "action": action}
        )
        return result

    # ─── 動作函式 ─────────────────────────────────────────────────────────
    def kill_process_by_pid(self, pid: int) -> bool:
        """終止指定 PID 程序（SIGTERM，失敗則 SIGKILL）"""
        try:
            p = psutil.Process(pid)
            name = p.name()
            p.terminate()
            time.sleep(0.5)
            if p.is_running():
                p.kill()
            logger.info(f"[Mitigate] Terminated PID {pid} ({name})")
            self._log_action("kill", pid, name)
            return True
        except psutil.NoSuchProcess:
            logger.warning(f"[Mitigate] PID {pid} not found")
            return False
        except Exception as e:
            try:
                subprocess.run(['kill', '-9', str(pid)], check=True)
                logger.info(f"[Mitigate] SIGKILL fallback OK for PID {pid}")
                return True
            except Exception as e2:
                logger.error(f"[Mitigate] kill failed for PID {pid}: {e2}")
                return False

    def kill_process_tree(self, pid: int) -> bool:
        """終止程序及其所有子程序"""
        try:
            parent = psutil.Process(pid)
            children = parent.children(recursive=True)
            for child in children:
                child.kill()
            parent.kill()
            logger.info(f"[Mitigate] Killed process tree for PID {pid}")
            self._log_action("kill_tree", pid, parent.name())
            return True
        except Exception as e:
            logger.error(f"[Mitigate] kill_tree failed for PID {pid}: {e}")
            return False

    def isolate_process(self, pid: int) -> bool:
        """暫停（凍結）程序（SIGSTOP）"""
        try:
            p = psutil.Process(pid)
            p.suspend()
            logger.warning(f"[Mitigate] 🧊 Suspended PID {pid} ({p.name()})")
            self._log_action("isolate", pid, p.name())
            return True
        except Exception as e:
            logger.error(f"[Mitigate] isolate failed for PID {pid}: {e}")
            return False

    def resume_process(self, pid: int) -> bool:
        """恢復已暫停的程序（SIGCONT）"""
        try:
            p = psutil.Process(pid)
            p.resume()
            logger.info(f"[Mitigate] ▶️ Resumed PID {pid} ({p.name()})")
            self._log_action("resume", pid, p.name())
            return True
        except Exception as e:
            logger.error(f"[Mitigate] resume failed for PID {pid}: {e}")
            return False

    def quarantine_file(self, source_path: str) -> bool:
        """將可疑檔案移到隔離區"""
        try:
            src = Path(source_path)
            if not src.exists():
                return False
            dst = self.quarantine_dir / src.name
            shutil.move(str(src), str(dst))
            logger.warning(f"[Mitigate] 🔒 Quarantined {src} → {dst}")
            IncidentLogger.record(
                module="MitigationManager",
                severity="WARNING",
                message=f"File quarantined",
                metadata={"source": str(src), "destination": str(dst)}
            )
            return True
        except Exception as e:
            logger.error(f"[Mitigate] quarantine failed: {e}")
            return False

    def clear_clipboard(self) -> bool:
        """清空剪貼簿"""
        try:
            import pyperclip
            pyperclip.copy("[CLEARED BY AEGIS GUARDIAN]")
            logger.info("[Mitigate] 📋 Clipboard cleared")
            return True
        except Exception as e:
            logger.error(f"[Mitigate] clear_clipboard failed: {e}")
            return False

    def get_top_processes(self, n: int = 10) -> list:
        """
        取得目前 CPU 用量最高的前 n 個程序。
        回傳格式：[{ pid, name, cpu_percent, memory_percent, status }]
        """
        procs = []
        for proc in psutil.process_iter(['pid', 'name', 'cpu_percent', 'memory_percent', 'status']):
            try:
                info = proc.info
                if info['cpu_percent'] is not None:
                    procs.append(info)
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue
        # 給 psutil 一點時間收集 CPU 差值
        time.sleep(0.1)
        procs.sort(key=lambda p: p['cpu_percent'] or 0, reverse=True)
        return procs[:n]

    # ─── 私有輔助方法 ──────────────────────────────────────────────────────
    def _get_top_cpu_process(self):
        """找到目前 CPU 用量最高的程序"""
        try:
            procs = [(p.cpu_percent(interval=0.1), p) for p in psutil.process_iter()]
            procs.sort(reverse=True, key=lambda x: x[0])
            if procs:
                return procs[0][1]
        except Exception:
            pass
        return None

    def _log_action(self, action: str, pid: int, name: str):
        IncidentLogger.record(
            module="MitigationManager",
            severity="WARNING",
            message=f"Mitigation action executed: {action}",
            metadata={"action": action, "pid": pid, "process_name": name}
        )
