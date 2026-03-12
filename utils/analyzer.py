import json
import os


def analyze_incidents():
    log_path = os.path.expanduser("~/Desktop/AI_Security_Guardian/logs/incidents.json")

    if not os.path.exists(log_path):
        print(f"Error: Log file not found at {log_path}")
        return

    try:
        with open(log_path, "r") as f:
            incidents = json.load(f)
    except Exception as e:
        print(f"Error reading log file: {e}")
        return

    total_count = len(incidents)
    module_breakdown = {}

    for incident in incidents:
        module = incident.get("module", "unknown")
        module_breakdown[module] = module_breakdown.get(module, 0) + 1

    print("--- Incident Summary ---")
    print(f"Total Incidents: {total_count}")
    print("Module Breakdown:")
    for module, count in module_breakdown.items():
        print(f"  - {module}: {count}")


if __name__ == "__main__":
    analyze_incidents()
