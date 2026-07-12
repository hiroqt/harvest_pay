# HarvestPay

Instant, trustless escrow for smallholder farmer-to-trader harvest sales, built on Stellar/Soroban.

## Problem

Rosa, a smallholder rice farmer in Nueva Ecija, Philippines, delivers her harvest to local traders who routinely delay payment by 2–4 weeks for "verification." That delay forces her to borrow at predatory interest rates just to cover inputs for the next planting cycle.

## Solution

Before delivery, the trader funds a Soroban escrow contract with USDC. The instant the trader confirms receipt of Rosa's harvest (e.g. by scanning a QR code at the trading post), the contract releases the funds directly to Rosa's wallet — no bank float, no paperwork delay, no counterparty risk on either side.

## Timeline (hackathon build)

- **Day 1:** Contract design, `create_escrow` / `confirm_delivery` / `cancel_escrow` implemented and unit tested
- **Day 2:** Testnet deployment, USDC trustline setup, CLI demo script
- **Day 3:** Minimal wallet UI (QR-based confirm flow) + live demo polish

## Stellar features used

- USDC transfers via a Stellar asset / anchor-issued token
- Soroban smart contract for escrow logic and state
- Trustlines (USDC on both farmer and buyer wallets)

## Vision and purpose

Rural agricultural trade in Southeast Asia still runs on paper receipts and delayed bank transfers, which quietly taxes the farmers who can least afford it. HarvestPay replaces that delay with programmable trust: funds move the moment goods change hands, verifiable by anyone, reversible by no one. The long-term vision is a network of local anchors letting farmers cash out directly to mobile money (e.g. GCash) without ever touching crypto UX.

## Prerequisites

- Rust (stable) with the `wasm32-unknown-unknown` target
- Soroban CLI v21+ (`cargo install --locked soroban-cli`)
- A funded Stellar testnet account (via [Friendbot](https://friendbot.stellar.org))

## Build

```bash
soroban contract build
```

## Test

```bash
cargo test
```

## Deploy to testnet

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/harvestpay.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```

## Sample CLI invocation

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <BUYER_SECRET_KEY> \
  --network testnet \
  -- \
  create_escrow \
  --buyer <BUYER_ADDRESS> \
  --farmer <FARMER_ADDRESS> \
  --token <USDC_TOKEN_ADDRESS> \
  --amount 5000000000
```

Then, once the harvest is delivered:

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <BUYER_SECRET_KEY> \
  --network testnet \
  -- \
  confirm_delivery \
  --escrow_id 0 \
  --buyer <BUYER_ADDRESS>
```

## License

MIT