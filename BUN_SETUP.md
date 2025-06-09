# MPC Wallet - Bun Development Setup

This project has been optimized for development with [Bun](https://bun.sh/), a fast JavaScript runtime and package manager.

## 🚀 Quick Start

```bash
# Install dependencies (much faster than npm/yarn)
bun install

# Start development server with WASM auto-rebuild
bun run dev

# Run tests with coverage
bun test --coverage

# Build for production
bun run build
```

## 📁 Bun Configuration Files

- `bunfig.toml` - Main Bun configuration
- `bun.config.ts` - Development and build settings
- `bun.test.config.ts` - Test-specific configuration
- `test-setup.ts` - Global test setup with WASM initialization

## 🔧 Available Scripts

### Development
- `bun run dev` - Start development server with automatic WASM rebuilding
- `bun run dev:simple` - Basic development without auto-rebuild
- `bun run dev:firefox` - Development for Firefox
- `bun run dev:edge` - Development for Edge

### Testing
- `bun test` - Run all tests with coverage
- `bun test --watch` - Watch mode for tests
- `bun test --verbose` - Verbose test output
- `bun run test:webrtc` - Run specific WebRTC tests

### Building
- `bun run build` - Production build for Chrome
- `bun run build:firefox` - Production build for Firefox
- `bun run build:edge` - Production build for Edge
- `bun run build:wasm` - Build WASM modules only
- `bun run build:wasm:dev` - Build WASM in development mode

### Maintenance
- `bun run clean` - Clean all build artifacts
- `bun run fresh` - Clean install and rebuild everything
- `bun run type-check` - TypeScript type checking
- `bun run lint` - Run linting checks

## 🎯 Key Features

### ⚡ Performance Optimizations
- **Fast installs**: Bun's package manager is significantly faster than npm/yarn
- **Quick tests**: Bun's test runner with built-in coverage
- **WASM integration**: Optimized WebAssembly loading and initialization
- **Hot reloading**: Automatic WASM rebuilding when Rust source changes

### 🧪 Enhanced Testing
- **Global WASM setup**: Automatic initialization in `test-setup.ts`
- **Coverage reporting**: Built-in coverage with HTML/LCOV reports
- **Fast execution**: Tests run significantly faster than with Jest/Vitest

### 🔧 Development Tools
- **Smart dev script**: `scripts/dev.ts` watches Rust files and rebuilds WASM automatically
- **Performance monitoring**: `scripts/performance.ts` tracks build times
- **CI/CD ready**: GitHub Actions workflow optimized for Bun

## 🔍 Performance Monitoring

The development setup includes performance monitoring:

```bash
# Start performance monitoring
bun run scripts/performance.ts

# View build metrics
cat build-metrics.json
```

## 🏗️ Project Structure

```
mpc-wallet/
├── bunfig.toml                 # Bun configuration
├── bun.config.ts              # Build configuration
├── bun.test.config.ts         # Test configuration
├── test-setup.ts              # Global test setup
├── scripts/
│   ├── dev.ts                 # Development script with auto-rebuild
│   └── performance.ts         # Performance monitoring
├── src/                       # Source code
├── pkg/                       # Generated WASM packages
└── target/                    # Rust build artifacts
```

## 🚀 WASM Development

The setup automatically handles WASM development:

1. **Auto-rebuild**: Changes to `.rs` files trigger WASM rebuilds
2. **Fast builds**: Development builds use `--dev` for faster compilation
3. **Hot reloading**: WXT automatically reloads when WASM changes

## 🧪 Test Integration

FROST signing tests run with real WASM:

```bash
# Run WebRTC/FROST tests specifically
bun run test:webrtc

# Run with verbose output to see WASM interactions
bun test --verbose
```

## 📊 Coverage Reports

Test coverage is automatically generated:

- **Terminal**: Displays in terminal after running tests
- **HTML**: `coverage/index.html` for detailed browser viewing
- **LCOV**: `coverage/lcov.info` for CI/CD integration

## 🔄 CI/CD Integration

The project includes a GitHub Actions workflow (`.github/workflows/ci.yml`) optimized for Bun:

- Caches Bun dependencies and Cargo artifacts
- Runs tests with coverage
- Builds for multiple browsers
- Uploads coverage reports to Codecov

## 🐛 Troubleshooting

### WASM Issues
If WASM initialization fails:
```bash
# Rebuild WASM modules
bun run build:wasm

# Clean and rebuild everything
bun run fresh
```

### Performance Issues
Monitor build performance:
```bash
# Check build times
bun run scripts/performance.ts

# Clean build cache
bun run clean
```

### Test Issues
```bash
# Run tests with verbose logging
bun test --verbose

# Check WASM initialization in tests
bun test src/entrypoints/offscreen/webrtc.test.ts
```

## 📈 Performance Benefits

Compared to npm/yarn + Jest/Vitest:

- **Install time**: ~3x faster package installation
- **Test execution**: ~2x faster test runs
- **Build times**: Optimized WASM rebuilding
- **Memory usage**: Lower memory footprint during development

## 🔧 Configuration Details

### Bun Configuration (`bunfig.toml`)
- Optimized install settings
- Test timeout and coverage configuration
- Performance optimizations

### TypeScript Integration
- Full TypeScript support with `bun-types`
- Type checking with `bun run type-check`
- Svelte component support

### WASM Integration
- Automatic loading in tests via `test-setup.ts`
- Hot reloading for Rust source changes
- Development and production build modes

---

For more information about Bun, visit [bun.sh](https://bun.sh/).
