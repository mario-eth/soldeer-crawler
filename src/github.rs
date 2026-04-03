use crate::utils::{format_version, get_current_working_dir, read_file_to_string};
use crate::VersionStruct;
use curl::easy::{Easy, List};
use serde_derive::Deserialize;
use std::fmt;
use std::fs::{self, File};
use std::io::{Cursor, Write};
use std::path::PathBuf;

fn github_api_message(err: &octocrab::Error) -> Option<&str> {
    if let octocrab::Error::GitHub { source, .. } = err {
        Some(source.message.as_str())
    } else {
        None
    }
}

fn is_bad_credentials(err: &octocrab::Error) -> bool {
    let Some(msg) = github_api_message(err) else {
        return false;
    };
    let m = msg.to_ascii_lowercase();
    m.contains("bad credentials") || m.contains("invalid credentials")
}

fn verbose_github_api_errors() -> bool {
    std::env::var("CRAWLER_VERBOSE_GITHUB_ERRORS").ok().is_some_and(|v| {
        matches!(v.as_str(), "1" | "true" | "yes")
    })
}

/// Compact logs by default (GitHub JSON body only). Set `CRAWLER_VERBOSE_GITHUB_ERRORS=1` for
/// full octocrab `Debug` (includes large snafu backtraces when `RUST_BACKTRACE` is enabled).
fn log_octocrab_api_failure(operation: &str, repository: &str, err: &octocrab::Error) {
    eprintln!("GitHub API failure while {} for {}", operation, repository);
    eprintln!("  error (Display): {}", err);
    if let octocrab::Error::GitHub { source, .. } = err {
        eprintln!("  GitHub API response (Debug): {:?}", source);
    }
    if verbose_github_api_errors() {
        eprintln!("  error (Debug, verbose): {:?}", err);
        let mut next = std::error::Error::source(err);
        let mut depth = 0usize;
        while let Some(e) = next {
            eprintln!("  error::source[{}]: {}", depth, e);
            next = e.source();
            depth += 1;
        }
    }
    eprint_github_api_recovery_hint(err);
}

fn eprint_github_api_recovery_hint(err: &octocrab::Error) {
    let token_set = std::env::var("GITHUB_TOKEN")
        .map(|t| !t.trim().is_empty())
        .unwrap_or(false);

    let msg_lower = github_api_message(err).map(|s| s.to_ascii_lowercase());

    if is_bad_credentials(err) {
        if token_set {
            eprintln!(
                "GitHub rejected the configured token (Bad credentials). Replace GITHUB_TOKEN with a valid PAT (repo/metadata read) or fix the Actions secret; revoked and mistyped tokens cause this."
            );
        } else {
            eprintln!("GitHub returned Bad credentials unexpectedly (no GITHUB_TOKEN set). Try again or report if this persists.");
        }
        return;
    }

    if msg_lower
        .as_deref()
        .is_some_and(|m| m.contains("rate limit") || m.contains("abuse detection"))
    {
        eprintln!(
            "GitHub API rate or abuse limit. Wait and retry, or set GITHUB_TOKEN for higher authenticated limits."
        );
        return;
    }

    if !token_set {
        eprintln!(
            "If this persists, set GITHUB_TOKEN (see GitHub docs for fine-grained or classic PAT scopes)."
        );
    } else {
        eprintln!("If this persists, check token scopes and that the repository is accessible to this token.");
    }
}

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

#[derive(Debug, Clone)]
pub enum GithubApiError {
    /// GitHub returned an authentication error for the current `GITHUB_TOKEN`.
    BadCredentials,
    /// No `main` or `master` branch (misconfiguration or empty repo).
    MissingDefaultBranch,
    /// Transient or other API failure (e.g. rate limit, network).
    Other,
}

fn classify_octocrab(err: &octocrab::Error) -> GithubApiError {
    if is_bad_credentials(err) {
        GithubApiError::BadCredentials
    } else {
        GithubApiError::Other
    }
}

