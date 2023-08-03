#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet_prelude::*, 
	traits::fungibles,
	sp_runtime::{
		ArithmeticError,
		Percent,
		traits::{
			CheckedAdd, 
			CheckedMul, 
			IntegerSquareRoot, 
			CheckedSub,
			AccountIdConversion,
			Hash,
			TrailingZeroInput
		}
	}
};

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::{fungible, fungibles::{self, Create}},
	};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type to access the Balances Pallet.
		type NativeBalance: fungible::Inspect<Self::AccountId>
			+ fungible::Mutate<Self::AccountId>
			+ fungible::hold::Inspect<Self::AccountId>
			+ fungible::hold::Mutate<Self::AccountId>
			+ fungible::freeze::Inspect<Self::AccountId>
			+ fungible::freeze::Mutate<Self::AccountId>;

		/// Type to access the Assets Pallet.
		type Fungibles: fungibles::Inspect<Self::AccountId> 
			+ fungibles::Mutate<Self::AccountId>
			+ fungibles::Create<Self::AccountId>;

		/// The DEXs pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<frame_support::PalletId>;
	}

	/// Types defined here

	// gives us access to the asset id and balance types of the fungibles 
	pub type AssetIdOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<
	<T as frame_system::Config>::AccountId,
	>>::AssetId;
	pub type AssetBalanceOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	// //gives us access to the asset id and balance types of the native currency
	// pub type NativeAssetBalanceOf<T> = <<T as Config>::NativeBalance as fungible::Inspect<
	// 	<T as frame_system::Config>::AccountId,
	// >>::Balance;

	
	// Stores Pool pairs in sorted order 
	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone)]
	#[scale_info(skip_type_params(T))]
	pub struct PoolPair<T: Config> {
		pub asset_1: AssetIdOf<T>,
		pub amount_1: AssetBalanceOf<T>,
		pub asset_2: AssetIdOf<T>,
		pub amount_2: AssetBalanceOf<T>,
	}
	impl<T: Config> PoolPair<T> {
		pub fn new(
			asset_a: AssetIdOf<T>, 
			amount_a: AssetBalanceOf<T>,
			asset_b: AssetIdOf<T>,
			amount_b: AssetBalanceOf<T>,
		) -> Result<Self, &'static str> {
			if asset_a == asset_b {
				return Err("cant use the same id twice");
			}
			if asset_a.encode() > asset_b.encode() {
				Ok(Self {
					asset_1: asset_b,
					amount_1: amount_b,
					asset_2: asset_a,
					amount_2: amount_a,
				})
			} else {
				Ok(Self {
					asset_1: asset_a,
					amount_1: amount_a,
					asset_2: asset_b,
					amount_2: amount_b,
				})
			}
		}

		pub fn default(
			asset_a: AssetIdOf<T>,
			asset_b: AssetIdOf<T>,
		) -> Result<Self, &'static str> {
			Self::new(
				asset_a, 
				AssetBalanceOf::<T>::default(), 
				asset_b, 
				AssetBalanceOf::<T>::default()
			)
		}
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Pool<T: Config> {
    	// stores the asset ids and balances of the two assets in the pool in sorted order
		pub pool_pair: PoolPair<T>,

		// stores the asset ids and balances of the fees collected for each asset
		pub fee_pair: PoolPair<T>,
		
    	// Total supply of the LP tokens
    	pub lp_supply: AssetBalanceOf<T>,
	}
	impl<T: Config> Pool<T> {
		pub fn new(
			pool_pair: PoolPair<T>,
			lp_supply: AssetBalanceOf<T>,
		) -> Self {
			Self {
				pool_pair: pool_pair.clone(),
				fee_pair: PoolPair::<T>::default(pool_pair.asset_1, pool_pair.asset_2).unwrap(),
				lp_supply,
			}
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn something)]
	// The pools are stored by a key that is the asset id of the LP token
	pub type PoolMap<T> =
		StorageMap<_, Blake2_128Concat, AssetIdOf<T>, Pool<T>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored { something: u32, who: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}
	use frame_support::sp_runtime::traits::{
		CheckedMul,
		CheckedAdd,
	};
	use crate::ArithmeticError;
	use frame_support::traits::fungibles::{
		Mutate,
		Inspect,
	};
	use frame_support::traits::tokens::Preservation::*;

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		

		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			asset_a: AssetIdOf<T>,
			asset_b: AssetIdOf<T>,
			amount_a: AssetBalanceOf<T>,
			amount_b: AssetBalanceOf<T>,
			) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let cur_lp_id = Self::get_lp_id(&asset_a, &asset_b)?;
			let add_amounts = PoolPair::<T>::new(asset_a.clone(), amount_a, asset_b.clone(), amount_b)?;
			match <PoolMap<T>>::get(&cur_lp_id) {
				// New Pool
				None => {
					// Calculate the amount of LP tokens to mint
					let lp_amount = Self::calculate_lp(&add_amounts, None)?;

					// Create the LP token and mint to user
					T::Fungibles::create(cur_lp_id.clone(), Self::account_id() , false, lp_amount)?;
					T::Fungibles::mint_into(cur_lp_id.clone(), &who, lp_amount)?;
					
					// Create the pool and store it
					let new_pool = Pool::<T>::new(add_amounts, lp_amount);
					<PoolMap<T>>::insert(&cur_lp_id, new_pool);
				},
				Some(existing_pool) => {
					// Calculate the amount of LP tokens to mint
					let lp_amount = Self::calculate_lp(&add_amounts, Some(&existing_pool))?;

					// mint to user
					T::Fungibles::mint_into(cur_lp_id.clone(), &who, lp_amount)?;

					// add to the pool
					Self::increase_pool(&add_amounts, &cur_lp_id)?;
				},
			}

			// send the assets to the dex pallet
			T::Fungibles::transfer(asset_a, &who, &Self::account_id(), amount_a, Expendable)?;
			T::Fungibles::transfer(asset_b, &who, &Self::account_id(), amount_b, Expendable)?;
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			asset_a: AssetIdOf<T>,
			asset_b: AssetIdOf<T>,
			token_amount: AssetBalanceOf<T>,
			) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// get the LP token id
			let cur_lp_id = Self::get_lp_id(&asset_a, &asset_b)?;

			// ensure caller has enough LP tokens
			ensure!(T::Fungibles::balance(cur_lp_id.clone(), &who) >= token_amount, "not enough LP tokens"); // not sure if this is an ok way to propagate the error

			// get the pool
			let mut pool = <PoolMap<T>>::get(&cur_lp_id).ok_or(Error::<T>::NoneValue)?;

			// calculate the amount of each asset to return to the user
			let amount_1 = pool.pool_pair.amount_1.checked_mul(&token_amount).ok_or(ArithmeticError::Overflow)? / pool.lp_supply;
			let amount_2 = pool.pool_pair.amount_2.checked_mul(&token_amount).ok_or(ArithmeticError::Overflow)? / pool.lp_supply;
			let amount_1_reward = pool.fee_pair.amount_1.checked_mul(&token_amount).ok_or(ArithmeticError::Overflow)? / pool.lp_supply;
			let amount_2_reward = pool.fee_pair.amount_2.checked_mul(&token_amount).ok_or(ArithmeticError::Overflow)? / pool.lp_supply;
			
			

			// burn the LP tokens
			

			
			// burn the LP tokens 
			// T::Fungibles::burn_from(cur_lp_id.clone(), &who, token_amount,)?;
			// T::Fungibles::mutate(cur_lp_id.clone(), &Self::account_id(), |balance| *balance -= token_amount)?;

			// return assets to the user 
			let total_1 = amount_1.checked_add(&amount_1_reward).ok_or(ArithmeticError::Overflow)?;
			let total_2 = amount_2.checked_add(&amount_2_reward).ok_or(ArithmeticError::Overflow)?;
			T::Fungibles::transfer(asset_a, &Self::account_id(), &who, total_1, Protect)?;
			T::Fungibles::transfer(asset_b, &Self::account_id(), &who, total_2, Protect)?;
			
			todo!()
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn swap_exact_in_for_out(
			origin: OriginFor<T>,
			asset_in: AssetIdOf<T>,
			asset_out: AssetIdOf<T>,
			exact_in: AssetBalanceOf<T>,
			min_out: AssetBalanceOf<T>,
			) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// get the LP token id
			let cur_lp_id = Self::get_lp_id(&asset_in, &asset_out)?;
			// get the pool
			let pool = <PoolMap<T>>::get(&cur_lp_id).ok_or(Error::<T>::NoneValue)?;
			// calculate the potential fee and insure that user can pay

			
			// ensure!(amount_out >= min_out, "slippage too high");
			Ok(())
		}
	}
}



