export interface Account {
    id: string;
    address: string;
    name: string;
    balance: string;
    publicKey?: string;
    blockchain: 'ethereum' | 'solana';
    created?: number;
    lastUsed?: number;
}

export interface AccountStorage {
    accounts: Account[];
    currentAccount: string | null;
}
