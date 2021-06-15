#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch::Vec, ensure, StorageMap,
	StorageValue,
};
use frame_system::ensure_signed;
use sp_runtime::DispatchResult;
use sp_std::{
	cmp::{Eq, PartialEq},
	prelude::*,
};
use utilities::{CommonManager, Did, Token, TokenManager};

// #[cfg(test)]
// mod tests;

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Common: CommonManager<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Config> as Token {
		pub Tokens get(fn token): map hasher(blake2_128_concat) Did => Option<Token<T::AccountId>>;
		pub Balances get(fn balance_of): map hasher(blake2_128_concat) (T::AccountId, Did) => u64;
		pub StaticBalances get(fn static_balance_of): map hasher(blake2_128_concat) (Did, Did) => u64;
		pub FreeBalances get(fn free_balance_of): map hasher(blake2_128_concat) (T::AccountId, Did) => u64;
		pub FreezedBalances get(fn freezed_balance_of): map hasher(blake2_128_concat) (T::AccountId, Did) => u64;

		pub OwnedTokens get(fn owned_token): map hasher(blake2_128_concat) (T::AccountId, u64) => Option<Did>;
		pub OwnedTokensIndex get(fn owned_token_index): map hasher(blake2_128_concat) T::AccountId => u64;

		pub Nonce get(fn nonce): u64;

	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Config>::AccountId,
	{
		Issued(AccountId, Did, u64),
		Transferd(AccountId, AccountId, Did, u64, Option<Vec<u8>>),
		StaticTransferdIn(AccountId, Did, Did, u64),
		StaticTransferdOut(Did, AccountId, Did, u64),
		Freezed(AccountId, Did, u64),
		UnFreezed(AccountId, Did, u64),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Attempted to initialize the token after it had already been initialized.
		AlreadyInitialized,
		/// Attempted to transfer more funds than were available
		InsufficientFunds,
		NoMatchingToken,
		MemoLengthExceedLimitation,
		SenderHaveNoToken,
		BalanceNotEnough,
		AmountOverflow,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// Initialize the token
		/// transfers the total_supply amout to the caller
		#[weight = 10_000]
		fn issue(origin, total_supply: u64, symbol: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::_issue(sender, total_supply, symbol)
		}

		/// Transfer tokens from one account to another
		#[weight = 10_000]
		fn transfer(origin, to: T::AccountId, token_id: Did, amount: u64, memo: Option<Vec<u8>>)-> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::_transfer(sender, to, token_id, amount, memo)
		}
	}
}
impl<T: Config> Module<T> {
	fn _issue(from: T::AccountId, total_supply: u64, symbol: Vec<u8>) -> DispatchResult {
		let nonce = Nonce::get();
		let new_token_id = T::Common::generate_did(from.clone(), nonce.clone());

		let token = Token {
			tid: new_token_id.clone(),
			owner: from.clone(),
			symbol: symbol.clone(),
			total_supply,
		};

		Nonce::mutate(|n| *n += 1);
		Tokens::<T>::insert(new_token_id.clone(), token);
		Balances::<T>::insert((from.clone(), new_token_id.clone()), total_supply);
		FreeBalances::<T>::insert((from.clone(), new_token_id.clone()), total_supply);

		let owned_token_index = OwnedTokensIndex::<T>::get(from.clone());
		OwnedTokens::<T>::insert((from.clone(), owned_token_index), new_token_id.clone());
		OwnedTokensIndex::<T>::insert(from.clone(), owned_token_index + 1);

		Self::deposit_event(RawEvent::Issued(from, new_token_id.clone(), total_supply));

		Ok(())
	}
	fn _transfer(
		sender: T::AccountId,
		to: T::AccountId,
		token_id: Did,
		amount: u64,
		memo: Option<Vec<u8>>,
	) -> DispatchResult {
		let token = Self::token(&token_id);
		ensure!(token.is_some(), Error::<T>::NoMatchingToken);

		if let Some(memos) = &memo {
			ensure!(memos.len() <= 512, Error::<T>::MemoLengthExceedLimitation);
		}

		ensure!(
			<FreeBalances<T>>::contains_key((sender.clone(), &token_id)),
			Error::<T>::SenderHaveNoToken
		);

		let from_amount = Self::balance_of((sender.clone(), token_id.clone()));
		ensure!(from_amount >= amount, Error::<T>::BalanceNotEnough);
		let new_from_amount = from_amount - amount;

		let from_free_amount = Self::free_balance_of((sender.clone(), token_id.clone()));
		ensure!(from_free_amount >= amount, Error::<T>::BalanceNotEnough);
		let new_from_free_amount = from_free_amount - amount;

		let to_amount = Self::balance_of((to.clone(), token_id.clone()));
		let new_to_amount = to_amount + amount;
		// ensure!(
		// 	new_to_amount <= T::Balance::max_value(),
		// 	Error::<T>::AmountOverflow
		// );

		let to_free_amount = Self::free_balance_of((to.clone(), token_id.clone()));
		let new_to_free_amount = to_free_amount + amount;
		// ensure!(
		// 	new_to_free_amount <= T::Balance::max_value(),
		// 	Error::<T>::AmountOverflow
		// );

		Balances::<T>::insert((sender.clone(), token_id.clone()), new_from_amount);
		FreeBalances::<T>::insert((sender.clone(), token_id.clone()), new_from_free_amount);
		Balances::<T>::insert((to.clone(), token_id.clone()), new_to_amount);
		FreeBalances::<T>::insert((to.clone(), token_id.clone()), new_to_free_amount);
		Self::deposit_event(RawEvent::Transferd(sender, to, token_id, amount, memo));

		Ok(())
	}

