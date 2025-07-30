# Tests for Botanika Programs

All tests use [solana-program-test](https://docs.rs/solana-program-test/latest/solana_program_test/).

## Test Coverage

| File | Covered Program | What it tests |
|------|------------------|----------------|
| `staking.rs` | `staking` | Initialization of the staking instruction |
| `rewards.rs` | `rewards` | Distribution logic with dummy call |
| `governance.rs` | `governance` | Set parameter value through governance |

## Run Tests

```bash
anchor test
```

You may also run specific files:
```bash
cargo test-bpf --test staking
```