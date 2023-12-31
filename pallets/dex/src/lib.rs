#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::Vec,
	pallet_prelude::*,
	sp_runtime::{
		traits::{
			AccountIdConversion, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Hash,
			IntegerSquareRoot, One, TrailingZeroInput,
		},
		ArithmeticError, Percent,
	},
	traits::fungibles::{self, Create, Inspect, Mutate},
};
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use crate::ArithmeticError;
	use frame_support::{
		dispatch::Dispatchable,
		pallet_prelude::*,
		sp_runtime::{traits::CheckedMul, Percent},
		traits::{
			fungible,
			fungibles::{self, Create, Inspect, Mutate},
			tokens::{Fortitude::Force, Precision::BestEffort, Preservation::*},
		},
	};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::boxed::Box;
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

		type PermissionOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The DEXs pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<frame_support::PalletId>;

		// type RuntimeCall: Parameter + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin> +
		// GetDispatchInfo;
	}

	// gives us access to the asset id and balance types of the fungibles
	pub type AssetIdOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::AssetId;
	pub type AssetBalanceOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

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
				return Err("cant use the same id twice")
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

	/// STORAGE DEFINED HERE
	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Pool<T: Config> {
		// stores the asset ids and balances of the two assets in the pool in sorted order
		pub pool_pair: PoolPair<T>,

		// Total supply of the LP tokens
		pub lp_supply: AssetBalanceOf<T>,
	}
	impl<T: Config> Pool<T> {
		pub fn new(pool_pair: PoolPair<T>, lp_supply: AssetBalanceOf<T>) -> Self {
			Self { pool_pair: pool_pair.clone(), lp_supply }
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn something)]
	// The pools are stored by a key that is the asset id of the LP token
	pub type PoolMap<T> = StorageMap<_, Blake2_128Concat, AssetIdOf<T>, Pool<T>>;

	#[pallet::storage]
	pub type Fee<T> = StorageValue<_, u16, ValueQuery, FeeDefault>;
	pub struct FeeDefault(u16);
	impl Default for FeeDefault {
		fn default() -> Self {
			Self(3) // The default value is 50.
		}
	}
	impl frame_support::traits::Get<u16> for FeeDefault {
		fn get() -> u16 {
			Self::default().0
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored {
			something: u32,
			who: T::AccountId,
		},

		// Liquidity added to the pool
		LiquidityAdded {
			asset_a: AssetIdOf<T>,
			asset_b: AssetIdOf<T>,
			amount_a: AssetBalanceOf<T>,
			amount_b: AssetBalanceOf<T>,
			amount_lp: AssetBalanceOf<T>,
		},

		// Liquidity removed from the pool
		LiquidityRemoved {
			asset_a: AssetIdOf<T>,
			asset_b: AssetIdOf<T>,
			amount_a: AssetBalanceOf<T>,
			amount_b: AssetBalanceOf<T>,
		},

		// exchange rate between represented as a percent `asset_out` / `asset_in`
		PriceOracleEvent {
			rate: Percent,
			asset_in: AssetIdOf<T>,
			asset_out: AssetIdOf<T>,
		},

		// Swap event
		SwapEvent {
			asset_in: AssetIdOf<T>,
			asset_out: AssetIdOf<T>,
			amount_in: AssetBalanceOf<T>,
			amount_out: AssetBalanceOf<T>,
		},

		// Fee update
		FeeUpdated {
			new_fee: u16,
		},

		// Flash loan event
		FlashLoanEvent {
			asset_id: AssetIdOf<T>,
			amount: AssetBalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,

		// slippage too high
		SlippageTooHigh,

		// Cannot create pool with the same asset
		SameAsset,

		// Trying to access a pool that doesn't exist
		NoPool,

		// insufficient lp balance
		InsufficientLPBalance,

		// Not allowed to set fee
		NotAllowedToSetFee,

		// Insufficient liquidity for flash loan
		InsufficientLiquidity,

		// Insufficient repayment for flash loan
		InsufficientRepayment,

		// Call failed
		CallFailed,
	}

	/// DISPATCHABLE FUNCTIONS DEFINED HERE
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Initiates a request to add liquidity to a specific pool pair.
		/// If the pool does not exist, it is created and the initial liquidity provided is minted.
		/// If the pool does exist, the function calculates the additional liquidity to be minted
		/// and adds it to the pool.
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
			let add_amounts =
				PoolPair::<T>::new(asset_a.clone(), amount_a, asset_b.clone(), amount_b)?;
			let lp_amount;
			match <PoolMap<T>>::get(&cur_lp_id) {
				None => {
					// New Pool
					lp_amount = Self::calculate_lp(&add_amounts, None)?;
					let _ = T::Fungibles::create(
						cur_lp_id.clone(),
						Self::account_id(),
						true,
						lp_amount,
					);
					T::Fungibles::mint_into(cur_lp_id.clone(), &who, lp_amount)?;
					let new_pool = Pool::<T>::new(add_amounts, lp_amount);
					<PoolMap<T>>::insert(&cur_lp_id, new_pool);
				},
				Some(existing_pool) => {
					lp_amount = Self::calculate_lp(&add_amounts, Some(&existing_pool))?;
					T::Fungibles::mint_into(cur_lp_id.clone(), &who, lp_amount)?;
					Self::increase_pool(&add_amounts, &lp_amount, &cur_lp_id)?;
				},
			}

			T::Fungibles::transfer(
				asset_a.clone(),
				&who,
				&Self::account_id(),
				amount_a,
				Expendable,
			)?;
			T::Fungibles::transfer(
				asset_b.clone(),
				&who,
				&Self::account_id(),
				amount_b,
				Expendable,
			)?;

			Self::deposit_event(Event::LiquidityAdded {
				asset_a,
				asset_b,
				amount_a,
				amount_b,
				amount_lp: lp_amount,
			});
			Ok(())
		}

		/// Removes liquidity from a given pool pair by burning LP tokens.
		/// The function checks the balance of the LP tokens, calculates the amount of each asset to
		/// return, burns the LP tokens, updates the pool, and then returns assets to the user.
		/// If LP tokens falls to default, the pool is removed from storage
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			asset_a: AssetIdOf<T>,
			asset_b: AssetIdOf<T>,
			token_amount: AssetBalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let cur_lp_id = Self::get_lp_id(&asset_a, &asset_b)?;
			ensure!(
				T::Fungibles::balance(cur_lp_id.clone(), &who) >= token_amount,
				Error::<T>::InsufficientLPBalance
			);

			let pool = <PoolMap<T>>::get(&cur_lp_id).ok_or(Error::<T>::NoPool)?;
			let amount_1 = pool
				.pool_pair
				.amount_1
				.checked_mul(&token_amount)
				.ok_or(ArithmeticError::Overflow)? /
				pool.lp_supply;
			let amount_2 = pool
				.pool_pair
				.amount_2
				.checked_mul(&token_amount)
				.ok_or(ArithmeticError::Overflow)? /
				pool.lp_supply;

			T::Fungibles::burn_from(cur_lp_id.clone(), &who, token_amount, BestEffort, Force)?;
			Self::decrease_pool(&amount_1, &amount_2, &token_amount, &cur_lp_id)?;
			T::Fungibles::transfer(
				pool.pool_pair.asset_1,
				&Self::account_id(),
				&who,
				amount_1,
				Expendable,
			)?;
			T::Fungibles::transfer(
				pool.pool_pair.asset_2,
				&Self::account_id(),
				&who,
				amount_2,
				Expendable,
			)?;

			Self::deposit_event(Event::LiquidityRemoved {
				asset_a,
				asset_b,
				amount_a: amount_1,
				amount_b: amount_2,
			});
			Ok(())
		}

		/// Performs an asset swap, providing an exact quantity of one asset to receive another.
		/// The function retrieves the liquidity pool, calculates the output amount, and performs a
		/// slippage check. If the trade is viable, it transfers the assets and updates the pool.
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
			let cur_lp_id = Self::get_lp_id(&asset_in, &asset_out)?;
			let pool = <PoolMap<T>>::get(&cur_lp_id).ok_or(Error::<T>::NoneValue)?;

			let amount_out = Self::calculate_out(&exact_in, &asset_in, &pool)?;
			if amount_out.0 < min_out {
				return Err(Error::<T>::SlippageTooHigh.into())
			}

			T::Fungibles::transfer(
				asset_in.clone(),
				&who,
				&Self::account_id(),
				exact_in,
				Expendable,
			)?;
			T::Fungibles::transfer(
				asset_out.clone(),
				&Self::account_id(),
				&who,
				amount_out.0,
				Protect,
			)?;
			<PoolMap<T>>::insert(&cur_lp_id, amount_out.1);

			Self::deposit_event(Event::SwapEvent {
				asset_in,
				asset_out,
				amount_in: exact_in,
				amount_out: amount_out.0,
			});
			Ok(())
		}

		/// Performs an asset swap aiming for an exact output amount, while allowing for a maximum
		/// input. The function retrieves the liquidity pool, calculates the potential input based
		/// on required output, checks for slippage, performs asset transfers if the trade is
		/// viable, and updates the pool.
		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn swap_in_for_exact_out(
			origin: OriginFor<T>,
			asset_in: AssetIdOf<T>,
			asset_out: AssetIdOf<T>,
			max_in: AssetBalanceOf<T>,
			exact_out: AssetBalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let cur_lp_id = Self::get_lp_id(&asset_in, &asset_out)?;
			let pool = <PoolMap<T>>::get(&cur_lp_id).ok_or(Error::<T>::NoPool)?;

			let amount_in = Self::calculate_in(&exact_out, &asset_out, &pool)?;
			if amount_in.0 > max_in {
				return Err(Error::<T>::SlippageTooHigh.into())
			}

			// trade is good transfer assets accordingly
			T::Fungibles::transfer(
				asset_in.clone(),
				&who,
				&Self::account_id(),
				amount_in.0,
				Protect,
			)?;
			T::Fungibles::transfer(
				asset_out.clone(),
				&Self::account_id(),
				&who,
				exact_out,
				Expendable,
			)?;
			<PoolMap<T>>::insert(&cur_lp_id, amount_in.1);

			Self::deposit_event(Event::SwapEvent {
				asset_in,
				asset_out,
				amount_in: amount_in.0,
				amount_out: exact_out,
			});
			Ok(())
		}

		/// This function computes the price ratio between `asset_in` and `asset_out` using the
		/// available liquidity pool.
		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn price_oracle(
			_origin: OriginFor<T>,
			asset_in: AssetIdOf<T>,
			asset_out: AssetIdOf<T>,
		) -> DispatchResult {
			let cur_lp_id = Self::get_lp_id(&asset_in, &asset_out)?;
			let pool = <PoolMap<T>>::get(&cur_lp_id).ok_or(Error::<T>::NoPool)?;

			let oracle_price;
			if asset_in == pool.pool_pair.asset_1 {
				oracle_price =
					Percent::from_rational(pool.pool_pair.amount_2, pool.pool_pair.amount_1);
			} else {
				oracle_price =
					Percent::from_rational(pool.pool_pair.amount_1, pool.pool_pair.amount_2);
			};
			Self::deposit_event(Event::PriceOracleEvent {
				rate: oracle_price,
				asset_in,
				asset_out,
			});
			Ok(())
		}

		/// This function allows the permission origin to set the fee
		#[pallet::call_index(5)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn set_fee(origin: OriginFor<T>, new_fee: u16) -> DispatchResult {
			ensure!(
				T::PermissionOrigin::try_origin(origin).is_ok(),
				Error::<T>::NotAllowedToSetFee
			);
			<Fee<T>>::put(new_fee);
			Self::deposit_event(Event::FeeUpdated { new_fee });
			Ok(())
		}

		/// This function allows a user to borrow a specified amount of an asset
		/// temporarily for execution of a predefined function (`call`), provided
		/// that the asset has sufficient liquidity. The borrowed assets are
		/// automatically transferred to the user.
		#[pallet::call_index(6)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn flash_loan(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T>,
			amount: AssetBalanceOf<T>,
			call: Box<<T as frame_system::Config>::RuntimeCall>,
		) -> DispatchResult {
			// ensuring sufficient liquidity and permission
			let who = ensure_signed(origin.clone())?;
			let total_liquidity = T::Fungibles::total_issuance(asset_id.clone());
			ensure!(total_liquidity >= amount, Error::<T>::InsufficientLiquidity);
			T::Fungibles::transfer(
				asset_id.clone(),
				&Self::account_id(),
				&who,
				amount,
				Expendable,
			)?;

			// execute the borrowers contract
			call.dispatch(origin).map_err(|_| Error::<T>::CallFailed)?;
			let fee = Self::calculate_fees(&amount)?;
			ensure!(
				T::Fungibles::balance(asset_id.clone(), &Self::account_id()) >=
					total_liquidity + fee,
				Error::<T>::InsufficientRepayment
			);

			Self::deposit_event(Event::FlashLoanEvent { asset_id, amount });
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Calculates the amount of LP tokens.
	///
	/// For existing pools, the formula is `lp = sqrt((A+a)*(B+b)) - sqrt(A*B)`. `A` and `B` are the
	/// current pool amounts and `a` and `b` are the amounts to add. For new pools, `lp =
	/// sqrt(a*b)`.
	fn calculate_lp(
		new_pair: &PoolPair<T>,
		pool: Option<&Pool<T>>,
	) -> Result<AssetBalanceOf<T>, DispatchError> {
		if let Some(pool) = pool {
			// Calculate LP for existing pool
			let total_1 = pool
				.pool_pair
				.amount_1
				.checked_add(&new_pair.amount_1)
				.ok_or(ArithmeticError::Overflow)?;
			let total_2 = pool
				.pool_pair
				.amount_2
				.checked_add(&new_pair.amount_2)
				.ok_or(ArithmeticError::Overflow)?;
			let total_product = total_1.checked_mul(&total_2).ok_or(ArithmeticError::Overflow)?;
			let sqrt_total = IntegerSquareRoot::integer_sqrt(&total_product);

			let current_product = pool
				.pool_pair
				.amount_1
				.checked_mul(&pool.pool_pair.amount_2)
				.ok_or(ArithmeticError::Overflow)?;
			let sqrt_current = IntegerSquareRoot::integer_sqrt(&current_product);

			let lp = sqrt_total.checked_sub(&sqrt_current).ok_or(ArithmeticError::Underflow)?;
			Ok(lp)
		} else {
			// Calculate LP for new pool: sqrt(a * b)
			let product = new_pair
				.amount_1
				.checked_mul(&new_pair.amount_2)
				.ok_or(ArithmeticError::Overflow)?;
			Ok(IntegerSquareRoot::integer_sqrt(&product))
		}
	}

	/// The account ID of the dex pallet. This account stores all of the assets in the dex.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	/// Generates a liquidity pool ID from the given asset IDs, ensuring the assets are distinct.
	/// The pool ID is based on a hash of the sorted asset IDs.
	pub fn get_lp_id(
		asset_a: &AssetIdOf<T>,
		asset_b: &AssetIdOf<T>,
	) -> Result<AssetIdOf<T>, DispatchError> {
		ensure!(asset_a != asset_b, Error::<T>::SameAsset);
		let bytes = if asset_a.encode() > asset_b.encode() {
			T::Hashing::hash(&(asset_b, asset_a).encode());
		} else {
			T::Hashing::hash(&(asset_a, asset_b).encode());
		};
		let generated_lp_id = AssetIdOf::<T>::decode(&mut TrailingZeroInput::new(&bytes.encode()))
			.expect("in our PBA exam, we assume all bytes can be ID");
		Ok(generated_lp_id)
	}

	// adds liquidity to an existing pool
	pub fn increase_pool(
		new_pair: &PoolPair<T>,
		new_lp: &AssetBalanceOf<T>,
		pool_id: &AssetIdOf<T>,
	) -> Result<(), DispatchError> {
		let mut pool = <PoolMap<T>>::get(pool_id).ok_or(Error::<T>::NoPool)?;
		pool.pool_pair.amount_1 = pool
			.pool_pair
			.amount_1
			.checked_add(&new_pair.amount_1)
			.ok_or(ArithmeticError::Overflow)?;
		pool.pool_pair.amount_2 = pool
			.pool_pair
			.amount_2
			.checked_add(&new_pair.amount_2)
			.ok_or(ArithmeticError::Overflow)?;
		pool.lp_supply = pool.lp_supply.checked_add(&new_lp).ok_or(ArithmeticError::Overflow)?;
		<PoolMap<T>>::insert(pool_id, pool);
		Ok(())
	}

	// removes liquidity from an existing pool
	pub fn decrease_pool(
		amount_1: &AssetBalanceOf<T>,
		amount_2: &AssetBalanceOf<T>,
		new_lp: &AssetBalanceOf<T>,
		pool_id: &AssetIdOf<T>,
	) -> Result<(), DispatchError> {
		let mut pool = <PoolMap<T>>::get(pool_id).ok_or(Error::<T>::NoPool)?;
		pool.pool_pair.amount_1 = pool
			.pool_pair
			.amount_1
			.checked_sub(&amount_1)
			.ok_or(ArithmeticError::Underflow)?;
		pool.pool_pair.amount_2 = pool
			.pool_pair
			.amount_2
			.checked_sub(&amount_2)
			.ok_or(ArithmeticError::Underflow)?;
		pool.lp_supply = pool.lp_supply.checked_sub(&new_lp).ok_or(ArithmeticError::Underflow)?;

		if pool.lp_supply == Default::default() {
			<PoolMap<T>>::remove(pool_id);
		} else {
			<PoolMap<T>>::insert(pool_id, pool);
		}
		Ok(())
	}

	// calculates the amount of fees to be collected
	// currently we have a hard coded 3% fee
	pub fn calculate_fees(
		amount_in: &AssetBalanceOf<T>,
	) -> Result<AssetBalanceOf<T>, DispatchError> {
		let percent = Percent::from_rational(<Fee<T>>::get(), 100u16);
		Ok(percent.mul_ceil(*amount_in))
	}

	// getter for the interface to grab the fee
	pub fn get_fee() -> u16 {
		<Fee<T>>::get()
	}

	// calculates the output of the exchange based on constant product formula
	// X * Y = K
	// returns both the output and the new pool
	pub fn calculate_out(
		amount_in: &AssetBalanceOf<T>,
		input_type: &AssetIdOf<T>,
		pool: &Pool<T>,
	) -> Result<(AssetBalanceOf<T>, Pool<T>), DispatchError> {
		// remove the fee from the input
		let fee = Self::calculate_fees(&amount_in)?; // 3 percent fee
		let exact_in_after_fee = amount_in.checked_sub(&fee).ok_or(ArithmeticError::Underflow)?;

		// get the constant k
		let k = pool
			.pool_pair
			.amount_1
			.checked_mul(&pool.pool_pair.amount_2)
			.ok_or(ArithmeticError::Overflow)?;

		let input_pool;
		let output_pool;
		if *input_type == pool.pool_pair.asset_1 {
			input_pool = pool.pool_pair.amount_1;
			output_pool = pool.pool_pair.amount_2;
		} else {
			input_pool = pool.pool_pair.amount_2;
			output_pool = pool.pool_pair.amount_1;
		}
		// New X value after adding to the pool
		let new_input_pool =
			input_pool.checked_add(&exact_in_after_fee).ok_or(ArithmeticError::Overflow)?;
		// Y = K / X : New Y value after adding to the pool
		let new_output_pool = k.checked_div(&new_input_pool).ok_or(ArithmeticError::Underflow)?;
		// old Y - Y = output
		let output = output_pool.checked_sub(&new_output_pool).ok_or(ArithmeticError::Underflow)?;

		let new_pool: Pool<T>;
		if *input_type == pool.pool_pair.asset_1 {
			new_pool = Pool::<T>::new(
				PoolPair::<T>::new(
					pool.pool_pair.asset_1.clone(),
					new_input_pool + fee,
					pool.pool_pair.asset_2.clone(),
					new_output_pool,
				)?,
				pool.lp_supply,
			);
		} else {
			new_pool = Pool::<T>::new(
				PoolPair::<T>::new(
					pool.pool_pair.asset_1.clone(),
					new_output_pool,
					pool.pool_pair.asset_2.clone(),
					new_input_pool + fee,
				)?,
				pool.lp_supply,
			);
		}
		Ok((output, new_pool))
	}

	// calculates the input of the exchange based on constant product formula
	// X * Y = K
	// returns both the input and the new pool
	pub fn calculate_in(
		amount_out: &AssetBalanceOf<T>,
		output_type: &AssetIdOf<T>,
		pool: &Pool<T>,
	) -> Result<(AssetBalanceOf<T>, Pool<T>), DispatchError> {
		// get the constant k
		let k = pool
			.pool_pair
			.amount_1
			.checked_mul(&pool.pool_pair.amount_2)
			.ok_or(ArithmeticError::Overflow)?;
		let input_pool;
		let output_pool;
		if *output_type == pool.pool_pair.asset_1 {
			input_pool = pool.pool_pair.amount_2;
			output_pool = pool.pool_pair.amount_1;
		} else {
			input_pool = pool.pool_pair.amount_1;
			output_pool = pool.pool_pair.amount_2;
		}

		// New X value after removing from the pool
		let new_output_pool =
			output_pool.checked_sub(&amount_out).ok_or(ArithmeticError::Overflow)?;
		// Y = K / X : New Y value after adding to the pool
		let new_input_pool = k.checked_div(&new_output_pool).ok_or(ArithmeticError::Underflow)?;
		// Y old - Y : The new pool will be larger this time as we are calculating the input
		let mut input_required =
			new_input_pool.checked_sub(&input_pool).ok_or(ArithmeticError::Underflow)?;

		let fee = Self::calculate_fees(&input_required)?;
		input_required = input_required.checked_add(&fee).ok_or(ArithmeticError::Overflow)?;
		let new_pool: Pool<T>;
		if *output_type == pool.pool_pair.asset_1 {
			new_pool = Pool::<T>::new(
				PoolPair::<T>::new(
					pool.pool_pair.asset_1.clone(),
					new_output_pool,
					pool.pool_pair.asset_2.clone(),
					new_input_pool + fee,
				)?,
				pool.lp_supply,
			);
		} else {
			new_pool = Pool::<T>::new(
				PoolPair::<T>::new(
					pool.pool_pair.asset_1.clone(),
					new_input_pool + fee,
					pool.pool_pair.asset_2.clone(),
					new_output_pool,
				)?,
				pool.lp_supply,
			);
		}
		Ok((input_required, new_pool))
	}

	// function for setting up accounts while testing
	pub fn setup_account(
		who: T::AccountId,
		assets: Vec<(AssetIdOf<T>, AssetBalanceOf<T>)>,
	) -> DispatchResult {
		for (asset_id, asset_balance) in assets {
			if !T::Fungibles::asset_exists(asset_id.clone()) {
				let pallet_account = Self::account_id();
				T::Fungibles::create(asset_id.clone(), pallet_account, true, One::one())?;
			}

			T::Fungibles::mint_into(asset_id.clone(), &who, asset_balance)?;
		}
		Ok(())
	}

	// function to get asset balance
	pub fn asset_balance(who: T::AccountId, asset_id: AssetIdOf<T>) -> AssetBalanceOf<T> {
		T::Fungibles::balance(asset_id, &who)
	}
}

