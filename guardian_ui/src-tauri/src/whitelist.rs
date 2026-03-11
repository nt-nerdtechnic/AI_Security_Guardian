// whitelist.rs — Network Port Whitelist (SQLite 持久化)
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WhitelistEntry {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub approved_at: String,
}

/// 取得 DB 路徑：存在 home dir 的 .aegis-guardian 資料夾中
pub fn db_path() -> PathBuf {
    let base = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(".aegis-guardian").join("whitelist.db")
}

/// 初始化資料庫：建立目錄與資料表
pub fn init_db() -> Result<Connection> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let conn = Connection::open(&path)?;
    // 為了保證欄位更新，先 DROP TABLE（清除舊資料）
    conn.execute_batch(
        "DROP TABLE IF EXISTS network_whitelist;
         CREATE TABLE network_whitelist (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            port          INTEGER NOT NULL UNIQUE,
            pid           INTEGER NOT NULL DEFAULT 0,
            process_name  TEXT NOT NULL DEFAULT '',
            approved_at   TEXT NOT NULL
        );"
    )?;
    Ok(conn)
}

/// 新增白名單記錄（若已存在則更新 pid, process_name 與時間）
pub fn add_whitelist(conn: &Connection, port: u16, pid: u32, process_name: &str) -> Result<()> {
    let now = chrono_now();
    conn.execute(
        "INSERT INTO network_whitelist (port, pid, process_name, approved_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(port) DO UPDATE SET pid=?2, process_name=?3, approved_at=?4",
        params![port as i64, pid as i64, process_name, now],
    )?;
    Ok(())
}

/// 移除白名單記錄
pub fn remove_whitelist(conn: &Connection, port: u16) -> Result<()> {
    conn.execute(
        "DELETE FROM network_whitelist WHERE port = ?1",
        params![port as i64],
    )?;
    Ok(())
}

/// 取得所有已核准的 port 與 pid 列表
pub fn get_whitelisted_ports(conn: &Connection) -> Result<Vec<(u16, u32)>> {
    let mut stmt = conn.prepare("SELECT port, pid FROM network_whitelist ORDER BY approved_at DESC")?;
    let ports: Vec<(u16, u32)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)? as u16,
                row.get::<_, i64>(1)? as u32,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(ports)
}

/// 自動清理已失效（不存在於目前監聽清單中，或 PID 不符）的白名單
pub fn cleanup_stale_whitelist(conn: &Connection, active_ports_and_pids: &[(u16, u32)]) -> Result<()> {
    let stored_ports = get_whitelisted_ports(conn)?;
    for (port, pid) in stored_ports {
        if !active_ports_and_pids.contains(&(port, pid)) {
            remove_whitelist(conn, port)?;
        }
    }
    Ok(())
}

/// 取得所有白名單詳細記錄
pub fn get_whitelist_entries(conn: &Connection) -> Result<Vec<WhitelistEntry>> {
    let mut stmt = conn.prepare(
        "SELECT port, pid, process_name, approved_at FROM network_whitelist ORDER BY approved_at DESC"
    )?;
    let entries = stmt
        .query_map([], |row| {
            Ok(WhitelistEntry {
                port:         row.get::<_, i64>(0)? as u16,
                pid:          row.get::<_, i64>(1)? as u32,
                process_name: row.get(2)?,
                approved_at:  row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(entries)
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // 格式化為 ISO-like 字串（不引入 chrono crate 保持輕量）
    let (y, mo, d, h, mi, s) = unix_to_datetime(secs);
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mo, d, h, mi, s)
}

fn unix_to_datetime(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    // 簡易日期計算（不需精確，夠用即可）
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    let month = day_of_year / 30 + 1;
    let day = day_of_year % 30 + 1;
    (year, month.min(12), day.min(31), h, m, s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_whitelist() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "DROP TABLE IF EXISTS network_whitelist;
             CREATE TABLE network_whitelist (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                port INTEGER NOT NULL UNIQUE,
                pid INTEGER NOT NULL DEFAULT 0,
                process_name TEXT NOT NULL DEFAULT '',
                approved_at TEXT NOT NULL
            );"
        ).unwrap();

        add_whitelist(&conn, 3000, 1234, "node").unwrap();
        add_whitelist(&conn, 8080, 5678, "nginx").unwrap();

        let ports = get_whitelisted_ports(&conn).unwrap();
        assert!(ports.contains(&(3000, 1234)));
        assert!(ports.contains(&(8080, 5678)));

        remove_whitelist(&conn, 3000).unwrap();
        let ports = get_whitelisted_ports(&conn).unwrap();
        assert!(!ports.contains(&(3000, 1234)));
        assert!(ports.contains(&(8080, 5678)));
    }
}
