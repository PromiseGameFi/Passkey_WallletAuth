use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey};
use tiny_bip39::{Mnemonic, MnemonicType};
use secp256k1::Secp256k1;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Invalid mnemonic")]
    InvalidMnemonic,
    #[error("Derivation error")]
    DerivationError,
}

pub struct HDWallet {
    master_key: ExtendedPrivKey,
    secp: Secp256k1<bitcoin::secp256k1::All>,
}

impl HDWallet {
    pub fn new() -> Result<Self, WalletError> {
        let mnemonic = Mnemonic::new(MnemonicType::Words12, bitcoin::Network::Bitcoin);
        let seed = mnemonic.to_seed("");
        let secp = Secp256k1::new();
        
        let master_key = ExtendedPrivKey::new_master(bitcoin::Network::Bitcoin, &seed)
            .map_err(|_| WalletError::DerivationError)?;
            
        Ok(Self { master_key, secp })
    }
    
    pub fn derive_child_wallet(&self, index: u32) -> Result<ExtendedPubKey, WalletError> {
        let path = format!("m/44'/0'/0'/0/{}", index);
        let child_priv = self.master_key
            .derive_priv(&self.secp, &path.parse().unwrap())
            .map_err(|_| WalletError::DerivationError)?;
            
        Ok(ExtendedPubKey::from_priv(&self.secp, &child_priv))
    }
}
