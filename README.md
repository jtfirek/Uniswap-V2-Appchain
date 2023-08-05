# Uniswap V2 Style DEX Pallet


This DEX (Decentralized Exchange) Pallet is based on the Uniswap V2 design and allows users to trustlessly exchange tokens. The DEX also allows for 
to attempt flash loans with the liquidity in the pools.The DEX includes functionality to incentivize users to create liquidity 
pools and also provides a price oracle based on the existing liquidity pools.
The swap and loan fee has a default value of 3 percent, but the pallet provides an the extrinsic `set_fee` to allow the fee to be changed. 
When configuring the runtime, the origin that has permission to set the fee must be set.

## How it works 

### Swaps
When a user calls `swap_exact_in_for_out` DOT for KSM when a liquidity pool already exists, they first pay a fee in the input token Dot in this case. The trading fees from each swap are added to the pool for the relevant pair of tokens. This means that, as a liquidity provider, when you withdraw your liquidity, you receive a portion of the transaction fees based on your share of the pool. 

Now after paying the fee, how much will the user receive? This is the determined by the constant product formula:
```
X * Y = K
```
This means that the product K of the reserves of the two tokens in the pool must remain constant. This is calculated by balancing the following equation `X * Y = newX * newY`.

### Price oracle 
The current exchange rate between an input toke and an output token is determined by the following formula:
```
ratio = output / input
```
Divides the amount of tokens in the output pool by the amount of tokens in the input pool and returns a percentage as the result

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
When a user removes liquidity from an existing pool, the amount of tokens a and b they receive is calculated by the following formula. 
```
amountA = (amountLP / totalLP) * totalA
amountB = (amountLP / totalLP) * totalB
```

## Extrinsic functions

Here are the extrinsics functions that are available to be called by users to interact with a runtime that implements the DEX pallet. To call an extrinsic, you need to create a transaction from an account with sufficient balance and broadcast it to the network. This action will trigger the associated extrinsic function.
You may construct these transactions directly through JavaScript or with the Polkadot.js API

#### [`add_liquidity`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/20fb7b87f5c3959e141663fff211a8bf28ce7208/pallets/dex/src/lib.rs#L229)
**Description:** Adds liquidity to a pool on behalf of the user. If necessary, this will create the pool. LP tokens are minted to the caller.  
**Call index**: 0
#### Signature:
```rust
fn add_liquidity(
origin: OriginFor<T>,
asset_a: AssetIdOf<T>,
asset_b: AssetIdOf<T>,
amount_a: AssetBalanceOf<T>,
amount_b: AssetBalanceOf<T>,
) -> DispatchResult
```
<br>

#### [`remove_liquidity`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/20fb7b87f5c3959e141663fff211a8bf28ce7208/pallets/dex/src/lib.rs#L267)
**Description:** Removes liquidity from a pool on behalf of the user. The token_amount represents the amount of LP tokens to be burned in exchange for underlying assets.  
**Call index**: 1
#### Signature:
```rust
fn remove_liquidity(
origin: OriginFor<T>,
asset_a: AssetIdOf<T>,
asset_b: AssetIdOf<T>,
token_amount: AssetBalanceOf<T>,
) -> DispatchResult
```
<br>

#### [`Swap_exact_in_for_out`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/20fb7b87f5c3959e141663fff211a8bf28ce7208/pallets/dex/src/lib.rs#L296)
**Description:** Swaps an exact amount of asset_in for a minimum amount of asset_out on behalf of who. The swap fee is deducted from the out amount.  
**Call index**: 2
#### Signature:
```rust
fn swap_exact_in_for_out(
origin: OriginFor<T>,
asset_in: AssetIdOf<T>,
asset_out: AssetIdOf<T>,
exact_in: AssetBalanceOf<T>,
min_out: AssetBalanceOf<T>,
) -> DispatchResult
```
<br>

#### [`Swap_in_for_exact_out`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/20fb7b87f5c3959e141663fff211a8bf28ce7208/pallets/dex/src/lib.rs#L325)
**Description:** Swaps a max amount of asset_in for an exact amount of asset_out on behalf of who. The swap fee is added to the in amount.  
**Call index**: 3
#### Signature:
```rust
swap_in_for_exact_out(
origin: OriginFor<T>,
asset_in: AssetIdOf<T>,
asset_out: AssetIdOf<T>,
max_in: AssetBalanceOf<T>,
exact_out: AssetBalanceOf<T>,
) -> DispatchResult 
```
<br>

#### [`price_oracle`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/20fb7b87f5c3959e141663fff211a8bf28ce7208/pallets/dex/src/lib.rs#L354)
**Description:** Emits an event with a percentage representing the current exchange rate between asset_in and asset_out.  
**Call index**: 4
#### Signature:
```rust
fn price_oracle(
_origin: OriginFor<T>,
asset_in: AssetIdOf<T>,
asset_out: AssetIdOf<T>,
) -> DispatchResult
```
<br>

#### [`set_fee`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/20fb7b87f5c3959e141663fff211a8bf28ce7208/pallets/dex/src/lib.rs#L375)
**Description:** Sets the fee for the DEX pallet. Each input represents 100 basis points. An input of 4 would yield a fee of 400 basis points or 4 percent. The fee can **Only** be set by the origin that is configured in the runtime.  
**Call index**: 5
#### Signature:
```rust
fn set_fee(
origin: OriginFor<T>,
new_fee: u16,
) -> DispatchResult
```
<br>

#### [`set_fee`](https://github.com/Polkadot-Blockchain-Academy/assigment-4-frame-jtfirek/blob/20fb7b87f5c3959e141663fff211a8bf28ce7208/pallets/dex/src/lib.rs#L375)
**Description:** Allows a user to attempt a flash loan. The user can dispatch any any system call to return the loan with the fee. The user must return the amount of tokens that they borrowed plus a fee of 0.3 percent.
**Call index**: 6
#### Signature:
```rust
fn flash_loan(
origin: OriginFor<T>,
asset_id: AssetIdOf<T>,
amount: AssetBalanceOf<T>,
call: Box<<T as frame_system::Config>::RuntimeCall>
) -> DispatchResult
```
<br>