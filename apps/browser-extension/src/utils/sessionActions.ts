export interface SessionActions {
  proposeSession: (params: ProposeSessionParams) => void;
  acceptSession: (params: AcceptSessionParams) => void;
  sendDirectMessage: (params: SendDirectMessageParams) => void;
  requestDeviceList: () => void;
}

export interface ProposeSessionParams {
  proposedSessionIdInput: string;
  totalParticipants: number;
  threshold: number;
  deviceId: string;
  connectedDevices: string[];
}

export interface AcceptSessionParams {
  sessionId: string;
  blockchain: string;
}

export interface SendDirectMessageParams {
  toDeviceId: string;
  fromDeviceId: string;
}

export function createSessionActions(): SessionActions {
  return {
    proposeSession: (params: ProposeSessionParams) => {
      const availableDevices = params.connectedDevices.filter(
        (p) => p !== params.deviceId,
      );

      if (availableDevices.length < params.totalParticipants - 1) {
        console.error(
          `Need at least ${params.totalParticipants - 1} other devices for a ${params.totalParticipants}-participant session`,
        );
        return;
      }

      if (params.threshold > params.totalParticipants) {
        console.error("Threshold cannot be greater than total participants");
        return;
      }

      if (params.threshold < 1) {
        console.error("Threshold must be at least 1");
        return;
      }

      const devicesToInclude = availableDevices.slice(0, params.totalParticipants - 1);
      const allParticipants = [params.deviceId, ...devicesToInclude];

      const sessionId =
        params.proposedSessionIdInput.trim() ||
        `wallet_${params.threshold}of${params.totalParticipants}_${Date.now()}`;

      chrome.runtime.sendMessage({
        type: "proposeSession",
        session_id: sessionId,
        total: params.totalParticipants,
        threshold: params.threshold,
        participants: allParticipants,
      });

      console.log(
        "[SessionActions] Proposing session:",
        sessionId,
        `(${params.threshold}-of-${params.totalParticipants})`,
        "with participants:",
        allParticipants,
      );
    },

    acceptSession: (params: AcceptSessionParams) => {
      chrome.runtime.sendMessage(
        {
          type: "acceptSession",
          session_id: params.sessionId,
          accepted: true,
          blockchain: params.blockchain,
        },
        (response) => {
          if (chrome.runtime.lastError) {
            console.error(
              "[SessionActions] Error accepting session:",
              chrome.runtime.lastError.message,
            );
          } else {
            console.log("[SessionActions] Session acceptance response:", response);
            if (!response?.success) {
              console.error("[SessionActions] Session acceptance failed:", response?.error);
            }
          }
        },
      );

      console.log(
        "[SessionActions] Accepting session invite:",
        params.sessionId,
        "with blockchain:",
        params.blockchain,
      );
    },

    sendDirectMessage: (params: SendDirectMessageParams) => {
      const testMessage = `Hello from ${params.fromDeviceId} at ${new Date().toLocaleTimeString()}`;
      
      chrome.runtime.sendMessage(
        {
          type: "sendDirectMessage",
          todeviceId: params.toDeviceId,
          message: testMessage,
        },
        (response) => {
          if (chrome.runtime.lastError) {
            console.error(
              "[SessionActions] Error sending direct message:",
              chrome.runtime.lastError.message,
            );
          } else {
            console.log("[SessionActions] Direct message response:", response);
            if (!response.success) {
              console.error(`Failed to send message: ${response.error}`);
            }
          }
        },
      );

      console.log(
        "[SessionActions] Sending direct message to:",
        params.toDeviceId,
        "Message:",
        testMessage,
      );
    },

    requestDeviceList: () => {
      console.log("[SessionActions] Requesting peer list");
      chrome.runtime.sendMessage({ type: "listdevices" }, (response) => {
        if (chrome.runtime.lastError) {
          console.error(
            "[SessionActions] Error requesting peer list:",
            chrome.runtime.lastError.message,
          );
          return;
        }
        console.log("[SessionActions] listdevices response:", response);
      });
    },
  };
}
