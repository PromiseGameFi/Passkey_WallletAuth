# HD Wallet with WebAuthn Authentication

A hierarchical deterministic (HD) wallet implementation with WebAuthn (passkey) authentication, built with Rust and Next.js.

## Project Structure
```
hdwallet/
├── backend/
│   ├── src/
│   │   ├── main.rs            # API server implementation
│   │   ├── wallet.rs          # HD wallet implementation
│   │   └── erc20_abi.json     # Token configurations
│   └── Cargo.toml
├── frontend/
│   ├── src/
│   │   ├── app/
│   │   │   └── page.tsx       # Main wallet interface
│   │   ├── components/
│   │   │   └── WalletDashboard.tsx
│   │   └── lib/
│   │       └── api.ts         # API client
│   ├── package.json
│   └── next.config.js
└── README.md
```

## Token Configuration (erc20_abi.json)
```json
{
  "abi": [...],  // Standard ERC20 ABI
  "tokens": {
    "USDT": {
      "address": "0x7169D38820dfd117C3FA1f22a697dBA58d90BA06",
      "decimals": 6,
      "symbol": "USDT"
    },
    "LINK": {
      "address": "0x779877A7B0D9E8603169DdbD7836e478b4624789",
      "decimals": 18,
      "symbol": "LINK"
    },
    "UNI": {
      "address": "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984",
      "decimals": 18,
      "symbol": "UNI"
    }
  }
}
```

## Setup & Installation

### Backend Setup
1. Install Rust and Cargo
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Install project dependencies
```bash
cd backend
cargo build
```

3. Configure environment variables
Create a `.env` file in the backend directory:
```
INFURA_PROJECT_ID=your_infura_project_id
PORT=8080
```

### Frontend Setup
1. Install Node.js and npm
2. Install dependencies
```bash
cd frontend
npm install
```

3. Configure environment variables
Create a `.env.local` file in the frontend directory:
```
NEXT_PUBLIC_API_URL=http://localhost:8080
```

## Running the Project

1. Start the backend server
```bash
cd backend
cargo run
```

2. Start the frontend development server
```bash
cd frontend
npm run dev
```

The application will be available at `http://localhost:3000`

## Key Features

### WebAuthn Authentication
- Uses WebAuthn for secure key generation and authentication
- Passkey credentials are used to derive the wallet's master seed
- Supports multiple devices and cross-platform synchronization

### HD Wallet Functionality
- BIP32 hierarchical deterministic wallet implementation
- BIP39 mnemonic generation from WebAuthn credentials
- Multiple account derivation from single master seed
- Supports ETH and ERC20 token transactions
- Balance tracking for ETH and supported tokens
- Sepolia testnet support

### Security Features
- Client-side key generation and signing
- No private key storage on server
- WebAuthn authentication prevents phishing
- Secure transaction signing

## API Endpoints

### Wallet Management
- `POST /wallet/create` - Create new wallet using WebAuthn credential
- `POST /wallet/derive` - Derive new account from master seed
- `GET /wallet/info` - Get wallet information and balances
- `GET /wallet/balances` - Update account balances

### Transactions
- `POST /wallet/send` - Send ETH or ERC20 tokens
  - Parameters:
    - `from_index`: Account index
    - `to`: Recipient address
    - `amount`: Transaction amount
    - `token_symbol`: Optional token symbol (null for ETH)
    - `gas_price`: Optional gas price

## Cargo.toml Configuration
```toml
[package]
name = "hd-wallet"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4.0"
actix-cors = "0.6"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
web3 = "0.18"
ethereum-types = "0.14"
bip39 = "1.0"
bitcoin = "0.29"
secp256k1 = "0.24"
sha2 = "0.10"
anyhow = "1.0"
dotenv = "0.15"
env_logger = "0.9"
```

## Development Notes

1. Token Management
- Add new tokens by updating `erc20_abi.json`
- Each token requires address, decimals, and symbol
- Use verified contract addresses

2. Transaction Handling
- Always verify gas estimates before sending
- Implement proper error handling for failed transactions
- Add retry mechanism for failed balance updates

3. Security Considerations
- Implement rate limiting
- Add transaction confirmation dialogs
- Validate all input addresses
- Add session management
- Implement proper error handling

4. Future Improvements
- Add support for multiple networks
- Implement transaction history
- Add token price tracking
- Support token swaps
- Add backup/recovery mechanism
