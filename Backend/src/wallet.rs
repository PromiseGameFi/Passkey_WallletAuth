use bip39::{Mnemonic, Language};
use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey, DerivationPath};
use secp256k1::{Secp256k1, SecretKey};
use web3::{
    contract::{Contract, Options},
    types::{Address, U256, TransactionParameters, TransactionRequest, H256},
    Transport,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TokenBalance {
    pub symbol: String,
    pub balance: U256,
    pub decimal: u8,
    pub contract_address: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WalletAccount {
    pub address: String,
    pub path: String,
    pub index: u32,
    pub balance: U256,
    pub tokens: HashMap<String, TokenBalance>,
}

pub struct HDWallet<T: Transport> {
    master_key: ExtendedPrivKey,
    pub accounts: Vec<WalletAccount>,
    web3: web3::Web3<T>,
    token_config: Value,
}

impl<T: Transport> HDWallet<T> {
    pub async fn new(passkey: &[u8], web3: web3::Web3<T>) -> Result<Self> {
        // Generate entropy from passkey
        let entropy = sha2::Sha256::digest(passkey);
        
        // Generate mnemonic from entropy
        let mnemonic = Mnemonic::from_entropy(&entropy)?;
        
        // Generate master key
        let seed = mnemonic.to_seed("");
        let secp = Secp256k1::new();
        let master_key = ExtendedPrivKey::new_master(bitcoin::Network::Bitcoin, &seed)?;

        // Load token configuration
        let token_config: Value = serde_json::from_str(include_str!("erc20_abi.json"))?;

        Ok(Self {
            master_key,
            accounts: Vec::new(),
            web3,
            token_config,
        })
    }

    // Derive new account
    pub async fn derive_account(&mut self) -> Result<WalletAccount> {
        let index = self.accounts.len() as u32;
        let path = format!("m/44'/60'/0'/0/{}", index);
        
        let child_key = self.master_key.derive_priv(
            &Secp256k1::new(),
            &DerivationPath::from_str(&path)?
        )?;
        
        let secret_key = SecretKey::from_slice(&child_key.private_key.secret_bytes())?;
        let public_key = secp256k1::PublicKey::from_secret_key(&Secp256k1::new(), &secret_key);
        
        let address = public_key_to_eth_address(&public_key);
        let address_str = format!("0x{:x}", address);

        let mut account = WalletAccount {
            address: address_str.clone(),
            path,
            index,
            balance: U256::zero(),
            tokens: HashMap::new(),
        };

        // Initialize with ETH balance and token balances
        account.balance = self.web3.eth().balance(address, None).await?;
        account.tokens = self.load_token_balances(address).await?;

        self.accounts.push(account.clone());
        Ok(account)
    }

    // Load balances for all configured tokens
    pub async fn load_token_balances(&self, address: Address) -> Result<HashMap<String, TokenBalance>> {
        let mut balances = HashMap::new();
        
        let tokens = self.token_config["tokens"].as_object()
            .ok_or_else(|| anyhow!("Invalid token configuration"))?;

        for (symbol, token_info) in tokens {
            let contract_address = Address::from_str(
                token_info["address"].as_str()
                    .ok_or_else(|| anyhow!("Invalid token address"))?
            )?;

            let contract = Contract::from_json(
                self.web3.eth(),
                contract_address,
                include_bytes!("erc20_abi.json")
            )?;

            let balance: U256 = contract.query(
                "balanceOf",
                (address,),
                None,
                Options::default(),
                None,
            ).await?;

            balances.insert(symbol.clone(), TokenBalance {
                symbol: symbol.clone(),
                balance,
                decimal: token_info["decimals"].as_u64()
                    .ok_or_else(|| anyhow!("Invalid decimals"))? as u8,
                contract_address: token_info["address"].as_str()
                    .ok_or_else(|| anyhow!("Invalid contract address"))?.to_string(),
            });
        }

        Ok(balances)
    }

    // Update all balances for an account
    pub async fn update_account_balances(&mut self, index: usize) -> Result<()> {
        let account = self.accounts.get_mut(index)
            .ok_or_else(|| anyhow!("Invalid account index"))?;

        let address = Address::from_str(&account.address)?;
        
        // Update ETH balance
        account.balance = self.web3.eth().balance(address, None).await?;
        
        // Update token balances
        account.tokens = self.load_token_balances(address).await?;

        Ok(())
    }

    // Send ETH
    pub async fn send_eth(
        &self,
        from_index: usize,
        to: &str,
        amount: U256,
        gas_price: Option<U256>,
    ) -> Result<H256> {
        let from_account = self.accounts.get(from_index)
            .ok_or_else(|| anyhow!("Invalid account index"))?;

        let from_address = Address::from_str(&from_account.address)?;
        let to_address = Address::from_str(to)?;

        let tx = TransactionRequest {
            from: from_address,
            to: Some(to_address),
            value: amount,
            gas_price,
            ..Default::default()
        };

        let tx_hash = self.web3.eth().send_transaction(tx).await?;
        Ok(tx_hash)
    }

    // Send ERC20 token
    pub async fn send_token(
        &self,
        from_index: usize,
        token_symbol: &str,
        to: &str,
        amount: U256,
        gas_price: Option<U256>,
    ) -> Result<H256> {
        let from_account = self.accounts.get(from_index)
            .ok_or_else(|| anyhow!("Invalid account index"))?;

        let token_info = self.token_config["tokens"][token_symbol].as_object()
            .ok_or_else(|| anyhow!("Token not found"))?;

        let contract_address = Address::from_str(
            token_info["address"].as_str()
                .ok_or_else(|| anyhow!("Invalid token address"))?
        )?;

        let contract = Contract::from_json(
            self.web3.eth(),
            contract_address,
            include_bytes!("erc20_abi.json")
        )?;

        let tx_hash = contract.call(
            "transfer",
            (
                Address::from_str(to)?,
                amount
            ),
            Address::from_str(&from_account.address)?,
            Options {
                gas_price,
                ..Default::default()
            }
        ).await?;

        Ok(tx_hash)
    }

    // Get token info
    pub fn get_token_info(&self, symbol: &str) -> Result<(&str, u8)> {
        let token_info = self.token_config["tokens"][symbol].as_object()
            .ok_or_else(|| anyhow!("Token not found"))?;

        Ok((
            token_info["address"].as_str()
                .ok_or_else(|| anyhow!("Invalid token address"))?,
            token_info["decimals"].as_u64()
                .ok_or_else(|| anyhow!("Invalid decimals"))? as u8
        ))
    }

    // List all supported tokens
    pub fn list_supported_tokens(&self) -> Vec<String> {
        self.token_config["tokens"].as_object()
            .map(|tokens| tokens.keys().cloned().collect())
            .unwrap_or_default()
    }

    // Get account by index
    pub fn get_account(&self, index: usize) -> Option<&WalletAccount> {
        self.accounts.get(index)
    }

    // Get total number of accounts
    pub fn get_account_count(&self) -> usize {
        self.accounts.len()
    }
}

// Helper function to convert public key to Ethereum address
fn public_key_to_eth_address(public_key: &secp256k1::PublicKey) -> Address {
    let public_key = public_key.serialize_uncompressed();
    let hash = ethereum::util::keccak256(&public_key[1..]);
    Address::from_slice(&hash[12..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use web3::Web3;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_wallet_creation() {
        let transport = web3::transports::Http::new("http://localhost:8545").unwrap();
        let web3 = Web3::new(transport);
        
        let wallet = HDWallet::new(&[1; 32], web3).await.unwrap();
        assert_eq!(wallet.get_account_count(), 0);
    }

    #[tokio::test]
    async fn test_account_derivation() {
        let transport = web3::transports::Http::new("http://localhost:8545").unwrap();
        let web3 = Web3::new(transport);
        
        let mut wallet = HDWallet::new(&[1; 32], web3).await.unwrap();
        let account = wallet.derive_account().await.unwrap();
        
        assert_eq!(account.index, 0);
        assert!(account.address.starts_with("0x"));
        assert_eq!(wallet.get_account_count(), 1);
    }
}