pub async fn github_retrieve_versions(
    repository: &str,
) -> Result<Vec<VersionStruct>, GithubApiError> {
    println!("repository: {}", repository);

    // Try to get GitHub token from environment variable and build octocrab instance
    let octocrab_builder = match std::env::var("GITHUB_TOKEN") {
        Ok(token) => match octocrab::OctocrabBuilder::new().personal_token(token).build() {
            Ok(instance) => Some(instance),
            Err(e) => {
                eprintln!("Failed to build GitHub API client from GITHUB_TOKEN: {}", e);
                return Err(GithubApiError::Other);
            }
        },
        Err(_) => {
            eprintln!("Warning: GITHUB_TOKEN not set. Using unauthenticated API access (lower rate limits)");
            eprintln!("Set GITHUB_TOKEN environment variable for higher rate limits");
            None
        }
    };

    let octocrab = if let Some(instance) = octocrab_builder {
        std::sync::Arc::new(instance)
    } else {
        octocrab::instance()
    };

    let split_versions: Vec<&str> = repository.split('/').collect();
    if split_versions.len() < 2 {
        eprintln!(
            "Invalid repository slug {:?} (expected owner/name)",
            repository
        );
        return Err(GithubApiError::Other);
    }

    let page = match octocrab
        .repos(split_versions[0], split_versions[1])
        .releases()
        .list()
        // Optional Parameters
        .per_page(100)
        .page(0u32)
        // Send the request
        .send()
        .await
    {
        Ok(page) => page,
        Err(err) => {
            log_octocrab_api_failure("listing releases", repository, &err);
            return Err(classify_octocrab(&err));
        }
    };
    let mut versions: Vec<VersionStruct> = Vec::new();
    if repository != "morpho-org/morpho-blue"
        && repository != "morpho-org/public-allocator"
        && repository != "gnsps/solidity-bytes-utils"
    {
        for val in page.into_iter().rev() {
            let Some(zip_url) = val.zipball_url else {
                continue;
            };
            let mut unsplit_name = val.name.unwrap_or_default();
            if unsplit_name.is_empty() {
                unsplit_name = val.tag_name;
            }
            let mut name = unsplit_name.as_str();
            if unsplit_name.contains('v') {
                if let Some((_, after_v)) = unsplit_name.split_once('v') {
                    name = after_v;
                }
            } else if unsplit_name.contains(' ') {
                let splitted: Vec<&str> = unsplit_name.split(' ').collect();
                name = splitted.last().copied().unwrap_or(name);
            }
            versions.push(VersionStruct {
                name: name.to_string(),
                url: zip_url.to_string(),
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
        let page = match octocrab
            .repos(split_versions[0], split_versions[1])
            .list_tags()
            // Optional Parameters
            .per_page(100)
            .page(0u32)
            // Send the request
            .send()
            .await
        {
            Ok(page) => page,
            Err(err) => {
                log_octocrab_api_failure("listing tags", repository, &err);
                return Err(classify_octocrab(&err));
            }
        };

        for val in page.into_iter().rev() {
            let mut unsplit_name = val.name;
            if unsplit_name.is_empty() {
                unsplit_name = val.commit.sha;
            }
            let mut name = unsplit_name.as_str();
            if unsplit_name.contains('v') {
                if let Some((_, after_v)) = unsplit_name.split_once('v') {
                    name = after_v;
                }
            } else if unsplit_name.contains(' ') {
                let splitted: Vec<&str> = unsplit_name.split(' ').collect();
                name = splitted.last().copied().unwrap_or(name);
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
        || repository == "SorellaLabs/angstrom"
    {
        let mut main_branch = None;
        let mut page_num = 1u32;

        // Iterate through all pages to find main or master branch
        loop {
            let page = match octocrab
                .repos(split_versions[0], split_versions[1])
                .list_branches()
                .per_page(100)
                .page(page_num)
                .send()
                .await
            {
                Ok(p) => p,
                Err(err) => {
                    log_octocrab_api_failure("listing branches", repository, &err);
                    return Err(classify_octocrab(&err));
                }
            };

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

        let Some(main_branch) = main_branch else {
            eprintln!(
                "No main or master branch found for repository: {}",
                repository
            );
            return Err(GithubApiError::MissingDefaultBranch);
        };

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
    let dependency_directory: PathBuf = get_current_working_dir()
        .map_err(|_| DownloadError)?
        .join("github");
    if !dependency_directory.is_dir() {
        fs::create_dir_all(&dependency_directory).map_err(|_| DownloadError)?;
    }

    let mut dst = Vec::new();
    let mut easy = Easy::new();
    easy.url(&version.url).map_err(|_| DownloadError)?;
    let mut list = List::new();
    list.append("User-Agent: Mozilla/5.0 (platform; rv:geckoversion) Gecko/geckotrail Firefox/firefoxversion")
        .map_err(|_| DownloadError)?;
    easy.http_headers(list).map_err(|_| DownloadError)?;
    let _redirect = easy.follow_location(true);

    {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                dst.extend_from_slice(data);
                Ok(data.len())
            })
            .map_err(|_| DownloadError)?;
        transfer.perform().map_err(|_| DownloadError)?;
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
        let mut file = File::create(dependency_directory.join(zip_path)).map_err(|_| DownloadError)?;
        file.write_all(dst.as_slice()).map_err(|_| DownloadError)?;
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
    let base = get_current_working_dir().map_err(|_| UnzippingError {
        name: dependency_name.clone(),
        version: dependency_version.clone(),
    })?;
    let zip_file = base.join("github").join(&file_name);
    let target = base.join("github").join(target_name);
    let archive: Vec<u8> = fs::read(&zip_file).map_err(|_| UnzippingError {
        name: dependency_name.clone(),
        version: dependency_version.clone(),
    })?;
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

impl fmt::Display for UnzippingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to unzip {} {}", self.name, self.version)
    }
}

impl std::error::Error for UnzippingError {}
