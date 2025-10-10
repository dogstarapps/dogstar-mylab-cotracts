
---

# Stellar Smart Contract Project

This project contains two Stellar smart contracts written in Rust: a token contract and an NFT contract. These contracts can be built, optimized, and deployed on the Stellar testnet.

## Project Structure

```
.
├── contracts
│   ├── token
│   │   └── src
│   │       └── lib.rs
│   ├── nft
│   │   └── src
│   │       └── lib.rs
├── target
│   └── wasm32-unknown-unknown
│       └── release
│           ├── soroban_token_contract.wasm
│           ├── soroban_token_contract.optimized.wasm
│           ├── soroban_nft_contract.wasm
│           ├── soroban_nft_contract.optimized.wasm
└── README.md
```

## Prerequisites

Ensure you have the following installed:

- Rust: [Install Rust](https://www.rust-lang.org/tools/install)
- Stellar CLI: [Install Stellar CLI](https://www.stellar.org/developers/tools-and-sdks/cli.html)

Note: Check Rust Version and Install target
        
        # Check Rust version
        rustc --version

        # If Rust 1.85.0 or newer, add the new target
        rustup target add wasm32v1-none

        # If older Rust version, ensure you have the standard target
        rustup target add wasm32-unknown-unknown
        
        # For Rust 1.85.0+
        cargo build --target wasm32v1-none --release --package soroban-nft-contract

        # For older Rust versions  
        cargo build --target wasm32-unknown-unknown --release --package soroban-nft-contract

## Building the Contracts

To build the contracts, navigate to the project root directory and run:

```sh
stellar contract build
```

This command will generate the WebAssembly (WASM) files for both contracts:

- `target/wasm32-unknown-unknown/release/soroban_token_contract.wasm`
- `target/wasm32-unknown-unknown/release/soroban_nft_contract.wasm`

## Optimizing the Contracts

Optimize the generated WASM files to reduce their size and improve performance:

```sh
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/soroban_token_contract.wasm
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/soroban_nft_contract.wasm
```

This will create the optimized WASM files:

- `target/wasm32-unknown-unknown/release/soroban_token_contract.optimized.wasm`
- `target/wasm32-unknown-unknown/release/soroban_nft_contract.optimized.wasm`

## Deploying the Contracts

Deploy the optimized WASM files to the Stellar testnet using the following command:

```sh
stellar contract deploy --wasm target/wasm32-unknown-unknown/release/soroban_token_contract.optimized.wasm --source alice --network testnet
stellar contract deploy --wasm target/wasm32-unknown-unknown/release/soroban_nft_contract.optimized.wasm --source alice --network testnet
```

Replace `alice` with the actual source account name.

##Initialize Contract

        ``{
            pub xtar_token: Address, //e.g. XTAR Contract Address
            pub oracle_contract_id: Address, //e.g. https://reflector.network/docs
            pub haw_ai_pot: Address, //e.g. Account Address - contract admin address
            pub withdrawable_percentage: u32,
            pub burnable_percentage: u32,
            pub haw_ai_percentage: u32,
            pub terry_per_power: i128,
            pub stake_periods: Vec<u32>,
            pub stake_interest_percentages: Vec<u32>,
            pub power_action_fee: u32,
            pub burn_receive_percentage: u32,
            pub terry_per_deck: i128,
            pub terry_per_fight: i128,
            pub terry_per_lending: i128,
            pub terry_per_stake: i128,
            pub apy_alpha: u32,
            pub power_to_usdc_rate: i128, // e.g., 1000 for 0.10 USDC per POWER (1000/10000 = 0.10)
            pub dogstar_fee_percentage: u32, // Basis points (e.g., 500 = 5%)
            pub dogstar_address: Address, // e.g. Used to recover the fee.
        }``


## Using the Contracts

Once deployed, you can interact with the contracts on the Stellar testnet. Refer to the Stellar documentation for details on how to call contract functions and handle transactions. After deploy this smar contract, You must have to initialize Contract.


This README file provides a comprehensive guide on building, optimizing, and deploying your Stellar smart contracts.