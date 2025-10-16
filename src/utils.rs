use regex::Regex;
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
    } else if repository == "gnosisguild/zodiac" {
        return "gnosisguild-zodiac".to_string();
    } else if repository == "huff-language/huffmate" {
        return "huff-language-huffmate".to_string();
    } else if repository == "smartcontractkit/chainlink-brownie-contracts" {
        return "smartcontractkit-chainlink-brownie-contracts".to_string();
    } else if repository == "ProjectOpenSea/operator-filter-registry" {
        return "projectopensea-operator-filter-registry".to_string();
    } else if repository == "latticexyz/store" {
        return "latticexyz-store".to_string();
    } else if repository == "succinctlabs/sp1-contracts" {
        return "succinctlabs-sp1-contracts".to_string();
    } else if repository == "Uniswap/v4-core" {
        return "uniswap-v4-core".to_string();
    } else if repository == "Uniswap/v4-periphery" {
        return "uniswap-v4-periphery".to_string();
    } else if repository == "smartcontractkit/chainlink" {
        return "smartcontractkit-chainlink".to_string();
    } else if repository == "limitbreakinc/creator-token-standards" {
        return "limitbreakinc-creator-token-standards".to_string();
    } else if repository == "morpho-org/morpho-blue" {
        return "morpho-org-morpho-blue".to_string();
    } else if repository == "hashgraph/hedera-forking" {
        return "hashgraph-hedera-forking".to_string();
    } else if repository == "gnsps/solidity-bytes-utils" {
        return "gnsps-solidity-bytes-utils".to_string();
    } else if repository == "Uniswap/smart-order-router" {
        return "uniswap-smart-order-router".to_string();
    } else if repository == "zeframlou/create3-factory" {
        return "zeframlou-create3-factory".to_string();
    } else if repository == "morpho-org/metamorpho-v1.1" {
        return "morpho-org-metamorpho-v1.1".to_string();
    } else if repository == "morpho-org/public-allocator" {
        return "morpho-org-public-allocator".to_string();
    } else if repository == "openzeppelin/uniswap-hooks" {
        return "openzeppelin-uniswap-hooks".to_string();
    } else if repository == "0xsequence/sstore2" {
        return "0xsequence-sstore2".to_string();
    } else if repository == "huff-language/foundry-huff" {
        return "huff-,language-foundry-huff".to_string();
    } else if repository == "a16z/halmos-cheatcodes" {
        return "a16z-halmos-cheatcodes".to_string();
    } else if repository == "manifoldxyz/libraries-solidity" {
        return "manifoldxyz-libraries-solidity".to_string();
    } else if repository == "solv-finance/erc-3525" {
        return "solv-finance-erc-3525".to_string();
    } else if repository == "smartcontractkit/chainlink-evm" {
        return "smartcontractkit-chainlink-evm".to_string();
    } else if repository == "estarriolvetch/erc721psi" {
        return "estarriolvetch-erc721psi".to_string();
    } else if repository == "circlefin/evm-cctp-contracts" {
        return "circlefin-evm-cctp-contracts".to_string();
    } else if repository == "manifoldxyz/creator-core-solidity" {
        return "manifoldxyz-creator-core-solidity".to_string();
    } else if repository == "transmissions11/solmate" {
        return "solmate".to_string();
    } else if repository == "boringcrypto/BoringSolidity" {
        return "boringcrypto-boringsolidity".to_string();
    } else if repository == "euler-xyz/euler-interfaces" {
        return "euler-xyz-euler-interfaces".to_string();
    } else if repository == "pendle-finance/pendle-core-v2-public" {
        return "pendle-finance-pendle-core-v2-public".to_string();
    } else if repository == "Balmy-protocol/uniswap-v3-oracle" {
        return "balmy-protocol-uniswap-v3-oracle".to_string();
    } else if repository == "Recon-Fuzz/chimera" {
        return "recon-fuzz-chimera".to_string();
    } else if repository == "Recon-Fuzz/setup-helpers" {
        return "recon-fuzz-setup-helpers".to_string();
    } else if repository == "morpho-org/morpho-blue-oracles" {
        return "morpho-org-morpho-blue-oracles".to_string();
    } else if repository == "risc0/risc0-ethereum" {
        return "risc0-risc0-ethereum".to_string();
    }
    let dependency_split: Vec<&str> = repository.split("/").collect();
    dependency_split[1].to_string()
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
        || dependency_name == "gnosisguild-zodiac"
        || dependency_name == "huff-language-huffmate"
        || dependency_name == "smartcontractkit-chainlink-brownie-contracts"
        || dependency_name == "projectopensea-operator-filter-registry"
        || dependency_name == "latticexyz-store"
        || dependency_name == "succinctlabs-sp1-contracts"
        || dependency_name == "uniswap-v4-core"
        || dependency_name == "uniswap-v4-periphery"
        || dependency_name == "smartcontractkit-chainlink"
        || dependency_name == "limitbreakinc-creator-token-standards"
        || dependency_name == "morpho-org-morpho-blue"
        || dependency_name == "hashgraph-hedera-forking"
        || dependency_name == "gnsps-solidity-bytes-utils"
        || dependency_name == "uniswap-smart-order-router"
        || dependency_name == "zeframlou-create3-factory"
        || dependency_name == "morpho-org-metamorpho-v1.1"
        || dependency_name == "morpho-org-public-allocator"
        || dependency_name == "openzeppelin-uniswap-hooks"
        || dependency_name == "0xsequence-sstore2"
        || dependency_name == "huff-language-foundry-huff"
        || dependency_name == "a16z-halmos-cheatcodes"
        || dependency_name == "manifoldxyz-libraries-solidity"
        || dependency_name == "solv-finance-erc-3525"
        || dependency_name == "smartcontractkit-chainlink-evm"
        || dependency_name == "estarriolvetch-erc721psi"
        || dependency_name == "circlefin-evm-cctp-contracts"
        || dependency_name == "manifoldxyz-creator-core-solidity"
        || dependency_name == "solmate"
        || dependency_name == "boringcrypto-boringsolidity"
        || dependency_name == "euler-xyz-euler-interfaces"
        || dependency_name == "pendle-finance-pendle-core-v2-public"
        || dependency_name == "balmy-protocol-uniswap-v3-oracle"
        || dependency_name == "recon-fuzz-chimera"
        || dependency_name == "recon-fuzz-setup-helpers"
        || dependency_name == "morpho-org-morpho-blue-oracles"
        || dependency_name == "risc0-risc0-ethereum"
    {
                let version_pattern = r"^v(\d+\.)*\d+$";
        let re = Regex::new(version_pattern).unwrap();
        if re.is_match(&version_to_return) {
            version_to_return = version_to_return[1..].to_string();
        } else if version_to_return.contains(" ") {
            version_to_return = version_to_return.replace(" ", "-");
        }
    }

    if version_to_return.contains(" ") {
        let split: Vec<&str> = version_to_return.split(" ").collect();
        return split[split.len() - 1].to_string();
    }
    version_to_return.to_string()
}

#[derive(Debug, Clone)]
pub struct FileNotFound;

impl fmt::Display for FileNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "file not found")
    }
}
