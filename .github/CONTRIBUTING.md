# Contributing to Hadamard-DB

Thank you for choosing to contribute to the future of sub-linear quantum database engineering. To maintain our strict standards for ultra-speed, architectural leanness, and cryptographic safety, all contributors must strictly adhere to the professional lifecycle protocols detailed below.

## Monorepo Architecture Standards

Hadamard-DB is organized as a multi-language monorepo workspace structure:
- `/core`: Bare-metal asynchronous query core engine written exclusively in Rust.
- `/bindings/python`: Maturin-compiled PyO3 extension layer.
- `/bindings/typescript`: Neon-CLI compiled high-performance Node.js V8 extension layer.

### 1. Code Style and Optimization Constraints
- **Rust Core**: All code updates must pass strict formatting checks (`cargo fmt --check`) and static analysis without warnings (`cargo clippy -- -D warnings`). Memory allocation on the heap must be heavily restricted; utilize zero-copy string slices and atomic synchronization blocks.
- **Asynchronous Execution**: Never invoke blocking code inside async contexts. Always utilize Tokio concurrency workers or explicitly yield control back to runtime execution pools via `tokio::task::yield_now().await`.

### 2. Pull Request Protocol
1. Fork the workspace repository and spin up an isolated, tracking feature branch.
2. If changing the core system, write mandatory integration validation suites under `/tests`.
3. Verify multi-platform stability by ensuring local execution success across both Linux and macOS targets.
4. Open a formal Pull Request targeting the `main` development branch. 
5. Ensure your commit signatures are valid and your PR passes all semantic automated CI validation pipelines successfully.
