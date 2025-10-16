use chrono::{DateTime, Duration, Utc};
use rusqlite::{Connection, Error, Result};
use serde_derive::{Deserialize, Serialize};
use std::fmt::{self};

#[derive(Deserialize, Serialize)]
pub struct Version {
    pub repository: String,
    pub version: String,
    pub last_updated: DateTime<Utc>,
}

pub fn get_versions_for_repo_from_db(repository: String) -> Result<Vec<String>, Error> {
    let conn = Connection::open("repositories.db")?;

    conn.execute(
        "create table if not exists versions (
             id integer primary key,
             repository text not null,
             version text not null,
             last_updated datetime not null
         )",
        (),
    )?;
    let mut stmt: rusqlite::Statement<'_> =
        conn.prepare("SELECT version from versions where repository = ?1")?;

    let versions = stmt.query_map([&repository], |row| Ok(row.get(0)?))?;

    Ok(versions
        .map(|version: std::result::Result<String, Error>| version.unwrap())
        .collect())
}

pub fn get_invalid_versions_for_repo_from_db(repository: String) -> Result<Vec<String>, Error> {
    let conn = Connection::open("repositories.db")?;

    conn.execute(
        "create table if not exists invalid_versions (
             id integer primary key,
             repository text not null,
             version text not null,
             last_updated datetime not null
         )",
        (),
    )?;
    let mut stmt: rusqlite::Statement<'_> =
        conn.prepare("SELECT version from invalid_versions where repository = ?1")?;

    let versions = stmt.query_map([&repository], |row| Ok(row.get(0)?))?;

    Ok(versions
        .map(|version: std::result::Result<String, Error>| version.unwrap())
        .collect())
}

pub fn insert_version_into_db(version: Version) -> Result<(), Error> {
    println!(
        "Inserting version {:?} into db for {:?}",
        version.version, version.repository
    );
    let conn = Connection::open("repositories.db")?;

    conn.execute(
        "create table if not exists versions (
             id integer primary key,
             repository text not null unique,
             version text not null unique,
             last_updated datetime not null
         )",
        (),
    )?;

    let mut stmt: rusqlite::Statement<'_> = conn
        .prepare("INSERT INTO versions (repository, version, last_updated) VALUES (?1, ?2, ?3)")?;

    stmt.execute([
        &version.repository,
        &version.version,
        &version.last_updated.to_string(),
    ])?;

    Ok(())
}

pub fn insert_invalid_version_into_db(version: Version) -> Result<(), Error> {
    println!(
        "Inserting invalid_versions {:?} into db for {:?}",
        version.version, version.repository
    );
    let conn = Connection::open("repositories.db")?;

    conn.execute(
        "create table if not exists invalid_versions (
             id integer primary key,
             repository text not null unique,
             version text not null unique,
             last_updated datetime not null
         )",
        (),
    )?;

    let mut stmt: rusqlite::Statement<'_> = conn.prepare(
        "INSERT INTO invalid_versions (repository, version, last_updated) VALUES (?1, ?2, ?3)",
    )?;

    stmt.execute([
        &version.repository,
        &version.version,
        &version.last_updated.to_string(),
    ])?;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct NotFound;

impl fmt::Display for NotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "file not found")
    }
}

pub fn get_repositories_not_updated_in_last_hour(
    all_repositories: Vec<String>,
) -> Result<Vec<String>, Error> {
    let conn = Connection::open("repositories.db")?;

    // Ensure the tables exist
    conn.execute(
        "create table if not exists versions (
             id integer primary key,
             repository text not null,
             version text not null,
             last_updated datetime not null
         )",
        (),
    )?;

    conn.execute(
        "create table if not exists invalid_versions (
             id integer primary key,
             repository text not null,
             version text not null,
             last_updated datetime not null
         )",
        (),
    )?;

    let one_hour_ago = Utc::now() - Duration::hours(1);
    let mut repositories_to_update = Vec::new();

    for repository in all_repositories {
        // Get the most recent update time for this repository from both tables
        let mut stmt = conn.prepare(
            "SELECT MAX(last_updated) FROM (
                SELECT last_updated FROM versions WHERE repository = ?1
                UNION ALL
                SELECT last_updated FROM invalid_versions WHERE repository = ?1
            )",
        )?;

        let mut rows = stmt.query_map([&repository], |row| {
            let last_updated_str: Option<String> = row.get(0)?;
            Ok(last_updated_str)
        })?;

        let should_update = if let Some(Ok(Some(last_updated_str))) = rows.next() {
            // Parse the datetime string and check if it's older than 1 hour
            match last_updated_str
                .strip_suffix(" UTC")
                .and_then(|s| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S.%6f").ok()
                })
                .or_else(|| {
                    last_updated_str.strip_suffix(" UTC").and_then(|s| {
                        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f").ok()
                    })
                })
                .or_else(|| {
                    last_updated_str.strip_suffix(" UTC").and_then(|s| {
                        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()
                    })
                })
                .map(|ndt| ndt.and_utc())
            {
                Some(last_updated) => last_updated < one_hour_ago,
                None => {
                    println!(
                        "Failed to parse datetime for repository {}: {}",
                        repository, last_updated_str
                    );
                    true // If we can't parse, assume it needs updating
                }
            }
        } else {
            // No records found for this repository, so it needs updating
            true
        };

        if should_update {
            repositories_to_update.push(repository);
        }
    }

    Ok(repositories_to_update)
}
