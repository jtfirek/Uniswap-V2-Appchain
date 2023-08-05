use crate::{mock::{*, self}, Error, Event};
use frame_support::{assert_noop, assert_ok, traits::fungibles::Inspect};

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