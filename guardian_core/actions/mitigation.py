import os
import subprocess
import logging

logger = logging.getLogger('AI_Guardian')

def kill_process_by_pid(pid):
    """
    Kills a process by its PID.
    """
    try:
        pid = int(pid)
        subprocess.run(['kill', '-9', str(pid)], check=True)
        logger.info(f"Successfully killed process {pid}")
        return True
    except Exception as e:
        logger.error(f"Failed to kill process {pid}: {e}")
        return False

def block_network_port(port):
    """
    Placeholder for future network blocking logic.
    On macOS, this typically requires pfctl or complex osascript.
    """
    logger.warning(f"Network blocking for port {port} is not yet implemented (Requires root/pfctl).")
    return False
