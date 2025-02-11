use actix_web::{web, HttpResponse, Scope};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateWalletRequest {
    name: String,
}

#[derive(Serialize)]
pub struct WalletResponse {
    address: String,
    public_key: String,
}

pub fn wallet_routes() -> Scope {
    web::scope("/api")
        .route("/wallet/create", web::post().to(create_wallet))
        .route("/wallet/list", web::get().to(list_wallets))
        .route("/auth/register", web::post().to(register_passkey))
        .route("/auth/login", web::post().to(verify_passkey))
}