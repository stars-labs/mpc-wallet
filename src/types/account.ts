export interface Account {
    address: string;
    name: string;
    publicKey?: string;
    created: number;
    lastUsed?: number;
}

export interface AccountStorage {
    accounts: Account[];
    currentAccount: string | null;
}
