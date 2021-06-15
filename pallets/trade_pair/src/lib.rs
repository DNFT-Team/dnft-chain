#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, StorageMap, StorageValue,
};
use frame_system::ensure_signed;
use sp_runtime::DispatchResult;
use sp_std::{
    cmp::{Eq, PartialEq},
    prelude::*,
};

use utilities::{CommonManager, Did, TokenManager, TradeMethod, TradePair, TradePairManager};

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Token: TokenManager<Self::AccountId>;
    type Common: CommonManager<Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Config> as TradePair {
        ///	TradePairId => TradePair
        TradePairs get(fn trade_pairs): map hasher(blake2_128_concat) Did => Option<TradePair>;
        /// (BaseTokenId, quoteTokenId) => TradePairId
        TradePairIdByBaseQuote get(fn trade_pair_id_by_base_quote): map hasher(blake2_128_concat) (Did, Did) => Option<Did>;
        /// Index => TradePairId
        TradePairIdByIndex get(fn trade_pair_id_by_index): map hasher(blake2_128_concat) u64 => Option<Did>;
        /// Index
        TradePairIndex get(fn trade_pair_index): u64;

        Nonce: u64;

    }
}

decl_event!(
    pub enum Event<T>
    where
        <T as frame_system::Config>::AccountId,
    {
        TradePairCreated(AccountId, Did, TradePair),
        TradePairUpdated(AccountId, Did, TradePair),
    }
);

decl_error! {
    /// Error for the trade module.
    pub enum Error for Module<T: Config> {
        /// No matching trade pair
        NoMatchingTradePair,
        /// Base equals to quote
        BaseEqualQuote,
        /// Token owner not found
        TokenOwnerNotFound,
        /// Sender not equal to base or quote owner
        SenderNotEqualToBaseOrQuoteOwner,
        /// Same trade pair with the given base and quote was already exist
        TradePairExisted,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        type Error = Error<T>;
        #[weight = 1_000_000]
        pub fn create_trade_pair(origin, base: Did, quote: Did, method: TradeMethod, matched_price: Option<u64>) {
            let sender = ensure_signed(origin)?;

            Self::_create_trade_pair(sender, base, quote, method, matched_price)?;
        }
    }
}

impl<T: Config> Module<T> {
    fn _create_trade_pair(
        sender: T::AccountId,
        base: Did,
        quote: Did,
        method: TradeMethod,
        matched_price: Option<u64>,
    ) -> Result<Did, dispatch::DispatchError> {
        ensure!(base != quote, Error::<T>::BaseEqualQuote);

        let base_owner = T::Token::owner_of(base.clone());
        let quote_owner = T::Token::owner_of(quote.clone());

        ensure!(
            base_owner.is_some() && quote_owner.is_some(),
            Error::<T>::TokenOwnerNotFound
        );

        let base_owner = base_owner.unwrap();
        let quote_owner = quote_owner.unwrap();

        ensure!(
            sender == base_owner || sender == quote_owner,
            Error::<T>::SenderNotEqualToBaseOrQuoteOwner
        );

        let bq = Self::trade_pair_id_by_base_quote((base.clone(), quote.clone()));
        let qb = Self::trade_pair_id_by_base_quote((quote.clone(), base.clone()));

        ensure!(!bq.is_some() && !qb.is_some(), Error::<T>::TradePairExisted);

        let nonce = Nonce::get();

        let tpid = T::Common::generate_did(sender.clone(), nonce.clone());

        let mut tp = TradePair {
            base: base.clone(),
            quote: quote.clone(),
            method: method.clone(),
            matched_price: Default::default(),
            one_day_trade_volume: Default::default(),
            one_day_highest_price: Default::default(),
            one_day_lowest_price: Default::default(),
        };
        if matched_price.is_some() {
            tp.matched_price = matched_price.unwrap();
        }

        Nonce::mutate(|n| *n += 1);
        <TradePairs>::insert(tpid.clone(), tp.clone());
        <TradePairIdByBaseQuote>::insert((base.clone(), quote.clone()), tpid.clone());

        let index = Self::trade_pair_index();
        <TradePairIdByIndex>::insert(index, tpid.clone());
        <TradePairIndex>::mutate(|n| *n += 1);

        Self::deposit_event(RawEvent::TradePairCreated(sender, tpid.clone(), tp));

        Ok(tpid)
    }
    pub fn _update_trade_pair(
        sender: T::AccountId,
        tpid: Did,
        new_trade_pair: TradePair,
    ) -> DispatchResult {
        ensure!(
            Self::trade_pairs(tpid.clone()).is_some(),
            Error::<T>::NoMatchingTradePair
        );
        <TradePairs>::insert(tpid.clone(), new_trade_pair.clone());
        Self::deposit_event(RawEvent::TradePairCreated(sender, tpid, new_trade_pair));
        Ok(())
    }
}

impl<T: Config> TradePairManager<T::AccountId> for Module<T> {
    // create_trade_pair
    fn create_trade_pair(
        sender: T::AccountId,
        base: Did,
        quote: Did,
        method: TradeMethod,
        matched_price: Option<u64>,
    ) -> Result<Did, dispatch::DispatchError> {
        Self::_create_trade_pair(sender, base, quote, method, matched_price)
    }

    // update_trade_pair
    fn transfer(sender: T::AccountId, tpid: Did, new_trade_pair: TradePair) -> DispatchResult {
        Self::_update_trade_pair(sender, tpid, new_trade_pair)
    }

    fn get_trade_pair(tpid: Did) -> Option<TradePair> {
        Self::trade_pairs(tpid)
    }
    fn get_trade_pair_id_by_base_quote(base: Did, quote: Did) -> Option<Did> {
        Self::trade_pair_id_by_base_quote((base, quote))
    }
}