impl<T: Config> Pallet<T> {
	// helper function to calculate the amount of LP tokens to mint and issue them 
	// lp = sqrt((A+a)*(B+b)) - sqrt(A*B) 
	// Where A and B are the current amount of tokenA and tokenB in the pool. a and b are the amounts of tokenA and tokenB that the user is adding to the pool
	// if new pool, the equation simplifies lp = sqrt(a*b)
	fn calculate_lp(
		new_pair: &PoolPair<T>,
		pool: Option<&Pool<T>>,
	) -> Result<AssetBalanceOf<T>, DispatchError> {
		match pool {
			None => {
				// new pool sqrt(a*b)
				return Ok(IntegerSquareRoot::integer_sqrt(&new_pair.amount_1.checked_mul(&new_pair.amount_2).ok_or(ArithmeticError::Overflow)?));
			},
			Some(pool) => {
				// (A + a)
				let total_1 = pool.pool_pair.amount_1.checked_add(&new_pair.amount_1).ok_or(ArithmeticError::Overflow)?;
				// (B + b)
				let total_2 = pool.pool_pair.amount_2.checked_add(&new_pair.amount_2).ok_or(ArithmeticError::Overflow)?;
				// (A + a) * (B + b)
				let total_1_2 = total_1.checked_mul(&total_2).ok_or(ArithmeticError::Overflow)?;
				// sqrt((A + a) * (B + b))
				let sqrt_1 = IntegerSquareRoot::integer_sqrt(&total_1_2);
				// sqrt(A * B)
				let sqrt_2 = IntegerSquareRoot::integer_sqrt(&pool.pool_pair.amount_1.checked_mul(&pool.pool_pair.amount_2).ok_or(ArithmeticError::Overflow)?);
				// sqrt((A + a) * (B + b)) - sqrt(A * B)
				let lp = sqrt_1.checked_sub(&sqrt_2).ok_or(ArithmeticError::Underflow)?;
				return Ok(lp);
			}
		}
	}

