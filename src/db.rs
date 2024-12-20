use chrono::{DateTime, Utc};
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
