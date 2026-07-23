# Contributing to `pitu` 📷⚡

Thank you for your interest in improving `pitu`! We welcome contributions from developers of all skill levels.

---

## 🛠️ Development Setup

1. **Fork & Clone repository**:
   ```bash
   git clone https://github.com/pitu-cli/pitu.git
   cd pitu
   ```

2. **Run tests**:
   ```bash
   cargo test
   ```

3. **Check formatting & lints**:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   ```

---

## 🌟 Guidelines

- Keep code modular (operations under `src/operations/`, UI under `src/ui/`).
- Add unit tests for new operations under `tests/`.
- Maintain clean, descriptive commit messages.

Thank you for making `pitu` better for everyone!