	fn _static_transfer_in(
		sender: T::AccountId,
		to: Did,
		token_id: Did,
		amount: u64,
	) -> DispatchResult {
		let token = Self::token(&token_id);
		ensure!(token.is_some(), Error::<T>::NoMatchingToken);

		ensure!(
			<FreeBalances<T>>::contains_key((sender.clone(), &token_id)),
			Error::<T>::SenderHaveNoToken
		);

		let from_amount = Self::balance_of((sender.clone(), token_id.clone()));
		ensure!(from_amount >= amount, Error::<T>::BalanceNotEnough);
		let new_from_amount = from_amount - amount;

		let from_free_amount = Self::free_balance_of((sender.clone(), token_id.clone()));
		ensure!(from_free_amount >= amount, Error::<T>::BalanceNotEnough);
		let new_from_free_amount = from_free_amount - amount;

		let to_amount = Self::static_balance_of((to.clone(), token_id.clone()));
		let new_to_amount = to_amount + amount;

		Balances::<T>::insert((sender.clone(), token_id.clone()), new_from_amount);
		FreeBalances::<T>::insert((sender.clone(), token_id.clone()), new_from_free_amount);
		StaticBalances::insert((to.clone(), token_id.clone()), new_to_amount);
		Self::deposit_event(RawEvent::StaticTransferdIn(sender, to, token_id, amount));

		Ok(())
	}
	fn _static_transfer_out(
		sender: Did,
		to: T::AccountId,
		token_id: Did,
		amount: u64,
	) -> DispatchResult {
		let token = Self::token(&token_id);
		ensure!(token.is_some(), Error::<T>::NoMatchingToken);

		let from_amount = Self::static_balance_of((sender.clone(), token_id.clone()));
		ensure!(from_amount >= amount, Error::<T>::BalanceNotEnough);
		let new_from_amount = from_amount - amount;

		let to_amount = Self::balance_of((to.clone(), token_id.clone()));
		let new_to_amount = to_amount + amount;

		let to_free_amount = Self::free_balance_of((to.clone(), token_id.clone()));
		let new_to_free_amount = to_free_amount + amount;
		// ensure!(
		// 	new_to_free_amount <= T::Balance::max_value(),
		// 	Error::<T>::AmountOverflow
		// );

		StaticBalances::insert((sender.clone(), token_id.clone()), new_from_amount);
		Balances::<T>::insert((to.clone(), token_id.clone()), new_to_amount);
		FreeBalances::<T>::insert((to.clone(), token_id.clone()), new_to_free_amount);
		Self::deposit_event(RawEvent::StaticTransferdOut(sender, to, token_id, amount));

		Ok(())
	}

