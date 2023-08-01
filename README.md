# Uniswap V2 Style DEX Pallet

This DEX (Decentralized Exchange) Pallet is based on the Uniswap V2 design and allows users to trustlessly exchange tokens. It is implemented as a Substrate runtime pallet.

The DEX includes functionality to incentivize users to create liquidity pools and also provides a price oracle based on the existing liquidity pools.

## API

The pallet exposes the following API. All methods are called on an instance of the DexInterface:

### [`setup_account`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/335e76986a7fffdde5eac6a2cfc4dd37415126db/pallets/interface/src/lib.rs#L32)
**Description:** A helper function to setup an account so it can hold any number of assets.

#### Signature:
```rust
fn setup_account(who: Self::AccountId) -> DispatchResult;
```


### [`mint_asset`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/335e76986a7fffdde5eac6a2cfc4dd37415126db/pallets/interface/src/lib.rs#L35)
** Description: ** Mints a given amount of a specific asset for a user.

#### Signature:
```rust
fn mint_asset(who: Self::AccountId, token_id: Self::AssetId, amount: Self::AssetBalance) -> DispatchResult;
```


### [`asset_balance`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/335e76986a7fffdde5eac6a2cfc4dd37415126db/pallets/interface/src/lib.rs#L42)
**Description:** Retrieves a user's balance for a specific asset.

#### Signature:
```rust
fn asset_balance(who: Self::AccountId, token_id: Self::AssetId) -> Self::AssetBalance;
```


### [`Swap_fee`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/335e76986a7fffdde5eac6a2cfc4dd37415126db/pallets/interface/src/lib.rs#L45)
**Description:** Returns the number of basis points (1/100) used for swap fees.

#### Signature:
```rust
fn swap_fee() -> u16;
```


### [`lp_id`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/335e76986a7fffdde5eac6a2cfc4dd37415126db/pallets/interface/src/lib.rs#L48)
**Description:** Get the LP Token ID that will be generated by creating a pool of asset_a and asset_b.

#### Signature:
```rust
fn lp_id(asset_a: Self::AssetId, asset_b: Self::AssetId) -> Self::AssetId;
```


[`add_liquidity`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/335e76986a7fffdde5eac6a2cfc4dd37415126db/pallets/interface/src/lib.rs#L54)
**Description:** Adds liquidity to a pool on behalf of the user. If necessary, this will create the pool. LP tokens are minted to the caller.

#### Signature:
```rust
fn add_liquidity(
who: Self::AccountId,
asset_a: Self::AssetId,
asset_b: Self::AssetId,
amount_a: Self::AssetBalance,
amount_b: Self::AssetBalance,
) -> DispatchResult;
```


[`remove_liquidity`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/335e76986a7fffdde5eac6a2cfc4dd37415126db/pallets/interface/src/lib.rs#L66)
**Description:** Removes liquidity from a pool on behalf of the user. The token_amount represents the amount of LP tokens to be burned in exchange for underlying assets.

#### Signature:
```rust
fn remove_liquidity(
who: Self::AccountId,
asset_a: Self::AssetId,
asset_b: Self::AssetId,
token_amount: Self::AssetBalance,
) -> DispatchResult;
```


### [`Swap_exact_in_for_out`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/335e76986a7fffdde5eac6a2cfc4dd37415126db/pallets/interface/src/lib.rs#L77)
**Description:** Swaps an exact amount of asset_in for a minimum amount of asset_out on behalf of who. The swap fee is deducted from the out amount.

#### Signature:
```rust
fn swap_exact_in_for_out(
who: Self::AccountId,
asset_in: Self::AssetId,
asset_out: Self::AssetId,
exact_in: Self::AssetBalance,
min_out: Self::AssetBalance,
) -> DispatchResult;
```


### [`Swap_in_for_exact_out`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/335e76986a7fffdde5eac6a2cfc4dd37415126db/pallets/interface/src/lib.rs#L89)
**Description:** Swaps a max amount of asset_in for an exact amount of asset_out on behalf of who. The swap fee is added to the in amount.

#### Signature:
```rust
fn swap_in_for_exact_out(
origin: Self::AccountId,
asset_in: Self::AssetId,
asset_out: Self::AssetId,
max_in: Self::AssetBalance,
exact_out: Self::AssetBalance,
) -> DispatchResult;
```



