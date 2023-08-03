#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet_prelude::*, 
	traits::fungibles,
	traits::fungibles::{
		Create, Mutate
	},
	sp_runtime::{
		ArithmeticError,
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

	// gives us the AssetId type represents the identifier of a fungible asset within a runtime.
	pub type AssetIdOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<
	<T as frame_system::Config>::AccountId,
	>>::AssetId;


	pub type AssetBalanceOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	// Stores Pool pairs in sorted order 
	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo)]
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
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Pool<T: Config> {
    	// stores the asset ids  and balances of the two assets in the pool in sorted order
		pub pool_pair: PoolPair<T>,
		
    	// Total supply of the LP tokens
    	pub lp_supply: AssetBalanceOf<T>,
	}
	impl<T: Config> Pool<T> {
		pub fn new(
			pool_pair: PoolPair<T>,
			lp_supply: AssetBalanceOf<T>,
		) -> Self {
			Self {
				pool_pair,
				lp_supply,
			}
		}

		// pub fn update(
		// 	additional_pool_pair: PoolPair<T>,
		// 	additional_lp_supply: AssetBalanceOf<T>,
		// ) -> Self {
		// 	let new_lp_supply = self.lp_supply.checked_add(&additional_lp_supply).ok_or(ArithmeticError::Overflow)?;

		// 	// Self {
				
				
		// 	// }
		// }
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
	use frame_support::traits::fungibles::Mutate;

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
			let add_amounts = PoolPair::<T>::new(asset_a, amount_a, asset_b, amount_b)?;
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
					Ok(())
				},
				Some(existing_pool) => {
					// Calculate the amount of LP tokens to mint
					let lp_amount = Self::calculate_lp(&add_amounts, Some(&existing_pool))?;

					// mint to user
					T::Fungibles::mint_into(cur_lp_id.clone(), &who, lp_amount)?;

					// update the pool
					
					Ok(())
				},
			}
			
		}
	}
}



impl<T: Config> Pallet<T> {

	// lp(a, b) == lp(b, a)
	// idea of logic would be:
	// 1. the user can submit in any order
	// 2. we "sort" the assets
	// 3. then create the id
	// you need to figure out what you want to return if anything
	// fn create_asset_pair(
	// 	asset_a: AssetIdOf<T>,
	// 	asset_b: AssetIdOf<T>,
	// ) -> Result<PoolPair<T>, DispatchError> {
	// 	ensure!(asset_a != asset_b, "cant use the same id twice");
	// 	return if asset_a.encode() > asset_b.encode() {
	// 		Ok(PoolPair::new(asset_b, asset_a))
	// 	} else {
	// 		Ok(PoolPair::new(asset_a, asset_b))
	// 	}
	// }

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
		let mut bytes;
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

	// This function assumes asset_a and asset_b have already been sorted
	// fn generate_account_from_asset_id_pair(
	// 	asset_a: AssetIdOf<T>,
	// 	asset_b: AssetIdOf<T>,
	// ) -> T::AccountId {
	// 	let bytes = T::Hashing::hash(&(asset_a, asset_b).encode());
	// 	let generated_account = T::AccountId::decode(&mut TrailingZeroInput::new(&bytes.encode()))
	// 		.expect("in our PBA exam, we assume all bytes can be turned into some account id");
	// 	generated_account
	// }

	// you can do the same thing for an asset id if needed...
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
