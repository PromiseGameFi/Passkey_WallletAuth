use actix_web::{web, App, HttpServer, HttpResponse, Error};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

// API State
pub struct AppState {
    wallet: Mutex<HDWallet>,
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
    token_address: Option<String>,
}

#[derive(Serialize)]
struct WalletResponse {
    accounts: Vec<WalletAccount>,
}

// API Routes implementation
async fn create_wallet(
    data: web::Data<AppState>,
    req: web::Json<CreateWalletRequest>,
) -> Result<HttpResponse, Error> {
    let wallet = HDWallet::new(req.passkey.as_bytes()).await?;
    *data.wallet.lock().unwrap() = wallet;
    
    Ok(HttpResponse::Ok().json("Wallet created successfully"))
}

async fn derive_account(
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let mut wallet = data.wallet.lock().unwrap();
    let account = wallet.derive_account()?;
    
    Ok(HttpResponse::Ok().json(account))
}

async fn send_transaction(
    data: web::Data<AppState>,
    req: web::Json<SendTransactionRequest>,
) -> Result<HttpResponse, Error> {
    let wallet = data.wallet.lock().unwrap();
    
    let tx_hash = match &req.token_address {
        Some(token_address) => {
            wallet.send_token(
                req.from_index,
                token_address,
                &req.to,
                req.amount.parse().unwrap()
            ).await?
        },
        None => {
            wallet.send_eth(
                req.from_index,
                &req.to,
                req.amount.parse().unwrap()
            ).await?
        }
    };
    
    Ok(HttpResponse::Ok().json(tx_hash))
}

async fn get_balances(
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let mut wallet = data.wallet.lock().unwrap();
    wallet.update_balances().await?;
    
    Ok(HttpResponse::Ok().json(WalletResponse {
        accounts: wallet.accounts.clone(),
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        wallet: Mutex::new(HDWallet::new(&[0; 32]).await.unwrap()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/wallet/create", web::post().to(create_wallet))
            .route("/wallet/derive", web::post().to(derive_account))
            .route("/wallet/send", web::post().to(send_transaction))
            .route("/wallet/balances", web::get().to(get_balances))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}