	/// The account ID of the dex pallet. It can be used as an admin for new assets created.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	// This function assumes asset_a and asset_b have already been sorted
	pub fn get_lp_id(
		asset_a: &AssetIdOf<T>,
		asset_b: &AssetIdOf<T>,
	) -> Result<AssetIdOf<T>, DispatchError> {
		let bytes;
		ensure!(asset_a != asset_b, "cant use the same id twice");
		if asset_a.encode() > asset_b.encode() {
			bytes = T::Hashing::hash(&(asset_b, asset_a).encode());
		} else {
			bytes = T::Hashing::hash(&(asset_a, asset_b).encode());
		}
		let generated_account = AssetIdOf::<T>::decode(&mut TrailingZeroInput::new(&bytes.encode()))
			.expect("in our PBA exam, we assume all bytes can be ID");
		Ok(generated_account)
	}

	// adds liquidity to an existing pool
	pub fn increase_pool(
		new_pair: &PoolPair<T>,
		pool_id: &AssetIdOf<T>,
	) -> Result<(), DispatchError> {
		let mut pool = <PoolMap<T>>::get(pool_id).ok_or(Error::<T>::NoneValue)?;
		pool.pool_pair.amount_1 = pool.pool_pair.amount_1.checked_add(&new_pair.amount_1).ok_or(ArithmeticError::Overflow)?;
		pool.pool_pair.amount_2 = pool.pool_pair.amount_2.checked_add(&new_pair.amount_2).ok_or(ArithmeticError::Overflow)?;
		<PoolMap<T>>::insert(pool_id, pool);
		Ok(())
	}

	// calculates the amount of fees to be collected
	// currently we have a hard coded 3% fee
	pub fn calculate_fees(
		amount_in: &AssetBalanceOf<T>,
		pool: &Pool<T>,
	) -> Result<AssetBalanceOf<T>, DispatchError> {
		let ten_percent = Percent::from_rational(3u32, 100u32);
		todo!()
	}

	
}


// Look at `../interface/` to better understand this API.
impl<T: Config> pba_interface::DexInterface for Pallet<T> {
	type AccountId = T::AccountId;
	type AssetId = <T::Fungibles as fungibles::Inspect<Self::AccountId>>::AssetId;
	type AssetBalance = <T::Fungibles as fungibles::Inspect<Self::AccountId>>::Balance;

	fn setup_account(_who: Self::AccountId) -> DispatchResult {
		unimplemented!()
	}

	fn mint_asset(
		_who: Self::AccountId,
		_token_id: Self::AssetId,
		_amount: Self::AssetBalance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn asset_balance(_who: Self::AccountId, _token_id: Self::AssetId) -> Self::AssetBalance {
		unimplemented!()
	}

	fn swap_fee() -> u16 {
		unimplemented!()
	}

	fn lp_id(_asset_a: Self::AssetId, _asset_b: Self::AssetId) -> Self::AssetId {
		unimplemented!()
	}

	fn add_liquidity(
		_who: Self::AccountId,
		_asset_a: Self::AssetId,
		_asset_b: Self::AssetId,
		_amount_a: Self::AssetBalance,
		_amount_b: Self::AssetBalance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn remove_liquidity(
		_who: Self::AccountId,
		_asset_a: Self::AssetId,
		_asset_b: Self::AssetId,
		_token_amount: Self::AssetBalance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn swap_exact_in_for_out(
		_who: Self::AccountId,
		_asset_in: Self::AssetId,
		_asset_out: Self::AssetId,
		_exact_in: Self::AssetBalance,
		_min_out: Self::AssetBalance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn swap_in_for_exact_out(
		_origin: Self::AccountId,
		_asset_in: Self::AssetId,
		_asset_out: Self::AssetId,
		_max_in: Self::AssetBalance,
		_exact_out: Self::AssetBalance,
	) -> DispatchResult {
		unimplemented!()
	}
}
