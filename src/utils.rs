use std::env;
use std::fmt;
use std::fs::{self};
use std::path::PathBuf;

// get the current working directory
pub fn get_current_working_dir() -> std::io::Result<PathBuf> {
    env::current_dir()
}

pub fn read_file_to_string(filename: String) -> Result<String, FileNotFound> {
    let contents: String = match fs::read_to_string(&filename) {
        // If successful return the files text as `contents`.
        // `c` is a local variable.
        Ok(c) => c,
        // Handle the `error` case.
        Err(_) => {
            eprintln!("Could not read file `{}`", &filename);
            return Err(FileNotFound);
        }
    };
    Ok(contents)
}

pub fn format_dependency_name(repository: &String) -> String {
    if repository == "eth-infinitism/account-abstraction" {
        return "eth-infinitism-account-abstraction".to_string();
    } else if repository == "base-org/webauthn-sol" {
        return "base-org-webauthn-sol".to_string();
    } else if repository == "safe-global/safe-smart-account" {
        return "safe-global-safe-smart-account".to_string();
    } else if repository == "colinnielsen/safe-tools" {
        return "colinnielsen-safe-tools".to_string();
    } else if repository == "worldcoin/world-id-contracts" {
        return "worldcoin-world-id-contracts".to_string();
    } else if repository == "Cyfrin/foundry-era-contracts" {
        return "cyfrin-foundry-era-contracts".to_string();
    } else if repository == "euler-xyz/ethereum-vault-connector" {
        return "euler-xyz-ethereum-vault-connector".to_string();
    } else if repository == "Cyfrin/foundry-devops" {
        return "cyfrin-foundry-devops".to_string();
    } else if repository == "alchemyplatform/modular-account" {
        return "alchemyplatform-modular-account".to_string();
    } else if repository == "erc6551/reference" {
        return "erc6551-reference".to_string();
    } else if repository == "Layr-Labs/eigenlayer-contracts" {
        return "layr-labs-eigenlayer-contracts".to_string();
    } else if repository == "smartcontractkit/ccip" {
        return "smartcontractkit-ccip".to_string();
    } else if repository == "perimetersec/fuzzlib" {
        return "perimetersec-fuzzlib".to_string();
    } else if repository == "crytic/properties" {
        return "crytic-properties".to_string();
    } else if repository == "ava-labs/avalanche-interchain-token-transfer" {
        return "ava-labs-avalanche-interchain-token-transfer".to_string();
    } else if repository == "Uniswap/permit2" {
        return "uniswap-permit2".to_string();
    }

    let dependency_split: Vec<&str> = repository.split("/").collect();
    return dependency_split[1].to_string();
}

pub fn format_version(dependency_name: &String, version: &String) -> String {
    let mut version_to_return = version.to_string();
    if dependency_name == "openzeppelin-foundry-upgrades"
        || dependency_name == "eth-infinitism-account-abstraction"
        || dependency_name == "colinnielsen-safe-tools"
        || dependency_name == "worldcoin-world-id-contracts"
        || dependency_name == "cyfrin-foundry-era-contracts"
        || dependency_name == "euler-xyz-ethereum-vault-connector"
        || dependency_name == "cyfrin-foundry-devops"
        || dependency_name == "alchemyplatform-modular-account"
        || dependency_name == "erc6551-reference"
        || dependency_name == "layr-labs-eigenlayer-contracts"
        || dependency_name == "smartcontractkit-ccip"
        || dependency_name == "perimetersec-fuzzlib"
        || dependency_name == "crytic-properties"
        || dependency_name == "ava-labs-avalanche-interchain-token-transfer"
        || dependency_name == "uniswap-permit2"
    {
        version_to_return = version_to_return.replace("v", "");
    }

    if version_to_return.contains(" ") {
        let split: Vec<&str> = version_to_return.split(" ").collect();
        return split[split.len() - 1].to_string();
    }
    return version_to_return.to_string();
}

#[derive(Debug, Clone)]
pub struct FileNotFound;

impl fmt::Display for FileNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "file not found")
    }
}
