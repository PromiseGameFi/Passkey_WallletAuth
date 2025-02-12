use actix_web::{web, App, HttpServer, HttpResponse, Error};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use web3::transports::Http;
use dotenv::dotenv;
use log::{info, error};

mod wallet;
use wallet::HDWallet;

// API State
pub struct AppState {
    wallet: Mutex<Option<HDWallet<Http>>>,
}

// Request/Response structures
#[derive(Deserialize)]
struct CreateWalletRequest {
    credential_id: String,
}

#[derive(Deserialize)]
struct SendTransactionRequest {
    from_index: usize,
    to: String,
    amount: String,
    token_symbol: Option<String>,
    gas_price: Option<String>,
}

#[derive(Serialize)]
struct TransactionResponse {
    hash: String,
}

async fn create_wallet(
    data: web::Data<AppState>,
    req: web::Json<CreateWalletRequest>,
) -> Result<HttpResponse, Error> {
    let credential_bytes = base64::decode(&req.credential_id)
        .map_err(actix_web::error::ErrorBadRequest)?;

    let transport = Http::new(&std::env::var("INFURA_URL")
        .unwrap_or_else(|_| "https://sepolia.infura.io/v3/YOUR-PROJECT-ID".to_string()))?;
    let web3 = web3::Web3::new(transport);
    
    let wallet = HDWallet::new(&credential_bytes, web3).await
        .map_err(|e| {
            error!("Failed to create wallet: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    *data.wallet.lock().unwrap() = Some(wallet);
    
    Ok(HttpResponse::Ok().json("Wallet created successfully"))
}

async fn derive_account(
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let mut wallet_lock = data.wallet.lock().unwrap();
    let wallet = wallet_lock.as_mut()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Wallet not initialized"))?;

    let account = wallet.derive_account().await
        .map_err(|e| {
            error!("Failed to derive account: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    Ok(HttpResponse::Ok().json(account))
}

async fn send_transaction(
    data: web::Data<AppState>,
    req: web::Json<SendTransactionRequest>,
) -> Result<HttpResponse, Error> {
    let mut wallet_lock = data.wallet.lock().unwrap();
    let wallet = wallet_lock.as_mut()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Wallet not initialized"))?;
    
    let amount = ethereum_types::U256::from_dec_str(&req.amount)
        .map_err(actix_web::error::ErrorBadRequest)?;
    
    let gas_price = req.gas_price.as_ref()
        .map(|p| ethereum_types::U256::from_dec_str(p))
        .transpose()
        .map_err(actix_web::error::ErrorBadRequest)?;

    let tx_hash = wallet.send_transaction(
        req.from_index,
        &req.to,
        amount,
        req.token_symbol.clone(),
        gas_price,
    ).await.map_err(|e| {
        error!("Transaction failed: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;
    
    Ok(HttpResponse::Ok().json(TransactionResponse {
        hash: format!("0x{:x}", tx_hash)
    }))
}

async fn get_wallet_info(
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let wallet_lock = data.wallet.lock().unwrap();
    let wallet = wallet_lock.as_ref()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Wallet not initialized"))?;
    
    Ok(HttpResponse::Ok().json(wallet.accounts.clone()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let app_state = web::Data::new(AppState {
        wallet: Mutex::new(None),
    });

    info!("Starting server at http://127.0.0.1:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .route("/wallet/create", web::post().to(create_wallet))
            .route("/wallet/derive", web::post().to(derive_account))
            .route("/wallet/send", web::post().to(send_transaction))
            .route("/wallet/info", web::get().to(get_wallet_info))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}