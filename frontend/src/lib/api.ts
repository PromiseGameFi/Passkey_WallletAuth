export interface TokenInfo {
    symbol: string;
    address: string;
    decimals: number;
    name: string;
  }
  
  export interface TokenBalance {
    symbol: string;
    balance: string;
    decimal: number;
    contract_address: string;
    name: string;
  }
  
  export interface WalletAccount {
    address: string;
    path: string;
    index: number;
    balance: string;
    tokens: Record<string, TokenBalance>;
  }
  
  export interface TransactionHistory {
    hash: string;
    from: string;
    to: string;
    value: string;
    token_symbol?: string;
    timestamp: number;
    status: boolean;
  }
  
  const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
  
  export async function createWallet(credential: PublicKeyCredential): Promise<void> {
    const response = await fetch(`${API_BASE_URL}/wallet/create`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        credential_id: btoa(String.fromCharCode(...new Uint8Array(credential.rawId)))
      }),
    });
  
    if (!response.ok) {
      throw new Error(`Failed to create wallet: ${response.statusText}`);
    }
  }
  
  export async function deriveAccount(): Promise<WalletAccount> {
    const response = await fetch(`${API_BASE_URL}/wallet/derive`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
    });
  
    if (!response.ok) {
      throw new Error(`Failed to derive account: ${response.statusText}`);
    }
  
    return response.json();
  }
  
  export async function getWalletInfo(): Promise<{
    accounts: WalletAccount[];
    supported_tokens: TokenInfo[];
  }> {
    const response = await fetch(`${API_BASE_URL}/wallet/info`);
    
    if (!response.ok) {
      throw new Error(`Failed to get wallet info: ${response.statusText}`);
    }
  
    return response.json();
  }
  
  export async function sendTransaction(params: {
    from_index: number;
    to: string;
    amount: string;
    token_symbol?: string;
    gas_price?: string;
  }): Promise<{ hash: string }> {
    const response = await fetch(`${API_BASE_URL}/wallet/send`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(params),
    });
  
    if (!response.ok) {
      throw new Error(`Transaction failed: ${response.statusText}`);
    }
  
    return response.json();
  }
  
  export async function updateBalances(): Promise<WalletAccount[]> {
    const response = await fetch(`${API_BASE_URL}/wallet/balances`);
    
    if (!response.ok) {
      throw new Error(`Failed to update balances: ${response.statusText}`);
    }
  
    return response.json();
  }