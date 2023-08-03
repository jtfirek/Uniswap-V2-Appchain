# Uniswap V2 Style DEX Pallet

This DEX (Decentralized Exchange) Pallet is based on the Uniswap V2 design and allows users to trustlessly exchange tokens. It is implemented as a Substrate runtime pallet.

The DEX includes functionality to incentivize users to create liquidity pools and also provides a price oracle based on the existing liquidity pools. 

## How it works 

### Swaps
When a user calls `swap_exact_in_for_out` DOT for KSM when a liquidity pool already exists, they first pay a 1 percent fee in the input token Dot in this case. The trading fees from each swap are added to the pool for the relevant pair of tokens. This means that, as a liquidity provider, when you withdraw your liquidity, you receive a portion of the transaction fees based on your share of the pool. 

Now after paying the fee, how much will the user receive? This is the determined by the constant product formula:
```
X * Y = K
```
This means that the product K of the reserves of the two tokens in the pool must remain constant. This is calculated by balancing the following equation `X * Y = newX * newY`.

### Price oracle 
The current exchange rate between tokenA and tokenB not including fees is computed by dividing the reserve assets of tokenA with reserve assets of tokenB. This is done in the `price_oracle` function.
```
Price = reserveA / reserveB
```

### LP token math
This is the math that is used to ensure a fair distribution of liquidity provider (LP) tokens based on the amount of liquidity provided. 
#### Creating a pool
When a user creates a creates a new pool. The amount of LP tokens they receive is calculated by the following formula:
```
lp = sqrt(a*b)
```
Where `a` is the amount of tokenA and `b` is the amount of tokenB.

#### Adding liquidity
When a user adds liquidity to an existing pool, the amount of LP tokens they receive is calculated by the following formula:
```
lp = sqrt((A+a)*(B+b)) - sqrt(A*B)
```
Where `A` is the amount of tokenA in the pool, `B` is the amount of tokenB in the pool, `a` is the amount of tokenA the user is adding, and `b` is the amount of tokenB the user is adding.

#### Removing liquidity
When a user removes liquidity from an existing pool, the amount of tokens a, b, AF, and BF (AF and BF represent the fees collected for each token in the pool) they receive is calculated by the following formula
```
amountA = (amountLP / totalLP) * totalA
amountB = (amountLP / totalLP) * totalB
amountAF = (amountLP / totalLP) * totalAF
amountBF = (amountLP / totalLP) * totalBF
```

## API

The pallet exposes the following API. All methods are called on an instance of the DexInterface:


#### [`price_oracle`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/335e76986a7fffdde5eac6a2cfc4dd37415126db/pallets/interface/src/lib.rs#L54)
**Description:** Returns the price of asset_out in terms of asset_in. The price is expressed as a ratio of asset_in to asset_out.

#### Signature:
```rust
fn price_oracle(
asset_in: Self::AssetId,
asset_out: Self::AssetId,
) -> DispatchResult;
```
<br>

#### [`add_liquidity`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/335e76986a7fffdde5eac6a2cfc4dd37415126db/pallets/interface/src/lib.rs#L54)
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
<br>

#### [`remove_liquidity`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/335e76986a7fffdde5eac6a2cfc4dd37415126db/pallets/interface/src/lib.rs#L66)
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
<br>

#### [`Swap_exact_in_for_out`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/335e76986a7fffdde5eac6a2cfc4dd37415126db/pallets/interface/src/lib.rs#L77)
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
<br>

#### [`Swap_in_for_exact_out`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/335e76986a7fffdde5eac6a2cfc4dd37415126db/pallets/interface/src/lib.rs#L89)
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
