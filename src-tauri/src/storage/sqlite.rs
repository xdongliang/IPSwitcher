use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::AppError;
use crate::models::profile::{IpMode, Profile};

pub struct ProfileRepository {
    conn: Mutex<Connection>,
}

impl ProfileRepository {
    pub fn new() -> Result<Self, AppError> {
        let db_path = Self::db_path()?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        let repo = Self {
            conn: Mutex::new(conn),
        };
        repo.run_migrations()?;
        Ok(repo)
    }

    fn db_path() -> Result<PathBuf, AppError> {
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME")
                .map_err(|_| AppError::Database(rusqlite::Error::InvalidPath(
                    "HOME environment variable not set".into()
                )))?;
            Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("com.ipswitcher.app")
                .join("profiles.db"))
        }

        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA")
                .map_err(|_| AppError::Database(rusqlite::Error::InvalidPath(
                    "APPDATA environment variable not set".into()
                )))?;
            Ok(PathBuf::from(appdata).join("IPSwitcher").join("profiles.db"))
        }
    }

    fn run_migrations(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::Database(rusqlite::Error::InvalidParameterName(
                format!("Mutex poison: {}", e)
            ))
        })?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS profiles (
                id              TEXT PRIMARY KEY,
                name            TEXT NOT NULL UNIQUE,
                ip_mode         TEXT NOT NULL CHECK (ip_mode IN ('Manual', 'Dhcp')),
                ip_address      TEXT,
                subnet_mask     TEXT,
                gateway         TEXT,
                dns_servers     TEXT NOT NULL DEFAULT '[]',
                interface_name  TEXT,
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );"
        )?;
        Ok(())
    }

    pub fn list_all(&self) -> Result<Vec<Profile>, AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::Database(rusqlite::Error::InvalidParameterName(format!("{}", e)))
        })?;
        let mut stmt = conn.prepare(
            "SELECT id, name, ip_mode, ip_address, subnet_mask, gateway,
                    dns_servers, interface_name, created_at, updated_at
             FROM profiles ORDER BY updated_at DESC"
        )?;

        let profiles = stmt.query_map([], |row| {
            let dns_json: String = row.get(6)?;
            let dns_servers: Vec<String> = serde_json::from_str(&dns_json).unwrap_or_default();

            Ok(Profile {
                id: row.get(0)?,
                name: row.get(1)?,
                ip_mode: match row.get::<_, String>(2)?.as_str() {
                    "Manual" => IpMode::Manual,
                    _ => IpMode::Dhcp,
                },
                ip_address: row.get(3)?,
                subnet_mask: row.get(4)?,
                gateway: row.get(5)?,
                dns_servers,
                interface_name: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(profiles)
    }

    pub fn get_by_id(&self, id: &str) -> Result<Profile, AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::Database(rusqlite::Error::InvalidParameterName(format!("{}", e)))
        })?;
        let mut stmt = conn.prepare(
            "SELECT id, name, ip_mode, ip_address, subnet_mask, gateway,
                    dns_servers, interface_name, created_at, updated_at
             FROM profiles WHERE id = ?1"
        )?;

        stmt.query_row(params![id], |row| {
            let dns_json: String = row.get(6)?;
            let dns_servers: Vec<String> = serde_json::from_str(&dns_json).unwrap_or_default();

            Ok(Profile {
                id: row.get(0)?,
                name: row.get(1)?,
                ip_mode: match row.get::<_, String>(2)?.as_str() {
                    "Manual" => IpMode::Manual,
                    _ => IpMode::Dhcp,
                },
                ip_address: row.get(3)?,
                subnet_mask: row.get(4)?,
                gateway: row.get(5)?,
                dns_servers,
                interface_name: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(id.to_string()),
            other => AppError::Database(other),
        })
    }

    pub fn insert(&self, profile: &Profile) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::Database(rusqlite::Error::InvalidParameterName(format!("{}", e)))
        })?;
        let dns_json = serde_json::to_string(&profile.dns_servers)?;

        let result = conn.execute(
            "INSERT INTO profiles (id, name, ip_mode, ip_address, subnet_mask, gateway,
             dns_servers, interface_name, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                profile.id,
                profile.name,
                match profile.ip_mode {
                    IpMode::Manual => "Manual",
                    IpMode::Dhcp => "Dhcp",
                },
                profile.ip_address,
                profile.subnet_mask,
                profile.gateway,
                dns_json,
                profile.interface_name,
                profile.created_at,
                profile.updated_at,
            ],
        );

        match result {
            Ok(_) => Ok(()),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(AppError::DuplicateName(profile.name.clone()))
            }
            Err(e) => Err(AppError::Database(e)),
        }
    }

    pub fn update(&self, profile: &Profile) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::Database(rusqlite::Error::InvalidParameterName(format!("{}", e)))
        })?;
        let dns_json = serde_json::to_string(&profile.dns_servers)?;

        let result = conn.execute(
            "UPDATE profiles SET name = ?1, ip_mode = ?2, ip_address = ?3,
             subnet_mask = ?4, gateway = ?5, dns_servers = ?6,
             interface_name = ?7, updated_at = ?8
             WHERE id = ?9",
            params![
                profile.name,
                match profile.ip_mode {
                    IpMode::Manual => "Manual",
                    IpMode::Dhcp => "Dhcp",
                },
                profile.ip_address,
                profile.subnet_mask,
                profile.gateway,
                dns_json,
                profile.interface_name,
                profile.updated_at,
                profile.id,
            ],
        );

        match result {
            Ok(0) => Err(AppError::NotFound(profile.id.clone())),
            Ok(_) => Ok(()),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(AppError::DuplicateName(profile.name.clone()))
            }
            Err(e) => Err(AppError::Database(e)),
        }
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::Database(rusqlite::Error::InvalidParameterName(format!("{}", e)))
        })?;
        let rows = conn.execute("DELETE FROM profiles WHERE id = ?1", params![id])?;
        if rows == 0 {
            return Err(AppError::NotFound(id.to_string()));
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn name_exists(&self, name: &str, exclude_id: Option<&str>) -> Result<bool, AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::Database(rusqlite::Error::InvalidParameterName(format!("{}", e)))
        })?;
        if let Some(exclude) = exclude_id {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM profiles WHERE name = ?1 AND id != ?2",
                params![name, exclude],
                |row| row.get(0),
            )?;
            Ok(count > 0)
        } else {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM profiles WHERE name = ?1",
                params![name],
                |row| row.get(0),
            )?;
            Ok(count > 0)
        }
    }

    pub fn get_active_profile_id(&self) -> Result<Option<String>, AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::Database(rusqlite::Error::InvalidParameterName(format!("{}", e)))
        })?;
        let result = conn.query_row(
            "SELECT value FROM settings WHERE key = 'active_profile_id'",
            [],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(id) => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }

    pub fn set_active_profile_id(&self, id: Option<&str>) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::Database(rusqlite::Error::InvalidParameterName(format!("{}", e)))
        })?;
        match id {
            Some(profile_id) => {
                conn.execute(
                    "INSERT INTO settings (key, value) VALUES ('active_profile_id', ?1)
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    params![profile_id],
                )?;
            }
            None => {
                conn.execute(
                    "DELETE FROM settings WHERE key = 'active_profile_id'",
                    [],
                )?;
            }
        }
        Ok(())
    }
}
