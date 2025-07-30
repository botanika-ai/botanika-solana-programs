# Botanika Solana Programs

This repository contains the Solana on-chain programs used by the Botanika protocol. These programs handle staking, reward distribution, governance configuration, and more.

## 📦 Included Programs

| Program     | Description                            |
|-------------|----------------------------------------|
| `staking`   | Initialize staking positions           |
| `rewards`   | Distribute BONSAI token rewards        |
| `governance`| Set system-wide parameters             |

Each program is implemented using [Anchor](https://www.anchor-lang.com/) and follows Solana best practices.

---

## 📁 Project Structure
```
botanika-solona-programs
├── Anchor.toml
├── Cargo.toml
├── programs/
│ ├── staking/
│ ├── rewards/
│ └── governance/
└── tests/
```

---

## 🚀 Getting Started

### 1. Prerequisites

- [Rust](https://rustup.rs)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor CLI](https://book.anchor-lang.com/getting_started/installation.html)

```bash
cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked
```

### 2. Build the Programs
```bash
anchor build
```

### 3. Run Tests
```bash
anchor test
```

## Test Coverage
All unit tests are written using [solana-program-test](https://docs.rs/solana-program-test/latest/solana_program_test/).  
They are located in the top-level `/tests` directory and cover basic interactions for each on-chain program:

- `tests/staking.rs` – Verifies staking initialization
- `tests/rewards.rs` – Verifies BONSAI reward distribution logic
- `tests/governance.rs` – Verifies parameter-setting via governance interface

You can run all tests with:

```bash
anchor test
```

## License
This project is licensed under the MIT License.
See LICENSE for full details.
---

