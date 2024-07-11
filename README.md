
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

## Using the Contracts

Once deployed, you can interact with the contracts on the Stellar testnet. Refer to the Stellar documentation for details on how to call contract functions and handle transactions.


This README file provides a comprehensive guide on building, optimizing, and deploying your Stellar smart contracts.