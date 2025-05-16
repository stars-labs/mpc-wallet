// Based on /home/freeman.xiong/Documents/github/hecoinfo/crypto-rust-tools/webrtc-signal-server/src/lib.rs

export interface SessionInfo {
    session_id: string;
    total: number;
    threshold: number;
    participants: string[];
}

// Server Messages
export interface ServerMsgPeers {
    type: "peers";
    peers: string[];
}

export interface ServerMsgRelay {
    type: "relay";
    from: string;
    data: any; // Using 'any' for serde_json::Value, can be refined if specific structure is known
}

export interface ServerMsgError {
    type: "error";
    error: string;
}

export type ServerMsg = ServerMsgPeers | ServerMsgRelay | ServerMsgError;

// Client Messages
export interface ClientMsgRegister {
    type: "register";
    peer_id: string;
}

export interface ClientMsgListPeers {
    type: "list_peers";
}

export interface ClientMsgRelay {
    type: "relay";
    to: string;
    data: any; // Using 'any' for serde_json::Value
}

export type ClientMsg = ClientMsgRegister | ClientMsgListPeers | ClientMsgRelay;

type MessageCallback = (message: ServerMsg) => void;
type ErrorCallback = (error: Event) => void;
type CloseCallback = (event: CloseEvent) => void;
type OpenCallback = (event: Event) => void;

import { ClientMsg, ServerMsg } from "./types";

export type { ServerMsg, ClientMsg };

export class WebSocketClient {
    private ws: WebSocket | null = null;
    private url: string;
    private onOpenCallbacks: Array<() => void> = [];
    private onMessageCallbacks: Array<(message: ServerMsg) => void> = [];
    private onErrorCallbacks: Array<(event: Event) => void> = [];
    private onCloseCallbacks: Array<(event: CloseEvent) => void> = [];

    constructor(url: string) {
        this.url = url;
    }

    public onOpen(callback: () => void): void {
        this.onOpenCallbacks.push(callback);
    }

    public onMessage(callback: (message: ServerMsg) => void): void {
        this.onMessageCallbacks.push(callback);
    }

    public onError(callback: (event: Event) => void): void {
        this.onErrorCallbacks.push(callback);
    }

    public onClose(callback: (event: CloseEvent) => void): void {
        this.onCloseCallbacks.push(callback);
    }

    public connect(): void {
        try {
            this.ws = new WebSocket(this.url);

            this.ws.onopen = () => {
                console.log("WebSocket connection established");
                this.onOpenCallbacks.forEach(callback => callback());
            };

            this.ws.onmessage = (event) => {
                try {
                    const message: ServerMsg = JSON.parse(event.data);
                    console.log("Received from server:", message);
                    this.onMessageCallbacks.forEach(callback => callback(message));
                } catch (error) {
                    console.error("Error parsing WebSocket message:", error);
                }
            };

            this.ws.onerror = (event) => {
                console.error("WebSocket error:", event);
                this.onErrorCallbacks.forEach(callback => callback(event));
            };

            this.ws.onclose = (event) => {
                console.log("WebSocket connection closed:", event.code, event.reason);
                this.onCloseCallbacks.forEach(callback => callback(event));
                this.ws = null;
            };
        } catch (error) {
            console.error("Error establishing WebSocket connection:", error);
        }
    }

    public register(peerId: string): void {
        this.sendMessage({
            type: "register",
            peer_id: peerId
        });
    }

    public listPeers(): void {
        console.log("Sending listPeers command to WebSocket server");
        this.sendMessage({
            type: "list_peers"
        });
    }

    public relayMessage(to: string, data: any): void {
        this.sendMessage({
            type: "relay",
            to,
            data
        });
    }

    private sendMessage(message: ClientMsg): void {
        if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
            console.error("WebSocket is not connected. Cannot send message:", message);
            return;
        }

        try {
            this.ws.send(JSON.stringify(message));
            console.log("Sent to server:", message);
        } catch (error) {
            console.error("Error sending WebSocket message:", error);
        }
    }

    public getReadyState(): number {
        return this.ws?.readyState ?? WebSocket.CLOSED;
    }

    public disconnect(): void {
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
    }
}
