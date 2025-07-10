/* tslint:disable */
/* eslint-disable */
export function main(): void;
export class FrostDkgEd25519 {
  free(): void;
  constructor();
  init_dkg(participant_index: number, total: number, threshold: number): void;
  generate_round1(): string;
  add_round1_package(participant_index: number, package_hex: string): void;
  can_start_round2(): boolean;
  generate_round2(): string;
  add_round2_package(sender_index: number, package_hex: string): void;
  can_finalize(): boolean;
  finalize_dkg(): string;
  get_group_public_key(): string;
  get_address(): string;
  is_dkg_complete(): boolean;
  signing_commit(): string;
  add_signing_commitment(participant_index: number, commitment_hex: string): void;
  sign(message_hex: string): string;
  add_signature_share(participant_index: number, share_hex: string): void;
  aggregate_signature(message_hex: string): string;
  clear_signing_state(): void;
}
export class FrostDkgSecp256k1 {
  free(): void;
  constructor();
  init_dkg(participant_index: number, total: number, threshold: number): void;
  generate_round1(): string;
  add_round1_package(participant_index: number, package_hex: string): void;
  can_start_round2(): boolean;
  generate_round2(): string;
  add_round2_package(sender_index: number, package_hex: string): void;
  can_finalize(): boolean;
  finalize_dkg(): string;
  get_group_public_key(): string;
  get_address(): string;
  get_eth_address(): string;
  is_dkg_complete(): boolean;
  signing_commit(): string;
  add_signing_commitment(participant_index: number, commitment_hex: string): void;
  sign(message_hex: string): string;
  add_signature_share(participant_index: number, share_hex: string): void;
  aggregate_signature(message_hex: string): string;
  clear_signing_state(): void;
}
export class WasmError {
  free(): void;
  constructor(message: string);
  readonly message: string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wasmerror_free: (a: number, b: number) => void;
  readonly wasmerror_new: (a: number, b: number) => number;
  readonly wasmerror_message: (a: number) => [number, number];
  readonly __wbg_frostdkged25519_free: (a: number, b: number) => void;
  readonly frostdkged25519_new: () => number;
  readonly frostdkged25519_init_dkg: (a: number, b: number, c: number, d: number) => [number, number];
  readonly frostdkged25519_generate_round1: (a: number) => [number, number, number, number];
  readonly frostdkged25519_add_round1_package: (a: number, b: number, c: number, d: number) => [number, number];
  readonly frostdkged25519_can_start_round2: (a: number) => number;
  readonly frostdkged25519_generate_round2: (a: number) => [number, number, number, number];
  readonly frostdkged25519_add_round2_package: (a: number, b: number, c: number, d: number) => [number, number];
  readonly frostdkged25519_can_finalize: (a: number) => number;
  readonly frostdkged25519_finalize_dkg: (a: number) => [number, number, number, number];
  readonly frostdkged25519_get_group_public_key: (a: number) => [number, number, number, number];
  readonly frostdkged25519_get_address: (a: number) => [number, number, number, number];
  readonly frostdkged25519_is_dkg_complete: (a: number) => number;
  readonly frostdkged25519_signing_commit: (a: number) => [number, number, number, number];
  readonly frostdkged25519_add_signing_commitment: (a: number, b: number, c: number, d: number) => [number, number];
  readonly frostdkged25519_sign: (a: number, b: number, c: number) => [number, number, number, number];
  readonly frostdkged25519_add_signature_share: (a: number, b: number, c: number, d: number) => [number, number];
  readonly frostdkged25519_aggregate_signature: (a: number, b: number, c: number) => [number, number, number, number];
  readonly frostdkged25519_clear_signing_state: (a: number) => void;
  readonly __wbg_frostdkgsecp256k1_free: (a: number, b: number) => void;
  readonly frostdkgsecp256k1_new: () => number;
  readonly frostdkgsecp256k1_init_dkg: (a: number, b: number, c: number, d: number) => [number, number];
  readonly frostdkgsecp256k1_generate_round1: (a: number) => [number, number, number, number];
  readonly frostdkgsecp256k1_add_round1_package: (a: number, b: number, c: number, d: number) => [number, number];
  readonly frostdkgsecp256k1_can_start_round2: (a: number) => number;
  readonly frostdkgsecp256k1_generate_round2: (a: number) => [number, number, number, number];
  readonly frostdkgsecp256k1_add_round2_package: (a: number, b: number, c: number, d: number) => [number, number];
  readonly frostdkgsecp256k1_can_finalize: (a: number) => number;
  readonly frostdkgsecp256k1_finalize_dkg: (a: number) => [number, number, number, number];
  readonly frostdkgsecp256k1_get_group_public_key: (a: number) => [number, number, number, number];
  readonly frostdkgsecp256k1_get_address: (a: number) => [number, number, number, number];
  readonly frostdkgsecp256k1_is_dkg_complete: (a: number) => number;
  readonly frostdkgsecp256k1_signing_commit: (a: number) => [number, number, number, number];
  readonly frostdkgsecp256k1_add_signing_commitment: (a: number, b: number, c: number, d: number) => [number, number];
  readonly frostdkgsecp256k1_sign: (a: number, b: number, c: number) => [number, number, number, number];
  readonly frostdkgsecp256k1_add_signature_share: (a: number, b: number, c: number, d: number) => [number, number];
  readonly frostdkgsecp256k1_aggregate_signature: (a: number, b: number, c: number) => [number, number, number, number];
  readonly frostdkgsecp256k1_clear_signing_state: (a: number) => void;
  readonly frostdkgsecp256k1_get_eth_address: (a: number) => [number, number, number, number];
  readonly main: () => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
