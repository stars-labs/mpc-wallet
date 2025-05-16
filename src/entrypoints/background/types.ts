// This file defines the types for WebSocket messages between client and server

export interface ServerMsg {
    type: "peers" | "relay" | "error";
    peers?: string[];
    from?: string;
    data?: any;
    error?: string;
}

export type ClientMsg =
    | { type: "register"; peer_id: string }
    | { type: "list_peers" }
    | { type: "relay"; to: string; data: any };
