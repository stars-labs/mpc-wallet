// Define the Chain type to use in our components
export interface Chain {
    id: number;
    name: string;
    network: string;
    nativeCurrency?: {
        name: string;
        symbol: string;
        decimals: number;
    };
    rpcUrls?: {
        default: {
            http: string[];
        };
        public?: {
            http: string[];
        };
    };
    blockExplorers?: {
        default: {
            name: string;
            url: string;
        };
    };
}