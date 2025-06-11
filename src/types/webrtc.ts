// Application-Level Messages (sent over established WebRTC Data Channel)
// Renamed from WebRTCAppMessage to WebRTCMessage to align with Rust.
// Using 'any' for DKG packages as their structure is complex and not fully defined for TS here.
export type WebRTCAppMessage =
  | { webrtc_msg_type: 'SimpleMessage'; text: string }
  | { webrtc_msg_type: 'DkgRound1Package'; package: any } // frost_core::keys::dkg::round1::Package<Ed25519Sha512>
  | { webrtc_msg_type: 'DkgRound2Package'; package: any } // frost_core::keys::dkg::round2::Package<Ed25519Sha512>
  | { webrtc_msg_type: 'ChannelOpen'; peer_id: string }
  | { webrtc_msg_type: 'MeshReady'; session_id: string; peer_id: string }
  // Signing Messages
  | { webrtc_msg_type: 'SigningRequest'; signing_id: string; transaction_data: string; threshold: number; participants: string[] }
  | { webrtc_msg_type: 'SigningAcceptance'; signing_id: string; accepted: boolean }
  | { webrtc_msg_type: 'SignerSelection'; signing_id: string; selected_signers: string[] }
  | { webrtc_msg_type: 'SigningCommitment'; signing_id: string; commitment: any } // FROST commitment
  | { webrtc_msg_type: 'SignatureShare'; signing_id: string; signature_share: any } // FROST signature share
  | { webrtc_msg_type: 'AggregatedSignature'; signing_id: string; signature: string }; // Final signature as string