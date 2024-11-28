mod db;
mod github;
mod manager;
mod npm;
mod utils;

use chrono::Utc;
use db::{
    get_invalid_versions_for_repo_from_db, get_versions_for_repo_from_db, insert_version_into_db,
    Version,
};
use github::{download_dependency, github_retrieve_versions, unzip_dependency};
use manager::{github_push_to_repository_remote, npm_push_to_repository_remote};
use npm::LoadError;
use npm::{npm_retrieve_versions, retrieve_version};
use rusqlite::Error;
use std::env;
use std::fs::{remove_dir_all, remove_file};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use utils::{format_dependency_name, format_version, get_current_working_dir};

#[tokio::main]
async fn main() {
    let target = env::args().nth(1);
    if target.is_none() {
        println!("Argument failed, should be npm or github");
        exit(1);
    }
    let repositories: Vec<String>;
    let source = target.unwrap();
    if source == "npm" {
        repositories = npm::load_repositories()
            .map_err(|err: LoadError| {
                println!("{:?}", err);
            })
            .unwrap();
    } else {
        repositories = match github::load_repositories() {
            Ok(repo) => repo,
            Err(err) => {
                eprintln!("Err {:?}", err);
                exit(1)
            }
        }
    }

    for repository in repositories {
        sleep(Duration::from_millis(1000));
        let existing_versions: Vec<String> = get_versions_for_repo_from_db(repository.clone())
            .map_err(|err: Error| {
                println!("{:?}", err);
            })
            .unwrap();
        let invalid_versions: Vec<String> =
            get_invalid_versions_for_repo_from_db(repository.clone())
                .map_err(|err: Error| {
                    println!("{:?}", err);
                })
                .unwrap();
        let versions: Vec<VersionStruct>;
        if source == "npm" {
            versions = npm_retrieve_versions(&repository)
                .map_err(|err: LoadError| {
                    println!("{:?}", err);
                })
                .unwrap();
        } else {
            versions = github_retrieve_versions(&repository).await.unwrap();
        }

        for version in versions.into_iter() {
            if existing_versions.contains(&version.name) || invalid_versions.contains(&version.name)
            {
                continue;
            }
            if source == "npm" {
                match retrieve_version(&repository, &version) {
                    Ok(_) => {}
                    Err(_) => {
                        continue;
                    }
                }
                match npm_push_to_repository_remote(&repository, &version.name).await {
                    Ok(_) => {}
                    Err(_) => {
                        continue;
                    }
                }
            } else {
                let dependency_name = &format_dependency_name(&repository);
                match download_dependency(dependency_name, &version).await {
                    Ok(_) => {}
                    Err(err) => {
                        eprint!("Error on downloading dependency {} {:?}", &repository, err);
                        exit(1);
                    }
                }

                match unzip_dependency(&dependency_name.to_string(), &version.name) {
                    Ok(_) => {}
                    Err(_) => {
                        eprintln!("Error unzipping {}", dependency_name);
                        exit(1);
                    }
                }
                let formatted_version = format_version(dependency_name, &version.name);
                match github_push_to_repository_remote(
                    &dependency_name.to_string(),
                    &formatted_version,
                )
                .await
                {
                    Ok(_) => {}
                    Err(err) => {
                        if err.cause.contains("Dependency already exists") {
                            let version_to_insert: Version = Version {
                                repository: repository.clone(),
                                version: version.name.clone(),
                                last_updated: Utc::now(),
                            };

                            insert_version_into_db(version_to_insert)
                                .map_err(|err: Error| {
                                    println!("{:?}", err);
                                })
                                .unwrap();
                        }
                        continue;
                    }
                }
            }
            let version_to_insert: Version = Version {
                repository: repository.clone(),
                version: version.name.clone(),
                last_updated: Utc::now(),
            };

            insert_version_into_db(version_to_insert)
                .map_err(|err: Error| {
                    println!("{:?}", err);
                })
                .unwrap();
            if source == "npm" {
                if get_current_working_dir()
                    .unwrap()
                    .join("node_modules")
                    .exists()
                {
                    remove_dir_all(get_current_working_dir().unwrap().join("node_modules"))
                        .unwrap();
                    remove_file(get_current_working_dir().unwrap().join("package.json")).unwrap();
                    remove_file(get_current_working_dir().unwrap().join("package-lock.json"))
                        .unwrap();
                }
            }
        }
    }
}

#[derive(Debug, Clone)]

pub struct VersionStruct {
    pub name: String,
    pub url: String,
}