// Look at `../interface/` to better understand this API.
impl<T: Config> pba_interface::DexInterface for Pallet<T> {
	type AccountId = T::AccountId;
	type AssetId = <T::Fungibles as fungibles::Inspect<Self::AccountId>>::AssetId;
	type AssetBalance = <T::Fungibles as fungibles::Inspect<Self::AccountId>>::Balance;

	fn setup_account(_who: Self::AccountId) -> DispatchResult {
		Ok(())
	}

	fn mint_asset(
		_who: Self::AccountId,
		_token_id: Self::AssetId,
		_amount: Self::AssetBalance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn asset_balance(_who: Self::AccountId, _token_id: Self::AssetId) -> Self::AssetBalance {
		Self::asset_balance(_who, _token_id)
	}

	fn swap_fee() -> u16 {
		// convert my percentage to basis points
		// default is 3 u16 to represent 3%
		let fee = Self::get_fee();

		// 3% -> 300 basis points
		fee * 100
	}

	fn lp_id(_asset_a: Self::AssetId, _asset_b: Self::AssetId) -> Self::AssetId {
		Self::get_lp_id(&_asset_a, &_asset_b).unwrap()
	}

	fn add_liquidity(
		_who: Self::AccountId,
		_asset_a: Self::AssetId,
		_asset_b: Self::AssetId,
		_amount_a: Self::AssetBalance,
		_amount_b: Self::AssetBalance,
	) -> DispatchResult {
		Self::add_liquidity(
			frame_system::RawOrigin::Signed(_who).into(),
			_asset_a,
			_asset_b,
			_amount_a,
			_amount_b,
		)
	}

	fn remove_liquidity(
		_who: Self::AccountId,
		_asset_a: Self::AssetId,
		_asset_b: Self::AssetId,
		_token_amount: Self::AssetBalance,
	) -> DispatchResult {
		Self::remove_liquidity(
			frame_system::RawOrigin::Signed(_who).into(),
			_asset_a,
			_asset_b,
			_token_amount,
		)
	}

	fn swap_exact_in_for_out(
		_who: Self::AccountId,
		_asset_in: Self::AssetId,
		_asset_out: Self::AssetId,
		_exact_in: Self::AssetBalance,
		_min_out: Self::AssetBalance,
	) -> DispatchResult {
		Self::swap_exact_in_for_out(
			frame_system::RawOrigin::Signed(_who).into(),
			_asset_in,
			_asset_out,
			_exact_in,
			_min_out,
		)
	}

	fn swap_in_for_exact_out(
		_origin: Self::AccountId,
		_asset_in: Self::AssetId,
		_asset_out: Self::AssetId,
		_max_in: Self::AssetBalance,
		_exact_out: Self::AssetBalance,
	) -> DispatchResult {
		Self::swap_in_for_exact_out(
			frame_system::RawOrigin::Signed(_origin).into(),
			_asset_in,
			_asset_out,
			_max_in,
			_exact_out,
		)
	}
}
