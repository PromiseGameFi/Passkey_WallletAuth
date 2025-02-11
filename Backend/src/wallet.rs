use bip39::{Mnemonic, Language};
use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey};
use secp256k1::{Secp256k1, SecretKey};
use web_sys::{CredentialCreationOptions, PublicKeyCredentialCreationOptions};
use ethereum_types::{Address, U256};
use web3::contract::{Contract, Options};
use web3::types::{TransactionParameters, TransactionRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

// Wallet Account Structure
#[derive(Clone, Serialize, Deserialize)]
pub struct WalletAccount {
    address: String,
    path: String,
    index: u32,
    balance: U256,
    tokens: HashMap<String, TokenBalance>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    symbol: String,
    balance: U256,
    decimal: u8,
    contract_address: String,
}

pub struct HDWallet {
    master_key: ExtendedPrivKey,
    accounts: Vec<WalletAccount>,
    web3: web3::Web3<web3::transports::Http>,
}

impl HDWallet {
    // Create new wallet with passkey authentication
    pub async fn new(passkey: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        // Generate entropy from passkey
        let entropy = sha256::Hash::hash(passkey);
        
        // Generate mnemonic from entropy
        let mnemonic = Mnemonic::from_entropy(&entropy, Language::English)?;
        
        // Generate master key
        let seed = mnemonic.to_seed("");
        let secp = Secp256k1::new();
        let master_key = ExtendedPrivKey::new_master(bitcoin::Network::Bitcoin, &seed)?;

        let transport = web3::transports::Http::new("https://sepolia.infura.io/v3/YOUR_PROJECT_ID")?;
        let web3 = web3::Web3::new(transport);

        Ok(Self {
            master_key,
            accounts: Vec::new(),
            web3,
        })
    }

    // Derive new account
    pub fn derive_account(&mut self) -> Result<WalletAccount, Box<dyn std::error::Error>> {
        let index = self.accounts.len() as u32;
        let path = format!("m/44'/60'/0'/0/{}", index);
        
        let child_key = self.master_key.derive_priv(&secp256k1::Secp256k1::new(), 
            &bitcoin::util::bip32::DerivationPath::from_str(&path)?)?;
        
        let secret_key = SecretKey::from_slice(&child_key.private_key.secret_bytes())?;
        let public_key = secp256k1::PublicKey::from_secret_key(&Secp256k1::new(), &secret_key);
        
        let address = public_key_to_eth_address(&public_key);

        let account = WalletAccount {
            address: format!("0x{:x}", address),
            path,
            index,
            balance: U256::zero(),
            tokens: HashMap::new(),
        };

        self.accounts.push(account.clone());
        Ok(account)
    }

    // Send ETH
    pub async fn send_eth(&self, from_index: usize, to: &str, amount: U256) 
        -> Result<String, Box<dyn std::error::Error>> {
        let from_account = &self.accounts[from_index];
        
        let tx = TransactionRequest {
            from: Address::from_str(&from_account.address)?,
            to: Some(Address::from_str(to)?),
            value: amount,
            ..Default::default()
        };

        let tx_hash = self.web3.eth().send_transaction(tx).await?;
        Ok(format!("0x{:x}", tx_hash))
    }

    // Send ERC20 token
    pub async fn send_token(&self, from_index: usize, token_address: &str, to: &str, amount: U256) 
        -> Result<String, Box<dyn std::error::Error>> {
        let from_account = &self.accounts[from_index];
        
        let contract = Contract::from_json(
            self.web3.eth(),
            Address::from_str(token_address)?,
            include_bytes!("erc20_abi.json")
        )?;

        let tx = contract.call(
            "transfer",
            (Address::from_str(to)?, amount),
            Address::from_str(&from_account.address)?,
            Options::default()
        ).await?;

        Ok(format!("0x{:x}", tx))
    }

    // Update account balances
    pub async fn update_balances(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for account in &mut self.accounts {
            let address = Address::from_str(&account.address)?;
            account.balance = self.web3.eth().balance(address, None).await?;

            // Update common token balances (example with DAI on Sepolia)
            let dai_address = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
            let contract = Contract::from_json(
                self.web3.eth(),
                Address::from_str(dai_address)?,
                include_bytes!("erc20_abi.json")
            )?;

            let balance: U256 = contract.query(
                "balanceOf",
                (address,),
                None,
                Options::default(),
                None,
            ).await?;

            account.tokens.insert(
                dai_address.to_string(),
                TokenBalance {
                    symbol: "DAI".to_string(),
                    balance,
                    decimal: 18,
                    contract_address: dai_address.to_string(),
                }
            );
        }
        Ok(())
    }
}

// Helper function to convert public key to Ethereum address
fn public_key_to_eth_address(public_key: &secp256k1::PublicKey) -> Address {
    let public_key = public_key.serialize_uncompressed();
    let hash = ethereum::util::keccak256(&public_key[1..]);
    Address::from_slice(&hash[12..])
}