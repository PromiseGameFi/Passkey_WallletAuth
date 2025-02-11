# HD Wallet with Passkey Authentication

A hierarchical deterministic (HD) wallet implementation with passkey authentication, supporting Ethereum and ERC20 tokens on the Sepolia network.

## Table of Contents
- [Features](#features)
- [Project Structure](#project-structure)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Running the Project](#running-the-project)
- [API Documentation](#api-documentation)
- [Frontend Documentation](#frontend-documentation)

## Features

- BIP32 Hierarchical Deterministic Wallet implementation
- BIP39 Mnemonic code generation
- Passkey authentication for enhanced security
- Multiple account derivation
- ETH and ERC20 token support
- Sepolia testnet integration
- Modern React frontend with shadcn/ui components

## Project Structure

```
hd-wallet/
├── backend/
│   ├── src/
│   │   ├── main.rs          # API server implementation
│   │   ├── wallet.rs        # Core wallet implementation
│   │   └── erc20_abi.json   # ERC20 contract ABI
│   ├── Cargo.toml
│   └── .env
├── frontend/
│   ├── src/
│   │   ├── app/
│   │   │   └── page.tsx     # Main page component
│   │   ├── components/
│   │   │   └── WalletDashboard.tsx
│   │   └── lib/
│   ├── package.json
│   └── .env.local
└── README.md
```

## Prerequisites

- Rust 1.70 or higher
- Node.js 18 or higher
- npm or yarn
- An Infura account for Sepolia testnet access

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd hd-wallet
```

2. Install backend dependencies:
```bash
cd backend
cargo build
```

3. Install frontend dependencies:
```bash
cd frontend
npm install
```

## Configuration

1. Backend Configuration (.env):
```env
RUST_LOG=debug
SERVER_PORT=8080
INFURA_PROJECT_ID=your_infura_project_id
INFURA_PROJECT_SECRET=your_infura_project_secret
```

2. Frontend Configuration (.env.local):
```env
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_INFURA_PROJECT_ID=your_infura_project_id
```

## Running the Project

1. Start the backend server:
```bash
cd backend
cargo run
```

2. Start the frontend development server:
```bash
cd frontend
npm run dev
```

The application will be available at `http://localhost:3000`

## API Documentation

### Endpoints

#### POST /wallet/create
Creates a new HD wallet with passkey authentication.

Request body:
```json
{
  "passkey": "string"
}
```

#### POST /wallet/derive
Derives a new account from the HD wallet.

#### POST /wallet/send
Sends ETH or ERC20 tokens.

Request body:
```json
{
  "from_index": 0,
  "to": "0x...",
  "amount": "1.0",
  "token_address": "0x..." // optional, for ERC20 transfers
}
```

#### GET /wallet/balances
Retrieves all account balances.

## Frontend Documentation

### Component Structure

- `WalletDashboard`: Main component handling wallet functionality
- Tabs:
  - Accounts: Displays derived accounts and balances
  - Send: Interface for sending transactions

### Features

1. Wallet Creation:
   - Click "Create Wallet" button
   - Complete passkey registration
   - Wallet is created and stored securely

2. Account Management:
   - View all derived accounts
   - Add new accounts
   - View ETH and token balances

3. Sending Transactions:
   - Select source account
   - Choose token (ETH or ERC20)
   - Enter recipient address
   - Specify amount
   - Confirm transaction

### Styling

The project uses:
- Tailwind CSS for utility-first styling
- shadcn/ui for component library
- Lucide icons for iconography

## Security Considerations

1. Passkey Authentication:
   - Uses WebAuthn standard
   - Keys never leave the user's device
   - Resistant to phishing attacks

2. Private Key Security:
   - Private keys are derived on-demand
   - Never stored in plaintext
   - Protected by hardware security when available

3. Transaction Security:
   - All transactions require explicit user confirmation
   - Support for hardware wallet integration (future feature)

## Development Guidelines

1. Code Style:
   - Follow Rust formatting guidelines
   - Use ESLint for TypeScript/React code
   - Document all public functions and types

2. Testing:
   - Write unit tests for all Rust functions
   - Include integration tests for API endpoints
   - Test frontend components with React Testing Library

3. Pull Requests:
   - Include detailed description
   - Add tests for new features
   - Update documentation as needed

## Troubleshooting

Common issues and solutions:

1. Connection Issues:
   - Verify Infura API credentials
   - Check network connectivity
   - Ensure correct network (Sepolia) is selected

2. Transaction Failures:
   - Verify sufficient balance
   - Check gas price and limits
   - Confirm network congestion

3. Passkey Issues:
   - Ensure browser supports WebAuthn
   - Check device compatibility
   - Verify correct origin settings

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the MIT License.
