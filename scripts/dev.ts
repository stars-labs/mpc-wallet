#!/usr/bin/env bun

// Development script that watches for Rust changes and rebuilds WASM
import { watch } from 'fs';
import { spawn } from 'child_process';
import { resolve } from 'path';

const RUST_SRC_DIR = resolve('./src');
const CARGO_TOML = resolve('./Cargo.toml');

let building = false;
let wxtProcess: any = null;

function log(message: string) {
    console.log(`🔧 [DEV] ${new Date().toLocaleTimeString()} - ${message}`);
}

function buildWasm(): Promise<void> {
    return new Promise((resolve, reject) => {
        if (building) {
            log('Build already in progress, skipping...');
            return resolve();
        }

        building = true;
        log('Building WASM...');

        const buildProcess = spawn('bun', ['run', 'build:wasm:dev'], {
            stdio: 'inherit',
            shell: true
        });

        buildProcess.on('close', (code) => {
            building = false;
            if (code === 0) {
                log('✅ WASM build completed successfully');
                resolve();
            } else {
                log('❌ WASM build failed');
                reject(new Error(`Build failed with code ${code}`));
            }
        });
    });
}

function startWxt() {
    if (wxtProcess) {
        log('Restarting WXT...');
        wxtProcess.kill();
    }

    log('Starting WXT development server...');
    wxtProcess = spawn('bun', ['wxt'], {
        stdio: 'inherit',
        shell: true
    });

    wxtProcess.on('close', (code: number) => {
        if (code !== 0) {
            log(`WXT process exited with code ${code}`);
        }
    });
}

async function init() {
    log('🚀 Starting MPC Wallet development server...');

    // Initial WASM build
    try {
        await buildWasm();
        startWxt();
    } catch (error) {
        log('❌ Initial build failed');
        process.exit(1);
    }

    // Watch for Rust source changes
    log(`👀 Watching ${RUST_SRC_DIR} for changes...`);
    watch(RUST_SRC_DIR, { recursive: true }, async (eventType, filename) => {
        if (filename && (filename.endsWith('.rs') || filename === 'Cargo.toml')) {
            log(`📝 Detected change in ${filename}, rebuilding WASM...`);
            try {
                await buildWasm();
                log('🔄 WASM rebuilt, WXT will reload automatically');
            } catch (error) {
                log('❌ WASM rebuild failed');
            }
        }
    });

    // Watch Cargo.toml for dependency changes
    watch(CARGO_TOML, async () => {
        log('📦 Cargo.toml changed, rebuilding WASM...');
        try {
            await buildWasm();
        } catch (error) {
            log('❌ WASM rebuild failed');
        }
    });
}

// Handle graceful shutdown
process.on('SIGINT', () => {
    log('🛑 Shutting down development server...');
    if (wxtProcess) {
        wxtProcess.kill();
    }
    process.exit(0);
});

process.on('SIGTERM', () => {
    log('🛑 Shutting down development server...');
    if (wxtProcess) {
        wxtProcess.kill();
    }
    process.exit(0);
});

init();
