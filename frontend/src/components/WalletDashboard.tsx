import React, { useState, useEffect } from 'react';
import { Card, CardHeader, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Wallet, Send, RefreshCw, Plus } from 'lucide-react';

const WalletDashboard = () => {
  const [accounts, setAccounts] = useState([]);
  const [supportedTokens, setSupportedTokens] = useState([]);
  const [selectedAccount, setSelectedAccount] = useState(0);
  const [sendAmount, setSendAmount] = useState('');
  const [recipientAddress, setRecipientAddress] = useState('');
  const [selectedToken, setSelectedToken] = useState('ETH');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');

  useEffect(() => {
    fetchWalletInfo();
  }, []);

  const fetchWalletInfo = async () => {
    try {
      setIsLoading(true);
      const response = await fetch('/api/wallet/info');
      const data = await response.json();
      setAccounts(data.accounts);
      setSupportedTokens(data.supported_tokens);
    } catch (error) {
      setError('Failed to fetch wallet information');
    } finally {
      setIsLoading(false);
    }
  };

  const updateBalances = async () => {
    try {
      setIsLoading(true);
      const response = await fetch('/api/wallet/balances');
      const data = await response.json();
      setAccounts(data);
    } catch (error) {
      setError('Failed to update balances');
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreateWallet = async () => {
    try {
      setIsLoading(true);
      const credential = await navigator.credentials.create({
        publicKey: {
          challenge: new Uint8Array(32),
          rp: {
            name: "HD Wallet",
            id: window.location.hostname
          },
          user: {
            id: new Uint8Array(16),
            name: "wallet-user",
            displayName: "Wallet User"
          },
          pubKeyCredParams: [{
            type: "public-key",
            alg: -7
          }],
          timeout: 60000
        }
      });

      await fetch('/api/wallet/create', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ passkey: credential.id })
      });

      fetchWalletInfo();
    } catch (error) {
      setError('Failed to create wallet');
    } finally {
      setIsLoading(false);
    }
  };

  const handleDeriveAccount = async () => {
    try {
      setIsLoading(true);
      await fetch('/api/wallet/derive', { 
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      });
      fetchWalletInfo();
    } catch (error) {
      setError('Failed to create new account');
    } finally {
      setIsLoading(false);
    }
  };

  const handleSend = async () => {
    try {
      setIsLoading(true);
      await fetch('/api/wallet/send', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          from_index: selectedAccount,
          to: recipientAddress,
          amount: sendAmount,
          token_symbol: selectedToken === 'ETH' ? null : selectedToken
        })
      });

      setSendAmount('');
      setRecipientAddress('');
      fetchWalletInfo();
    } catch (error) {
      setError('Transaction failed');
    } finally {
      setIsLoading(false);
    }
  };

  const formatBalance = (balance, decimals = 18) => {
    return (Number(balance) / Math.pow(10, decimals)).toFixed(6);
  };

  return (
    <div className="max-w-4xl mx-auto p-4">
      <Card className="shadow-lg">
        <CardHeader className="flex flex-row justify-between items-center">
          <h1 className="text-2xl font-bold">HD Wallet</h1>
          <div className="space-x-2">
            <Button onClick={handleCreateWallet} variant="outline" disabled={isLoading}>
              Create Wallet
            </Button>
            <Button onClick={handleDeriveAccount} disabled={isLoading}>
              <Plus className="w-4 h-4 mr-2" />
              Add Account
            </Button>
            <Button onClick={updateBalances} variant="outline" disabled={isLoading}>
              <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
            </Button>
          </div>
        </CardHeader>

        {error && (
          <Alert variant="destructive" className="mx-4 mb-4">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        <CardContent>
          <Tabs defaultValue="accounts" className="space-y-4">
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="accounts" className="flex items-center">
                <Wallet className="w-4 h-4 mr-2" />
                Accounts
              </TabsTrigger>
              <TabsTrigger value="send" className="flex items-center">
                <Send className="w-4 h-4 mr-2" />
                Send
              </TabsTrigger>
            </TabsList>

            <TabsContent value="accounts" className="space-y-4">
              {accounts.map((account, index) => (
                <Card 
                  key={account.address}
                  className={`cursor-pointer transition-colors ${
                    selectedAccount === index ? 'border-blue-500' : ''
                  }`}
                  onClick={() => setSelectedAccount(index)}
                >
                  <CardContent className="p-4">
                    <div className="flex justify-between items-center">
                      <div className="space-y-1">
                        <p className="font-mono text-sm">{account.address}</p>
                        <p className="text-sm text-gray-500">Path: {account.path}</p>
                      </div>
                      <div className="text-right space-y-1">
                        <p className="font-bold">
                          {formatBalance(account.balance)} ETH
                        </p>
                        {Object.entries(account.tokens).map(([address, token]) => (
                          <p key={address} className="text-sm text-gray-600">
                            {formatBalance(token.balance, token.decimal)} {token.symbol}
                          </p>
                        ))}
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </TabsContent>

            <TabsContent value="send">
              <Card>
                <CardContent className="space-y-4 pt-4">
                  <div className="space-y-2">
                    <label className="text-sm font-medium">From Account</label>
                    <Select 
                      value={selectedAccount.toString()} 
                      onValueChange={(value) => setSelectedAccount(Number(value))}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="Select account" />
                      </SelectTrigger>
                      <SelectContent>
                        {accounts.map((account, index) => (
                          <SelectItem key={account.address} value={index.toString()}>
                            Account {index + 1} - {account.address.slice(0, 6)}...{account.address.slice(-4)}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>

                  <div className="space-y-2">
                    <label className="text-sm font-medium">Token</label>
                    <Select 
                      value={selectedToken}
                      onValueChange={setSelectedToken}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="Select token" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="ETH">ETH</SelectItem>
                        {supportedTokens.map((token) => (
                          <SelectItem key={token.address} value={token.symbol}>
                            {token.symbol}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>

                  <div className="space-y-2">
                    <label className="text-sm font-medium">Recipient Address</label>
                    <Input
                      value={recipientAddress}
                      onChange={(e) => setRecipientAddress(e.target.value)}
                      placeholder="0x..."