	fn _freeze(sender: T::AccountId, token_id: Did, amount: u64) -> DispatchResult {
		let token = Self::token(&token_id);
		ensure!(token.is_some(), Error::<T>::NoMatchingToken);

		ensure!(
			FreeBalances::<T>::contains_key((sender.clone(), token_id.clone())),
			Error::<T>::SenderHaveNoToken
		);

		let old_free_amount = Self::free_balance_of((sender.clone(), token_id.clone()));
		ensure!(old_free_amount >= amount, Error::<T>::BalanceNotEnough);

		let old_freezed_amount = Self::freezed_balance_of((sender.clone(), token_id.clone()));
		// ensure!(
		// 	old_freezed_amount + amount <= T::Balance::max_value(),
		// 	Error::<T>::AmountOverflow
		// );

		FreeBalances::<T>::insert((sender.clone(), token_id.clone()), old_free_amount - amount);
		FreezedBalances::<T>::insert(
			(sender.clone(), token_id.clone()),
			old_freezed_amount + amount,
		);

		Self::deposit_event(RawEvent::Freezed(sender, token_id, amount));

		Ok(())
	}

	pub fn _unfreeze(sender: T::AccountId, token_id: Did, amount: u64) -> DispatchResult {
		let token = Self::token(&token_id);
		ensure!(token.is_some(), Error::<T>::NoMatchingToken);

		ensure!(
			FreeBalances::<T>::contains_key((sender.clone(), token_id.clone())),
			Error::<T>::SenderHaveNoToken
		);

		let old_freezed_amount = Self::freezed_balance_of((sender.clone(), token_id.clone()));
		ensure!(old_freezed_amount >= amount, Error::<T>::BalanceNotEnough);

		let old_free_amount = Self::free_balance_of((sender.clone(), token_id.clone()));
		// ensure!(
		// 	old_free_amount + amount <= T::Balance::max_value(),
		// 	Error::<T>::AmountOverflow
		// );

		FreeBalances::<T>::insert((sender.clone(), token_id.clone()), old_free_amount + amount);
		FreezedBalances::<T>::insert(
			(sender.clone(), token_id.clone()),
			old_freezed_amount - amount,
		);

		Self::deposit_event(RawEvent::UnFreezed(sender, token_id, amount));

		Ok(())
	}

	fn _ensure_free_balance(sender: T::AccountId, token_id: Did, amount: u64) -> DispatchResult {
		let token = Self::token(&token_id);
		ensure!(token.is_some(), Error::<T>::NoMatchingToken);

		ensure!(
			FreeBalances::<T>::contains_key((sender.clone(), token_id.clone())),
			Error::<T>::SenderHaveNoToken
		);

		let free_amount = Self::free_balance_of((sender.clone(), token_id.clone()));
		ensure!(free_amount >= amount, Error::<T>::BalanceNotEnough);

		Ok(())
	}
}

impl<T: Config> TokenManager<T::AccountId> for Module<T> {
	// issue
	fn issue(from: T::AccountId, total_supply: u64, symbol: Vec<u8>) -> DispatchResult {
		Self::_issue(from, total_supply, symbol)
	}

	// transfer
	fn transfer(
		from: T::AccountId,
		to: T::AccountId,
		token_id: Did,
		value: u64,
		memo: Option<Vec<u8>>,
	) -> DispatchResult {
		Self::_transfer(from, to, token_id, value, memo)
	}
	fn static_transfer_in(
		from: T::AccountId,
		to: Did,
		token_id: Did,
		value: u64,
	) -> DispatchResult {
		Self::_static_transfer_in(from, to, token_id, value)
	}
	fn static_transfer_out(
		from: Did,
		to: T::AccountId,
		token_id: Did,
		value: u64,
	) -> DispatchResult {
		Self::_static_transfer_out(from, to, token_id, value)
	}

	// freeze
	fn freeze(from: T::AccountId, token_id: Did, value: u64) -> DispatchResult {
		Self::_freeze(from, token_id, value)
	}

	// unfreeze
	fn unfreeze(from: T::AccountId, token_id: Did, value: u64) -> DispatchResult {
		Self::_unfreeze(from, token_id, value)
	}

	// query
	fn balance_of(from: T::AccountId, token_id: Did) -> u64 {
		Self::balance_of((from, token_id))
	}

	fn static_balance_of(from: Did, token_id: Did) -> u64 {
		Self::static_balance_of((from, token_id))
	}
	fn owner_of(token_id: Did) -> Option<T::AccountId> {
		if let Some(token) = Self::token(token_id) {
			return Some(token.owner);
		}
		None
	}
	fn ensure_free_balance(sender: T::AccountId, token_id: Did, amount: u64) -> DispatchResult {
		Self::_ensure_free_balance(sender, token_id, amount)
	}
}
