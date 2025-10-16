use crate::utils::{format_version, get_current_working_dir, read_file_to_string};
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
    println!("repository: {}", repository);
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
    if repository != "morpho-org/morpho-blue"
        && repository != "morpho-org/public-allocator"
        && repository != "gnsps/solidity-bytes-utils"
    {
        for val in page.into_iter().rev() {
            let mut unsplit_name = val.name.unwrap();
            if unsplit_name.is_empty() {
                unsplit_name = val.tag_name;
            }
            let mut name = unsplit_name.as_str();
            if unsplit_name.contains("v") {
                (_, name) = unsplit_name.split_once("v").unwrap();
            } else if unsplit_name.contains(" ") {
                let splitted: Vec<&str> = unsplit_name.split(" ").collect();
                name = splitted[splitted.len() - 1];
            }
            versions.push(VersionStruct {
                name: name.to_string(),
                url: val.zipball_url.unwrap().to_string(),
            });
        }
    }

    //tags
    if (versions.is_empty() && repository != "Uniswap/permit2")
        || repository == "morpho-org/morpho-blue"
        || repository == "gnsps/solidity-bytes-utils"
        || repository == "smartcontractkit/chainlink-evm"
        || repository == "manifoldxyz/creator-core-solidity"
        || repository == "Balmy-protocol/uniswap-v3-oracle"
        || repository == "Recon-Fuzz/chimera"
    {
        let page = octocrab
            .repos(split_versions[0], split_versions[1])
            .list_tags()
            // Optional Parameters
            .per_page(100)
            .page(0u32)
            // Send the request
            .send()
            .await
            .unwrap();

        for val in page.into_iter().rev() {
            let mut unsplit_name = val.name;
            if unsplit_name.is_empty() {
                unsplit_name = val.commit.sha;
            }
            let mut name = unsplit_name.as_str();
            if unsplit_name.contains("v") {
                (_, name) = unsplit_name.split_once("v").unwrap();
            } else if unsplit_name.contains(" ") {
                let splitted: Vec<&str> = unsplit_name.split(" ").collect();
                name = splitted[splitted.len() - 1];
            }
            versions.push(VersionStruct {
                name: name.to_string(),
                url: val.zipball_url.to_string(),
            });
        }
    }

    if repository == "morpho-org/metamorpho-v1.1"
        || repository == "zeframlou/create3-factory"
        || repository == "0xsequence/sstore2"
        || repository == "huff-language/foundry-huff"
        || repository == "a16z/halmos-cheatcodes"
        || repository == "Uniswap/v4-periphery"
        || repository == "transmissions11/solmate"
        || repository == "boringcrypto/BoringSolidity"
        || repository == "euler-xyz/euler-interfaces"
        || repository == "pendle-finance/pendle-core-v2-public"
        || repository == "Recon-Fuzz/setup-helpers"
        || repository == "morpho-org/morpho-blue-oracles"
    {
        let mut main_branch = None;
        let mut page_num = 1u32;

        // Iterate through all pages to find main or master branch
        loop {
            let page = octocrab
                .repos(split_versions[0], split_versions[1])
                .list_branches()
                .per_page(100)
                .page(page_num)
                .send()
                .await
                .unwrap();

            // Look for main or master branch in current page
            if let Some(branch) = page
                .items
                .iter()
                .find(|b| b.name == "main" || b.name == "master")
            {
                main_branch = Some(branch.clone());
                break;
            }

            // If no more pages, break
            if page.items.len() < 100 {
                break;
            }

            page_num += 1;
        }

        let main_branch = main_branch.unwrap_or_else(|| {
            eprintln!(
                "No main or master branch found for repository: {}",
                repository
            );
            std::process::exit(1);
        });

        let commit_sha = main_branch.commit.sha.clone();
        versions.push(VersionStruct {
            name: commit_sha.clone(),
            url: format!(
                "https://api.github.com/repos/{}/{}/zipball/{}",
                split_versions[0], split_versions[1], commit_sha
            ),
        });
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
        // Try to decode the response data as a string to check for error messages
        if let Ok(response_str) = String::from_utf8(dst.clone()) {
            if response_str.contains("\"message\"") && response_str.contains("\"status\"") {
                let mut new_version = version.clone();
                new_version.url = new_version
                    .url
                    .clone()
                    .replace("/zipball/", "/zipball/refs/tags/");
                return Box::pin(download_dependency(dependency_name, &new_version)).await;
            }
        }
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
    let target_dep_version = format_version(dependency_name, dependency_version);
    let target_name: String = format!("{}-{}/", dependency_name, target_dep_version);
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
pub struct UnzippingError {
    pub name: String,
    pub version: String,
}
