#!/usr/bin/env bun

// Performance monitoring script for MPC Wallet development
import { performance } from 'perf_hooks';
import { watch } from 'fs';
import { resolve } from 'path';

interface BuildMetrics {
    wasmBuildTime: number;
    wxtBuildTime: number;
    totalTime: number;
    timestamp: Date;
}

class PerformanceMonitor {
    private metrics: BuildMetrics[] = [];
    private currentBuildStart: number = 0;

    startBuild() {
        this.currentBuildStart = performance.now();
        console.log('⏱️  Build started...');
    }

    endBuild(type: 'wasm' | 'wxt' | 'total') {
        const buildTime = performance.now() - this.currentBuildStart;
        console.log(`✅ ${type.toUpperCase()} build completed in ${buildTime.toFixed(2)}ms`);

        if (type === 'total') {
            this.metrics.push({
                wasmBuildTime: 0, // Will be updated by actual measurements
                wxtBuildTime: 0,  // Will be updated by actual measurements
                totalTime: buildTime,
                timestamp: new Date()
            });

            this.showStats();
        }

        return buildTime;
    }

    showStats() {
        if (this.metrics.length === 0) return;

        const recent = this.metrics.slice(-5);
        const avgTotal = recent.reduce((sum, m) => sum + m.totalTime, 0) / recent.length;

        console.log('\n📊 Performance Stats (last 5 builds):');
        console.log(`   Average total build time: ${avgTotal.toFixed(2)}ms`);
        console.log(`   Latest build: ${recent[recent.length - 1].totalTime.toFixed(2)}ms`);
        console.log(`   Total builds: ${this.metrics.length}\n`);
    }

    exportMetrics() {
        const data = {
            metrics: this.metrics,
            summary: {
                totalBuilds: this.metrics.length,
                averageTime: this.metrics.reduce((sum, m) => sum + m.totalTime, 0) / this.metrics.length,
                lastBuild: this.metrics[this.metrics.length - 1]
            }
        };

        Bun.write('./build-metrics.json', JSON.stringify(data, null, 2));
        console.log('📈 Metrics exported to build-metrics.json');
    }
}

const monitor = new PerformanceMonitor();

// Export for use in other scripts
export { PerformanceMonitor };

// If run directly, start monitoring
if (import.meta.main) {
    console.log('🔍 Performance monitoring started...');

    // Watch for build completion indicators
    const wxtOutputDir = resolve('./.wxt');
    const pkgDir = resolve('./pkg');

    watch(pkgDir, (eventType, filename) => {
        if (filename && filename.includes('mpc_wallet_bg.wasm')) {
            console.log('🦀 WASM build detected');
        }
    });

    watch(wxtOutputDir, { recursive: true }, (eventType, filename) => {
        if (filename && filename.includes('manifest.json')) {
            console.log('📦 Extension build detected');
        }
    });

    // Export metrics on exit
    process.on('SIGINT', () => {
        monitor.exportMetrics();
        process.exit(0);
    });
}
