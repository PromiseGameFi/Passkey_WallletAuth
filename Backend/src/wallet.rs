use bip39::{Mnemonic, Language};
use bitcoin::util::bip32::{ExtendedPrivKey, DerivationPath};
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
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TokenBalance {
    pub symbol: String,
    pub balance: U256,
    pub decimal: u8,
    pub contract_address: String,
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WalletAccount {
    pub address: String,
    pub path: String,
    pub index: u32,
    pub balance: U256,
    pub tokens: HashMap<String, TokenBalance>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TransactionHistory {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: U256,
    pub token_symbol: Option<String>,
    pub timestamp: u64,
    pub status: bool,
}

pub struct HDWallet<T: Transport> {
    master_key: ExtendedPrivKey,
    pub accounts: Vec<WalletAccount>,
    web3: web3::Web3<T>,
    token_config: Value,
    pub transaction_history: Vec<TransactionHistory>,
}

impl<T: Transport> HDWallet<T> {
    pub async fn new(credential_id: &[u8], web3: web3::Web3<T>) -> Result<Self> {
        // Generate deterministic seed from WebAuthn credential
        let mut hasher = Sha256::new();
        hasher.update(credential_id);
        let entropy = hasher.finalize();
        
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
            transaction_history: Vec::new(),
        })
    }

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

        // Initialize balances
        self.update_account_balances(self.accounts.len()).await?;
        self.accounts.push(account.clone());
        
        Ok(account)
    }

    pub async fn send_transaction(
        &mut self,
        from_index: usize,
        to: &str,
        amount: U256,
        token_symbol: Option<String>,
        gas_price: Option<U256>,
    ) -> Result<H256> {
        let tx_hash = match token_symbol {
            Some(symbol) => {
                self.send_token(from_index, &symbol, to, amount, gas_price).await?
            },
            None => {
                self.send_eth(from_index, to, amount, gas_price).await?
            }
        };

        // Record transaction in history
        let from_account = self.accounts.get(from_index)
            .ok_or_else(|| anyhow!("Invalid account index"))?;

        self.transaction_history.push(TransactionHistory {
            hash: format!("0x{:x}", tx_hash),
            from: from_account.address.clone(),
            to: to.to_string(),
            value: amount,
            token_symbol,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            status: true,
        });

        Ok(tx_hash)
    }

    // Rest of the implementation remains the same as in your provided code
    // Including send_eth, send_token, update_account_balances, etc.
}

// Helper function to convert public key to Ethereum address
fn public_key_to_eth_address(public_key: &secp256k1::PublicKey) -> Address {
    let public_key = public_key.serialize_uncompressed();
    let hash = ethereum::util::keccak256(&public_key[1..]);
    Address::from_slice(&hash[12..])
}