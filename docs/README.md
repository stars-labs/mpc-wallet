# Documentation

## Overview

Welcome to the MPC Wallet documentation. This directory contains comprehensive technical documentation, guides, and references for all aspects of the MPC Wallet ecosystem.

## Quick Links

- [Technical Documentation](../MPC_WALLET_TECHNICAL_DOCUMENTATION.md) - Complete technical reference
- [Main README](../README.md) - Project overview and quick start
- [Contributing Guide](../CONTRIBUTING.md) - How to contribute to the project

## Documentation Structure

### Core Documentation

#### [Architecture](architecture/)
System design, architectural patterns, and technical decisions
- System overview and component architecture
- Data flow and state management
- Network architecture and P2P design
- Design patterns and best practices

#### [Security](security/)
Security model, threat analysis, and best practices
- Threat model and security controls
- Cryptographic design and implementation
- Key management and data protection
- Incident response procedures

#### [API Reference](api/)
Complete API documentation for all components
- FROST protocol APIs
- Application APIs (Browser, CLI, Desktop)
- Network protocols and message formats
- Integration guides

#### [Development](development/)
Development setup, workflows, and guidelines
- Environment setup and prerequisites
- Building from source
- Testing strategies
- Debugging and troubleshooting

#### [Deployment](deployment/)
Production deployment and operations
- Deployment strategies and options
- Infrastructure requirements
- Monitoring and observability
- Backup and recovery procedures

#### [Testing](testing/)
Testing documentation and strategies
- Unit and integration testing
- End-to-end test scenarios
- Performance testing
- Test coverage reports

### Application-Specific Documentation

#### [Browser Extension](../apps/browser-extension/docs/)
Chrome/Firefox extension documentation
- Extension architecture
- Development guide
- Publishing instructions

#### [Terminal UI](../apps/tui-node/docs/)
Terminal user interface documentation
- Command reference
- Configuration options
- Offline mode usage

#### [Desktop Application](../apps/native-node/docs/)
Native desktop application documentation
- GUI overview
- Platform-specific guides
- Distribution instructions

#### [Signal Server](../apps/signal-server/docs/)
WebRTC signaling server documentation
- Deployment options
- Scaling strategies
- Cloudflare Worker setup

## Getting Started

### For Users
1. Start with the [Main README](../README.md) for installation
2. Review the relevant application guide for your platform
3. Check the [API Reference](api/) for integration

### For Developers
1. Set up your [Development Environment](development/)
2. Review the [Architecture Documentation](architecture/)
3. Understand the [Security Model](security/)
4. Follow the [Contributing Guide](../CONTRIBUTING.md)

### For Operators
1. Review [Deployment Options](deployment/)
2. Understand [Security Requirements](security/)
3. Set up [Monitoring](deployment/monitoring.md)
4. Plan [Backup Strategy](deployment/backup.md)

## Documentation Standards

### Writing Style
- Clear and concise technical writing
- Code examples for all concepts
- Diagrams for complex architectures
- Step-by-step instructions for procedures

### Formatting
- Markdown for all documentation
- Consistent heading hierarchy
- Code blocks with syntax highlighting
- Tables for structured data

### Maintenance
- Regular reviews and updates
- Version-specific documentation
- Deprecation notices
- Migration guides

## Contributing to Documentation

We welcome documentation improvements! Please:

1. Follow the existing structure and style
2. Include code examples where appropriate
3. Update the navigation links
4. Test all commands and code samples
5. Submit a pull request with clear description

## Resources

### External Links
- [FROST Paper](https://eprint.iacr.org/2020/852)
- [WebRTC Specification](https://www.w3.org/TR/webrtc/)
- [Ethereum JSON-RPC](https://ethereum.org/en/developers/docs/apis/json-rpc/)

### Community
- [GitHub Discussions](https://github.com/your-org/mpc-wallet/discussions)
- [Discord Server](https://discord.gg/mpc-wallet)
- [Stack Overflow Tag](https://stackoverflow.com/questions/tagged/mpc-wallet)

## Documentation Versions

- **Current**: v2.0.0 (January 2025)
- **Previous**: v1.0.0 (archived)
- **Next**: v3.0.0 (development)

## License

This documentation is licensed under the same terms as the MPC Wallet project (MIT License).
   ```bash
   bun run build:wasm
   ```

4. **Start development server**
   ```bash
   bun run dev
   ```

5. **Load extension in Chrome**
   - Navigate to `chrome://extensions/`
   - Enable "Developer mode"
   - Click "Load unpacked"
   - Select the `dist` folder

### Available Scripts

- `bun run dev` - Start development server with hot reload
- `bun run build` - Build production extension
- `bun run build:wasm` - Build Rust/WASM modules
- `bun run test` - Run test suite
- `bun run check` - Run Svelte type checking

## 📖 Documentation

- [Development Guide](./docs/DEVELOPMENT.md)
- [Architecture Overview](./docs/architecture/)
- [Testing Guide](./docs/testing/TESTING.md)
- [CLI Keystore Format](./docs/cli-keystore-format.md)
- [DKG Test Guide](./docs/DKG_TEST_GUIDE.md)

## 🧪 Testing

```bash
# Run all tests
bun test

# Run specific test suites
bun test services/
bun test components/
bun test webrtc

# Run with coverage
bun test --coverage
```

## 🔐 Security

- **No Single Point of Failure**: Keys are distributed using MPC
- **Threshold Signatures**: Requires t-of-n participants to sign
- **Secure Communication**: WebRTC encrypted channels
- **Isolated Contexts**: Crypto operations in offscreen document

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

Apache-2.0 License - see LICENSE file for details

## 🙏 Acknowledgments

- Built with [WXT](https://wxt.dev/) - Next-gen Web Extension Framework
- FROST implementation using [frost-core](https://github.com/ZcashFoundation/frost)
- UI components with [Svelte](https://svelte.dev/)

## 📞 Support

- Create an issue for bug reports or feature requests
- Check [existing issues](https://github.com/your-repo/issues) before creating new ones
- See [CLAUDE.md](./CLAUDE.md) for AI assistant guidance