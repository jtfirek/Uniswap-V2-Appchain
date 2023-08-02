#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet_prelude::*, 
	traits::fungibles,
	sp_runtime::{
		ArithmeticError,
		traits::{CheckedAdd, CheckedMul, IntegerSquareRoot, CheckedSub}
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
		traits::{fungible, fungibles},
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
	}

	/// Types defined here

	// gives us the AssetId type represents the identifier of a fungible asset within a runtime.
	pub type AssetIdOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<
	<T as frame_system::Config>::AccountId,
	>>::AssetId;


	pub type AssetBalanceOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct AssetPair<T: Config> {
		pub asset_1: AssetIdOf<T>,
		pub asset_2: AssetIdOf<T>,
	}
	impl<T: Config> AssetPair<T> {
		pub fn new(asset_1: AssetIdOf<T>, asset_2: AssetIdOf<T>) -> Self {
			Self {
				asset_1,
				asset_2,
			}
		}
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Pool<T: Config> {
    	/// The first token in the pair
    	pub token1: AssetIdOf<T>,
    
    	/// The second token in the pair
    	pub token2: AssetIdOf<T>,

    	/// Total amount of the first token
    	pub total_reserve_1: AssetBalanceOf<T>,

    	/// Total amount of the second token
    	pub total_reserve_2: AssetBalanceOf<T>,

    	// Total supply of the LP tokens
    	pub lp_supply: AssetBalanceOf<T>,
	}
	impl<T: Config> Pool<T> {
		pub fn new(
			token1: AssetIdOf<T>,
			token2: AssetIdOf<T>,
			total_reserve_1: AssetBalanceOf<T>,
			total_reserve_2: AssetBalanceOf<T>,
			lp_supply: AssetBalanceOf<T>,
		) -> Self {
			Self {
				token1,
				token2,
				total_reserve_1,
				total_reserve_2,
				lp_supply,
			}
		}
	}


	/// storage defined here
	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type PoolMap<T> =
		StorageMap<_, Blake2_128Concat, AssetPair<T>, Pool<T>>;

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

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
		// #[pallet::call_index(0)]
		// #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		// pub fn add_liquidity(
		// 	origin: OriginFor<T>,
		// 	asset_a: AssetIdOf<T>,
		// 	asset_b: AssetIdOf<T>,
		// 	amount_a: AssetBalanceOf<T>,
		// 	amount_b: AssetBalanceOf<T>,
		// 	) -> DispatchResult {
				
		// 	let key = Self::create_asset_pair(asset_a, asset_b)?;
		// 	match <PoolMap<T>>::get() {
		// 		// New Pool
		// 		None => {

		// 		},
		// 		Some(old) => {
		// 			// Increment the value read from storage; will error in the event of overflow.
		// 			let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
		// 			// Update the value in storage with the incremented result.
					
		// 			Ok(())
		// 		},
		// 	}
			
		// }
	}
}



impl<T: Config> Pallet<T> {

	// lp(a, b) == lp(b, a)
	// idea of logic would be:
	// 1. the user can submit in any order
	// 2. we "sort" the assets
	// 3. then create the id
	// you need to figure out what you want to return if anything
	fn create_asset_pair(
		asset_a: AssetIdOf<T>,
		asset_b: AssetIdOf<T>,
	) -> Result<AssetPair<T>, DispatchError> {
		ensure!(asset_a != asset_b, "cant use the same id twice");
		return if asset_a.encode() > asset_b.encode() {
			Ok(AssetPair::new(asset_b, asset_a))
		} else {
			Ok(AssetPair::new(asset_a, asset_b))
		}
	}

	// helper function to calculate the amount of LP tokens to mint and issue them 
	// lp = sqrt((A+a)*(B+b)) - sqrt(A*B) 
	// Where A and B are the current amount of tokenA and tokenB in the pool. a and b are the amounts of tokenA and tokenB that the user is adding to the pool
	// if new pool, the equation simplifies lp = sqrt(a*b)
	fn calculate_lp(
		additional_1: AssetBalanceOf<T>,
		additional_2: AssetBalanceOf<T>,
		pool: Pool<T>,
	) -> Result<AssetBalanceOf<T>, DispatchError> {
		// (A + a)
		let total_1 = pool.total_reserve_1.checked_add(&additional_1).ok_or(ArithmeticError::Overflow)?;
		// (B + b)
		let total_2 = pool.total_reserve_2.checked_add(&additional_2).ok_or(ArithmeticError::Overflow)?;
		// (A + a) * (B + b)
		let total_1_2 = total_1.checked_mul(&total_2).ok_or(ArithmeticError::Overflow)?;
		// sqrt((A + a) * (B + b))
		let sqrt_1 = IntegerSquareRoot::integer_sqrt(&total_1_2);
		// sqrt(A * B)
		let sqrt_2 = IntegerSquareRoot::integer_sqrt(&pool.total_reserve_1.checked_mul(&pool.total_reserve_2).ok_or(ArithmeticError::Overflow)?);
		// sqrt((A + a) * (B + b)) - sqrt(A * B)
		let lp = sqrt_1.checked_sub(&sqrt_2).ok_or(ArithmeticError::Underflow)?;
		
		return Ok(lp);
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
