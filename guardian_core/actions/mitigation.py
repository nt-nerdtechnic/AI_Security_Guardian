import os
import subprocess
import logging
import psutil

logger = logging.getLogger('AI_Guardian')

def kill_process_by_pid(pid):
    """
    Kills a process by its PID.
    """
    try:
        pid = int(pid)
        # Using psutil for a more cross-platform/clean kill first
        p = psutil.Process(pid)
        p.terminate()
        logger.info(f"Successfully terminated process {pid} ({p.name()})")
        return True
    except psutil.NoSuchProcess:
        logger.warning(f"Process {pid} no longer exists.")
        return False
    except Exception as e:
        # Fallback to shell kill -9 if psutil fails or permission issues
        try:
            subprocess.run(['kill', '-9', str(pid)], check=True)
            logger.info(f"Successfully killed process {pid} via SIGKILL fallback.")
            return True
        except Exception as e2:
            logger.error(f"Failed to kill process {pid}: {e2}")
            return False

def isolate_process(pid):
    """
    Isolates a process by suspending it (SIGSTOP) instead of killing it.
    Useful for forensic analysis or 'Freezing' suspicious behavior.
    """
    try:
        pid = int(pid)
        p = psutil.Process(pid)
        p.suspend()
        logger.warning(f"🛡️ [ISOLATION] Process {pid} ({p.name()}) has been SUSPENDED (Frozen).")
        return True
    except Exception as e:
        logger.error(f"Failed to isolate process {pid}: {e}")
        return False

def resume_process(pid):
    """
    Resumes a previously isolated/suspended process.
    """
    try:
        pid = int(pid)
        p = psutil.Process(pid)
        p.resume()
        logger.info(f"Process {pid} ({p.name()}) has been RESUMED.")
        return True
    except Exception as e:
        logger.error(f"Failed to resume process {pid}: {e}")
        return False

def block_network_port(port):
    """
    Placeholder for future network blocking logic.
    On macOS, this typically requires pfctl or complex osascript.
    """
    logger.warning(f"Network blocking for port {port} is not yet implemented (Requires root/pfctl).")
    return False
