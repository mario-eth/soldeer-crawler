use crate::utils::{get_current_working_dir, read_file_to_string};
use crate::VersionStruct;
use curl::easy::{Easy, List};
use serde_derive::Deserialize;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{BufReader, Cursor, Read};
use std::path::{Path, PathBuf};

pub fn load_repositories() -> Result<Vec<String>, LoadError> {
    println!("Loading list of repositories for Github");
    let filename: String = get_current_working_dir()
        .unwrap()
        .join(String::from("repositories.toml"))
        .to_str()
        .unwrap()
        .to_string();
    let contents = read_file_to_string(filename.clone()).unwrap();
    let data: Data = match toml::from_str(&contents) {
        // If successful, return data as `Data` struct.
        // `d` is a local variable.
        Ok(d) => d,
        // Handle the `error` case.
        Err(err) => {
            eprintln!("Error: {}", err);
            // Write `msg` to `stderr`.
            eprintln!("Unable to load data from repositories.toml");
            // Exit the program with exit code `1`.
            return Err(LoadError);
        }
    };

    let mut repositories: Vec<String> = Vec::new();
    data.github.iter().for_each(|value: &String| {
        repositories.push(value.clone());
    });

    Ok(repositories)
}

pub async fn github_retrieve_versions(repository: &str) -> Result<Vec<VersionStruct>, LoadError> {
    let octocrab = octocrab::instance();
    let split_versions: Vec<&str> = repository.split("/").collect();
    let page = octocrab
        .repos(split_versions[0], split_versions[1])
        .releases()
        .list()
        // Optional Parameters
        .per_page(100)
        .page(0u32)
        // Send the request
        .send()
        .await
        .unwrap();

    let mut versions: Vec<VersionStruct> = Vec::new();
    for val in page.into_iter() {
        versions.push(VersionStruct {
            name: val.name.unwrap(),
            url: val.zipball_url.unwrap().to_string(),
        })
    }
    Ok(versions)
}

pub async fn download_dependency(
    dependency_name: &str,
    version: &VersionStruct,
) -> Result<(), DownloadError> {
    let dependency_directory: PathBuf = get_current_working_dir().unwrap().join("github");
    if !dependency_directory.is_dir() {
        fs::create_dir(&dependency_directory).unwrap();
    }

    let mut dst = Vec::new();
    let mut easy = Easy::new();
    easy.url(&version.url).unwrap();
    let mut list = List::new();
    list.append("User-Agent: Mozilla/5.0 (platform; rv:geckoversion) Gecko/geckotrail Firefox/firefoxversion").unwrap();
    easy.http_headers(list).unwrap();
    let _redirect = easy.follow_location(true);

    {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                dst.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }
    {
        let zip_path = format!("{}-{}.zip", &dependency_name, &version.name);

        let mut file = File::create(dependency_directory.join(zip_path)).unwrap();
        let _ = file.write_all(dst.as_slice());
    }
    Ok(())
}

pub fn unzip_dependency(
    dependency_name: &String,
    dependency_version: &String,
) -> Result<(), UnzippingError> {
    let file_name: String = format!("{}-{}.zip", dependency_name, dependency_version);
    let target_name: String = format!("{}-{}/", dependency_name, dependency_version);
    let current_dir: PathBuf = get_current_working_dir()
        .unwrap()
        .join(Path::new(&("github/".to_owned() + &file_name)));

    let target = get_current_working_dir()
        .unwrap()
        .join("github/")
        .join(target_name);
    let archive: Vec<u8> = read_file(current_dir.as_path().to_str().unwrap().to_string()).unwrap();
    match zip_extract::extract(Cursor::new(archive), &target, true) {
        Ok(_) => {}
        Err(_) => {
            return Err(UnzippingError {
                name: dependency_name.to_string(),
                version: dependency_version.to_string(),
            })
        }
    }
    println!(
        "The dependency {}-{} was unzipped!",
        dependency_name, dependency_version
    );
    Ok(())
}

pub fn read_file(path: String) -> Result<Vec<u8>, std::io::Error> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    // Read file into vector.
    reader.read_to_end(&mut buffer)?;

    Ok(buffer)
}

#[derive(Deserialize, Debug)]
struct Data {
    github: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LoadError;

#[derive(Debug, Clone)]
pub struct DownloadError;

#[derive(Debug, Clone)]
#[warn(unused_imports)]
pub struct UnzippingError {
    pub name: String,
    pub version: String,
}
