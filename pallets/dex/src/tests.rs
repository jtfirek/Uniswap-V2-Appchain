
use crate::{mock::{*, self}, Error, Event};
use frame_support::{assert_noop, assert_ok, traits::fungibles::Inspect, assert_err};

#[test]
fn simple_add_remove_liquidity() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// setting up account 1 with 1000 of asset type 1 and 2
		assert_ok!(Dex::setup_account(1, vec![(1, 1000), (2, 1000)]));
		assert_eq!(Assets::total_balance(1, &1), 1000);
		assert_eq!(Assets::total_balance(2, &1), 1000);

		// deposit 500 of each asset
		assert_ok!(Dex::add_liquidity(RuntimeOrigin::signed(1), 1, 2, 500, 500));

		// Lp issued sqr(500*500) = 500
		System::assert_last_event(mock::RuntimeEvent::Dex(Event::LiquidityAdded{ asset_a: 1, asset_b: 2, amount_a: 500, amount_b: 500, amount_lp: 500 }));
		assert_eq!(Assets::total_balance(1, &1), 500);
		assert_eq!(Assets::total_balance(2, &1), 500);
		assert_eq!(Assets::total_balance(Dex::get_lp_id(&1, &2).unwrap(), &1), 500);

		assert_ok!(Dex::remove_liquidity(RuntimeOrigin::signed(1), 1, 2, 500));
	});
}


#[test]
fn simple_swap_withdraw() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// setting up account 1 with 1000 of asset type 1 and 2
		assert_ok!(Dex::setup_account(1, vec![(1, 1000), (2, 1000)]));
		// deposit 500 of each asset
		assert_ok!(Dex::add_liquidity(RuntimeOrigin::signed(1), 1, 2, 500, 500));

		// Lp issued sqr(500*500) = 500
		assert_eq!(Assets::total_balance(2, &1), 500);
		assert_eq!(Assets::total_balance(Dex::get_lp_id(&1, &2).unwrap(), &1), 500);

		// create a new account to swap
		assert_ok!(Dex::setup_account(2, vec![(1, 1000)]));


		// swap for 100 of asset 2
		// should be 103 input
		assert_ok!(Dex::swap_in_for_exact_out(RuntimeOrigin::signed(2), 1, 2, 150, 100));

		// account two should have 100 of asset 2
		assert_eq!(Assets::total_balance(2, &2), 100);
		// dex should have 629 of asset 1 and 400 of asset 2
		assert_eq!(Assets::total_balance(1, &Dex::account_id()), 629);
		assert_eq!(Assets::total_balance(2, &&Dex::account_id()), 400);

		// should fail because the max input they are providing is too low
		assert_noop!(Dex::swap_in_for_exact_out(RuntimeOrigin::signed(2), 1, 2, 20, 100), Error::<Test>::SlippageTooHigh);

		// withdraw liquidity from account 1 should get more total tokens due to the fee paid by account 2
		assert_ok!(Dex::remove_liquidity(RuntimeOrigin::signed(1), 1, 2, 500));

		let account1_net_worth = Assets::total_balance(1, &1) + Assets::total_balance(2, &1);
		assert!(account1_net_worth > 1000);
	});
}


#[test]
fn rewards_check() {
	// account 1 and account 2 put the same amount of funds into the pool but account 1 leaves it in longer and should get more rewards

	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		// setting up accounts
		assert_ok!(Dex::setup_account(1, vec![(1, 1000), (2, 1000)]));
		assert_ok!(Dex::setup_account(2, vec![(1, 1000), (2, 1000)]));
		assert_ok!(Dex::setup_account(3, vec![(1, 10_000), (2, 10_000)]));

		// account 1 and 2 add liquidity
		assert_ok!(Dex::add_liquidity(RuntimeOrigin::signed(1), 1, 2, 500, 500));
		assert_ok!(Dex::add_liquidity(RuntimeOrigin::signed(2), 1, 2, 500, 500));

		// account 3 does a bunch of swaps to increase the rewards
		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(3), 1, 2, 150, 0));
		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(3), 2, 1, 150, 0));
		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(3), 1, 2, 150, 0));
		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(3), 2, 1, 150, 0));
		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(3), 1, 2, 150, 0));
		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(3), 2, 1, 150, 0));
		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(3), 1, 2, 150, 0));
		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(3), 2, 1, 150, 0));

		//there should be more liquidity in the pool
		let pool_total = Assets::total_balance(1, &Dex::account_id()) + Assets::total_balance(2, &Dex::account_id());
		assert!(pool_total > 1000);

		// account 2 removes liquidity
		assert_ok!(Dex::remove_liquidity(RuntimeOrigin::signed(2), 1, 2, 500));

		// account 3 does a bunch of swaps to increase the rewards
		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(3), 1, 2, 150, 0));
		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(3), 2, 1, 150, 0));
		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(3), 1, 2, 150, 0));
		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(3), 2, 1, 150, 0));
		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(3), 1, 2, 150, 0));
		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(3), 2, 1, 150, 0));
		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(3), 1, 2, 150, 0));
		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(3), 2, 1, 150, 0));

		// account 1 removes liquidity
		assert_ok!(Dex::remove_liquidity(RuntimeOrigin::signed(1), 1, 2, 500));


		// account 1 should have more rewards than account 2
		let account1_rewards = Assets::total_balance(1, &1) + Assets::total_balance(2, &1);
		let account2_rewards = Assets::total_balance(1, &2) + Assets::total_balance(2, &2);
		assert!(account1_rewards > account2_rewards);
	});
}


#[test]
fn add_pool_same_asset_fail() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// setting up account 1 with 1000 of asset type 1 and 2
		assert_ok!(Dex::setup_account(1, vec![(1, 1000), (2, 1000)]));

		// can't create pool with the same asset
		assert_err!(Dex::add_liquidity(RuntimeOrigin::signed(1), 1, 1, 500, 500), Error::<Test>::SameAsset);
	});
}

#[test]
fn parameters_order_no_diff() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		assert_ok!(Dex::setup_account(1, vec![(1, 1000), (2, 1000)]));
		assert_ok!(Dex::add_liquidity(RuntimeOrigin::signed(1), 1, 2, 500, 500));

		// should work even though the parameters are in a different order
		assert_ok!(Dex::remove_liquidity(RuntimeOrigin::signed(1), 2, 1, 500));

		assert_ok!(Dex::add_liquidity(RuntimeOrigin::signed(1), 1, 2, 500, 500));
		assert_ok!(Dex::add_liquidity(RuntimeOrigin::signed(1), 2, 1, 500, 500));

		// should have added liquidity to the same pool
		let account_1_lp = Assets::total_balance(Dex::get_lp_id(&1, &2).unwrap(), &1);
		assert_eq!(account_1_lp, 1000);

		// should both represent the same lp token
		assert_eq!(Dex::get_lp_id(&1, &2).unwrap(), Dex::get_lp_id(&2, &1).unwrap());

	});
}

#[test]
fn changing_fee() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		assert_ok!(Dex::setup_account(1, vec![(1, 1000), (2, 1000)]));

	});
}