# HD Wallet with Passkey Authentication

## Overview
This implementation provides a hierarchical deterministic (HD) wallet with WebAuthn/Passkey authentication. The system allows users to securely create and manage multiple wallet addresses derived from a single master seed, with authentication handled through modern passkey technology.

## Technical Architecture

### Components

1. **HD Wallet Core**
   - Implements BIP32 for hierarchical deterministic wallets
   - Uses BIP39 for mnemonic seed generation
   - Supports multiple address derivation paths
   - Implements secure key storage and management

2. **Passkey Authentication**
   - WebAuthn-based authentication system
   - Supports resident keys for enhanced security
   - Implements user verification requirements
   - Handles key attestation and verification

3. **API Layer**
   - RESTful endpoints for wallet operations
   - WebSocket support for real-time updates
   - Authentication middleware
   - Rate limiting and security measures

### Security Features

1. **Key Management**
   - Master seed never leaves secure storage
   - Child keys are derived on-demand
   - Private keys are never exposed to the network
   - Implements secure key deletion

2. **Authentication**
   - Biometric authentication support via passkeys
   - No password storage required
   - Protection against phishing attacks
   - Hardware security key support

## User Flow

1. **First-time Setup**
   ```
   User -> Create Passkey -> Register Device -> Generate Master Seed -> Create First Wallet
   ```

2. **Return User Login**
   ```
   User -> Authenticate with Passkey -> Access Wallet
   ```

3. **Adding New Wallet Addresses**
   ```
   Authenticated User -> Request New Address -> Derive Child Key -> Return New Address
   ```

## API Documentation

### Authentication Endpoints

#### POST /api/auth/register
Register a new passkey for a user.

Request:
```json
{
  "username": "string"
}
```

Response:
```json
{
  "challenge": "string",
  "registration_options": {}
}
```

#### POST /api/auth/login
Authenticate using a registered passkey.

Request:
```json
{
  "credential": "string"
}
```

Response:
```json
{
  "token": "string"
}
```

### Wallet Endpoints

#### POST /api/wallet/create
Create a new wallet address.

Request:
```json
{
  "name": "string"
}
```

Response:
```json
{
  "address": "string",
  "public_key": "string"
}
```

#### GET /api/wallet/list
List all wallet addresses for the authenticated user.

Response:
```json
{
  "wallets": [
    {
      "name": "string",
      "address": "string",
      "public_key": "string"
    }
  ]
}
```

## Development Setup

1. Install dependencies:
   ```bash
   cargo install --path .
   ```

2. Set up the database:
   ```bash
   sqlx database setup
   ```

3. Run the development server:
   ```bash
   cargo run
   ```

## Security Considerations

1. **Key Storage**
   - Master seed must be stored in secure hardware when available
   - Use OS-provided secure storage APIs
   - Implement proper key deletion procedures

2. **Network Security**
   - All API endpoints must use HTTPS
   - Implement rate limiting
   - Use security headers (CORS, CSP, etc.)

3. **Authentication**
   - Implement proper session management
   - Use secure WebAuthn parameters
   - Implement proper error handling

## Error Handling

The system implements comprehensive error handling:

1. **Wallet Errors**
   - Invalid derivation paths
   - Key generation failures
   - Network connectivity issues

2. **Authentication Errors**
   - Invalid passkey data
   - Failed attestation
   - Device compatibility issues

## Testing

Run the test suite:
```bash
cargo test
```

The test suite includes:
- Unit tests for core wallet functionality
- Integration tests for API endpoints
- Security tests for authentication flow
- Performance tests for key derivation
