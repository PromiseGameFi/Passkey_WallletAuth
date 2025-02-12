use actix_web::{web, App, HttpServer, HttpResponse, Error};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use ethereum_types::U256;
use web3::transports::Http;

// API State
pub struct AppState {
    wallet: Mutex<HDWallet<Http>>,
}

// Request/Response structures
#[derive(Deserialize)]
struct CreateWalletRequest {
    passkey: String,
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
struct TokenListResponse {
    tokens: Vec<TokenInfo>,
}

#[derive(Serialize)]
struct TokenInfo {
    symbol: String,
    address: String,
    decimals: u8,
}

#[derive(Serialize)]
struct WalletResponse {
    accounts: Vec<WalletAccount>,
    supported_tokens: Vec<TokenInfo>,
}

// API Routes implementation
async fn create_wallet(
    data: web::Data<AppState>,
    req: web::Json<CreateWalletRequest>,
) -> Result<HttpResponse, Error> {
    let transport = Http::new("https://sepolia.infura.io/v3/YOUR-PROJECT-ID")?;
    let web3 = web3::Web3::new(transport);
    
    let wallet = HDWallet::new(req.passkey.as_bytes(), web3).await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    *data.wallet.lock().unwrap() = wallet;
    
    Ok(HttpResponse::Ok().json("Wallet created successfully"))
}

async fn derive_account(
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let mut wallet = data.wallet.lock().unwrap();
    let account = wallet.derive_account().await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(account))
}

async fn send_transaction(
    data: web::Data<AppState>,
    req: web::Json<SendTransactionRequest>,
) -> Result<HttpResponse, Error> {
    let wallet = data.wallet.lock().unwrap();
    
    let amount = U256::from_dec_str(&req.amount)
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;
    
    let gas_price = req.gas_price.as_ref()
        .map(|p| U256::from_dec_str(p))
        .transpose()
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    let tx_hash = match &req.token_symbol {
        Some(token) => {
            wallet.send_token(
                req.from_index,
                token,
                &req.to,
                amount,
                gas_price,
            ).await
        },
        None => {
            wallet.send_eth(
                req.from_index,
                &req.to,
                amount,
                gas_price,
            ).await
        }
    }.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(format!("0x{:x}", tx_hash)))
}

async fn get_wallet_info(
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let wallet = data.wallet.lock().unwrap();
    
    let supported_tokens: Vec<TokenInfo> = wallet.list_supported_tokens()
        .into_iter()
        .filter_map(|symbol| {
            wallet.get_token_info(&symbol).ok().map(|(address, decimals)| TokenInfo {
                symbol,
                address: address.to_string(),
                decimals,
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(WalletResponse {
        accounts: wallet.accounts.clone(),
        supported_tokens,
    }))
}

async fn update_balances(
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let mut wallet = data.wallet.lock().unwrap();
    
    for i in 0..wallet.get_account_count() {
        wallet.update_account_balances(i).await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    }
    
    Ok(HttpResponse::Ok().json(wallet.accounts.clone()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let app_state = web::Data::new(AppState {
        wallet: Mutex::new(HDWallet::new(&[0; 32], web3::Web3::new(
            Http::new("https://sepolia.infura.io/v3/YOUR-PROJECT-ID")?
        )).await.unwrap()),
    });

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
            .route("/wallet/balances", web::get().to(update_balances))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}