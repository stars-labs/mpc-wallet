// Application-Level Messages (sent over established WebRTC Data Channel)
// Renamed from WebRTCAppMessage to WebRTCMessage to align with Rust.
// Using 'any' for DKG packages as their structure is complex and not fully defined for TS here.
export type WebRTCAppMessage =
  | { webrtc_msg_type: 'SimpleMessage'; text: string }
  | { webrtc_msg_type: 'DkgRound1Package'; package: any } // frost_core::keys::dkg::round1::Package<Ed25519Sha512>
  | { webrtc_msg_type: 'DkgRound2Package'; package: any } // frost_core::keys::dkg::round2::Package<Ed25519Sha512>
  | { webrtc_msg_type: 'ChannelOpen'; peer_id: string }
  | { webrtc_msg_type: 'MeshReady'; session_id: string; peer_id: string }
  // FROST Signing Messages (compatible with CLI nodes)
  | { webrtc_msg_type: 'SignTx'; message_bytes: number[]; transaction_data?: any }
  | { webrtc_msg_type: 'SignCommitment'; sender_identifier: string; commitment: any }
  | { webrtc_msg_type: 'SignShare'; sender_identifier: string; share: any }
  | { webrtc_msg_type: 'SignAggregated'; signature_bytes: number[] }
  | { webrtc_msg_type: 'SignerSelection'; selected_identifiers: string[] };