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

use utilities::{
    AmmOrder, CommonManager, Did, LiquidityPool, TokenManager, TradeMethod, TradePair,
    TradePairManager,
};

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Token: TokenManager<Self::AccountId>;
    type Common: CommonManager<Self::AccountId>;
    type TradePair: TradePairManager<Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Config> as Amm {

        ///	LiquidityPoolId => TradePair
        LiquidityPools get(fn liquidity_pools): map hasher(blake2_128_concat) Did => Option<LiquidityPool>;
        /// Index => LiquidityPoolId
        LiquidityPoolIdByIndex get(fn liquidity_pool_id_by_index): map hasher(blake2_128_concat) u64 => Option<Did>;
        /// Index
        LiquidityPoolIndex get(fn liquidity_pool_index): u64;
        /// LiquidityPoolId => Vec<AccountId>
        LiquidityPoolProviders get(fn liquidity_pool_providers):map hasher(blake2_128_concat) Did  => Vec<T::AccountId>;
        /// AccountId => Vec<Index>
        OwnedLiquidityPools get(fn owned_liquidity_pools):map hasher(blake2_128_concat) T::AccountId  => Vec<Did>;
        /// (AccountId,LiquidityPoolId)=> Share
        OwnedLiquidityPoolShare get(fn owned_liquidity_pool_share):map hasher(blake2_128_concat) (T::AccountId,Did)  => u64;


        /// AmmOrderId => AmmOrder
        AmmOrders get(fn amm_orders): map hasher(blake2_128_concat) Did => Option<AmmOrder<T::AccountId>>;
        /// Index => AmmOrderId
        AmmOrderIdByIndex get(fn amm_order_id_by_index): map hasher(blake2_128_concat) u64 => Option<Did>;
        /// Index
        AmmOrderIndex get(fn amm_order_index): u64;
        ///	AccountId => Vec<u64>
        OwnedAmmOrderIndex get(fn owned_amm_order_index): map hasher(blake2_128_concat) T::AccountId => Vec<u64>;
        /// Did => Vec<u64>
        LiquidityPoolsOwnedAmmOrderIndex get(fn liquidity_pools_owned_amm_order_index): map hasher(blake2_128_concat) Did => Vec<u64>;

        Nonce: u64;

    }
}

decl_event!(
    pub enum Event<T>
    where
        <T as frame_system::Config>::AccountId,
    {
        TradePairCreated(AccountId, Did, TradePair),
        LiquidityPoolInited(AccountId, Did, u64, u64),
        LiquidityAdded(AccountId, Did, u64),
        TradeDoned(AccountId, Did, Did, u64),
        LiquidityRemoved(AccountId, Did, u64),
    }
);

decl_error! {
    /// Error for the trade module.
    pub enum Error for Module<T: Config> {
        /// No matching trade pair
        NoMatchingTradePair,
        /// TradePairErr
        TradePairErr,
        /// No matching Liquidity Pool
        NoMatchingLiquidityPool,
        /// Balance Is Not Enough
        BalanceIsNotEnough,
        /// LPShare Is Not Enough
        LPShareIsNotEnough,
        /// Liquidity Pool Token Is Not Enough
        LiquidityPoolTokenIsNotEnough,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        type Error = Error<T>;
        #[weight = 1_000_000]
        pub fn init_liquidity_pool(origin, tp_id: Did, base_amount: u64, quote_amount: u64){
            let sender = ensure_signed(origin)?;

            Self::_init_liquidity_pool(sender, tp_id, base_amount, quote_amount)?;
        }
        #[weight = 1_000_000]
        pub fn add_liquidity(origin, lpid: Did, liquidity_share: u64){
            let sender = ensure_signed(origin)?;

            Self::_add_liquidity(sender, lpid, liquidity_share)?;
        }
        #[weight = 1_000_000]
        pub fn trade(origin, lpid: Did,  token_have: Did, trade_amount: u64, token_want: Did){
            let sender = ensure_signed(origin)?;
            Self::_trade(sender, lpid, token_have, trade_amount, token_want)?;
        }
        #[weight = 1_000_000]
        pub fn remove_liquidity(origin, tp_id: Did, lp_share: u64) {
            let sender = ensure_signed(origin)?;
            Self::do_remove_liquidity(sender, tp_id, lp_share)?;
        }

    }
}

impl<T: Config> Module<T> {
    fn _init_liquidity_pool(
        sender: T::AccountId,
        tpid: Did,
        base_amount: u64,
        quote_amount: u64,
    ) -> DispatchResult {
        let trade_pair_raw = T::TradePair::get_trade_pair(tpid.clone());

        ensure!(trade_pair_raw.is_some(), Error::<T>::NoMatchingTradePair);

        let trade_pair = trade_pair_raw.unwrap().clone();
        ensure!(
            trade_pair.method == TradeMethod::AMMOrder,
            Error::<T>::TradePairErr
        );

        let nonce = Nonce::get();

        let lpid = T::Common::generate_did(sender.clone(), nonce.clone());

        let lp = LiquidityPool {
            tpid: tpid.clone(),
            token0: trade_pair.base.clone(),
            token1: trade_pair.quote.clone(),
            token0_amount: base_amount.clone(),
            token1_amount: quote_amount.clone(),
            k_last: base_amount.clone() * quote_amount.clone(),
            swap_price_last: base_amount.clone() / quote_amount.clone(),
            swap_price_highest: base_amount.clone() / quote_amount.clone(),
            swap_price_lowest: base_amount.clone() / quote_amount.clone(),
            token0_trade_volume_total: Default::default(),
            token1_trade_volume_total: Default::default(),
        };

        Nonce::mutate(|n| *n += 1);
        <LiquidityPools>::insert(lpid.clone(), lp.clone());

        let index = Self::liquidity_pool_index();
        <LiquidityPoolIdByIndex>::insert(index.clone(), tpid.clone());
        <LiquidityPoolIndex>::mutate(|n| *n += 1);

        Self::_add_liquidity_providers(lpid.clone(), sender.clone());
        Self::_add_owned_liquidity_pools(sender.clone(), lpid.clone());
        Self::_add_owned_liquidity_pools_share(
            sender.clone(),
            lpid.clone(),
            base_amount.clone() / lp.swap_price_last.clone(),
        );

        Self::deposit_event(RawEvent::LiquidityPoolInited(
            sender,
            tpid,
            base_amount,
            quote_amount,
        ));

        Ok(())
    }

    fn _add_liquidity(sender: T::AccountId, lpid: Did, liquidity_share: u64) -> DispatchResult {
        let liquidity_pool_raw = Self::liquidity_pools(lpid.clone());
        ensure!(
            liquidity_pool_raw.is_some(),
            Error::<T>::NoMatchingLiquidityPool
        );
        let mut liquidity_pool = liquidity_pool_raw.unwrap().clone();

        let token0_balance = T::Token::balance_of(sender.clone(), liquidity_pool.token0.clone());
        let token1_balance = T::Token::balance_of(sender.clone(), liquidity_pool.token1.clone());

        let tolen0_need = liquidity_pool.swap_price_last.clone() * liquidity_share.clone();
        let tolen1_need = liquidity_share.clone();

        ensure!(
            token0_balance >= tolen0_need,
            Error::<T>::BalanceIsNotEnough
        );
        ensure!(
            token1_balance >= tolen1_need,
            Error::<T>::BalanceIsNotEnough
        );

        T::Token::static_transfer_in(
            sender.clone(),
            lpid.clone(),
            liquidity_pool.token0.clone(),
            tolen0_need.clone(),
        )?;
        T::Token::static_transfer_in(
            sender.clone(),
            lpid.clone(),
            liquidity_pool.token1.clone(),
            tolen1_need.clone(),
        )?;

        liquidity_pool.token0_amount += tolen0_need.clone();
        liquidity_pool.token1_amount += tolen1_need.clone();
        liquidity_pool.k_last +=
            liquidity_pool.token0_amount.clone() * liquidity_pool.token1_amount.clone();

        Self::_update_liquidity_pool(lpid.clone(), liquidity_pool.clone())?;
        Self::_add_liquidity_providers(lpid.clone(), sender.clone());
        Self::_add_owned_liquidity_pools(sender.clone(), lpid.clone());
        Self::_add_owned_liquidity_pools_share(
            sender.clone(),
            lpid.clone(),
            liquidity_share.clone(),
        );

        Self::deposit_event(RawEvent::LiquidityAdded(sender, lpid, liquidity_share));

        Ok(())
    }

    fn _trade(
        sender: T::AccountId,
        lpid: Did,
        token_have: Did,
        token_have_amount: u64,
        token_want: Did,
    ) -> DispatchResult {
        let liquidity_pool_raw = Self::liquidity_pools(lpid.clone());
        ensure!(
            liquidity_pool_raw.is_some(),
            Error::<T>::NoMatchingLiquidityPool
        );
        let mut liquidity_pool = liquidity_pool_raw.unwrap();
        let swap_price_old = liquidity_pool.swap_price_last.clone();
        let tpid =
            T::TradePair::get_trade_pair_id_by_base_quote(token_have.clone(), token_want.clone());
        let tpid1 =
            T::TradePair::get_trade_pair_id_by_base_quote(token_want.clone(), token_have.clone());
        ensure!(
            (tpid.is_some() && tpid.unwrap() == liquidity_pool.tpid)
                || (tpid1.is_some() && tpid1.unwrap() == liquidity_pool.tpid),
            Error::<T>::NoMatchingLiquidityPool
        );
        let mut token_want_amount: u64;
        if token_have == liquidity_pool.token0 {
            ensure!(
                liquidity_pool.token0_amount > token_have_amount,
                Error::<T>::LiquidityPoolTokenIsNotEnough
            );
            token_want_amount = liquidity_pool.token1_amount.clone();
            liquidity_pool.token0_amount += token_have_amount.clone();
            liquidity_pool.token1_amount = liquidity_pool.k_last
                / (liquidity_pool.token0_amount.clone() - token_have_amount.clone() * 30);
            token_want_amount -= liquidity_pool.token1_amount;
            liquidity_pool.k_last =
                liquidity_pool.token0_amount.clone() * liquidity_pool.token1_amount.clone();
            liquidity_pool.swap_price_last = token_have_amount.clone() / token_want_amount.clone();
            liquidity_pool.token0_trade_volume_total += token_have_amount.clone();
            liquidity_pool.token1_trade_volume_total += token_want_amount.clone();
        } else {
            ensure!(
                liquidity_pool.token1_amount > token_have_amount,
                Error::<T>::LiquidityPoolTokenIsNotEnough
            );
            token_want_amount = liquidity_pool.token0_amount.clone();
            liquidity_pool.token1_amount += token_have_amount.clone();
            liquidity_pool.token0_amount = liquidity_pool.k_last
                / (liquidity_pool.token1_amount.clone() - token_have_amount.clone() * 30);
            token_want_amount -= liquidity_pool.token0_amount;
            liquidity_pool.k_last =
                liquidity_pool.token0_amount.clone() * liquidity_pool.token1_amount.clone();
            liquidity_pool.swap_price_last = token_want_amount.clone() / token_have_amount.clone();
            liquidity_pool.token0_trade_volume_total += token_want_amount.clone();
            liquidity_pool.token1_trade_volume_total += token_have_amount.clone();
        }
        if liquidity_pool.swap_price_last >= swap_price_old {
            liquidity_pool.swap_price_highest = liquidity_pool.swap_price_last.clone();
        } else {
            liquidity_pool.swap_price_lowest = liquidity_pool.swap_price_last.clone();
        }
        Self::_update_liquidity_pool(lpid.clone(), liquidity_pool.clone())?;

        Self::deposit_event(RawEvent::TradeDoned(
            sender,
            lpid,
            token_have,
            token_have_amount,
        ));

        Ok(())
    }

    fn do_remove_liquidity(
        sender: T::AccountId,
        lpid: Did,
        liquidity_share: u64,
    ) -> DispatchResult {
        let liquidity_pool_raw = Self::liquidity_pools(lpid.clone());
        ensure!(
            liquidity_pool_raw.is_some(),
            Error::<T>::NoMatchingLiquidityPool
        );
        let mut liquidity_pool = liquidity_pool_raw.unwrap();

        let owned_liquidity_share =
            Self::owned_liquidity_pool_share((sender.clone(), lpid.clone()));

        ensure!(
            owned_liquidity_share >= liquidity_share,
            Error::<T>::LPShareIsNotEnough
        );

        let tolen0_free = liquidity_pool.swap_price_last.clone() * liquidity_share.clone();
        let tolen1_free = liquidity_share.clone();

        T::Token::static_transfer_out(
            lpid.clone(),
            sender.clone(),
            liquidity_pool.token0.clone(),
            tolen0_free.clone(),
        )?;
        T::Token::static_transfer_out(
            lpid.clone(),
            sender.clone(),
            liquidity_pool.token1.clone(),
            tolen1_free.clone(),
        )?;

        liquidity_pool.token0_amount -= tolen0_free.clone();
        liquidity_pool.token1_amount -= tolen1_free.clone();
        liquidity_pool.k_last +=
            liquidity_pool.token0_amount.clone() * liquidity_pool.token1_amount.clone();

        Self::_update_liquidity_pool(lpid.clone(), liquidity_pool.clone())?;
        Self::_add_liquidity_providers(lpid.clone(), sender.clone());
        Self::_add_owned_liquidity_pools(sender.clone(), lpid.clone());
        Self::_remove_owned_liquidity_pools_share(
            sender.clone(),
            lpid.clone(),
            liquidity_share.clone(),
        );

        Self::deposit_event(RawEvent::LiquidityRemoved(sender, lpid, liquidity_share));

        Ok(())
    }
}

impl<T: Config> Module<T> {
    fn _create_amm_order(
        lpid: Did,
        sender: T::AccountId,
        token_have: Did,
        token_have_amount: u64,
        token_want: Did,
        token_want_amount: u64,
        token_swap_price: u64,
    ) -> DispatchResult {
        let nonce = Nonce::get();

        let aorder_id = T::Common::generate_did(sender.clone(), nonce.clone());

        let aorder = AmmOrder {
            lpid: lpid.clone(),
            owner: sender.clone(),
            token_have: token_have.clone(),
            token_have_amount: token_have_amount.clone(),
            token_want: token_want.clone(),
            token_want_amount: token_want_amount.clone(),
            token_swap_price: token_swap_price.clone(),
        };

        Nonce::mutate(|n| *n += 1);
        <AmmOrders<T>>::insert(aorder_id.clone(), aorder.clone());

        let index = Self::amm_order_index();
        <AmmOrderIdByIndex>::insert(index, aorder_id.clone());
        <AmmOrderIndex>::mutate(|n| *n += 1);
        Self::_add_owned_amm_order_index(sender.clone(), index.clone());
        Self::_add_liquidity_pools_owned_amm_order_index(lpid.clone(), index.clone());

        Ok(())
    }
}
impl<T: Config> Module<T> {
    pub fn _update_liquidity_pool(lpid: Did, new_liquidity_pool: LiquidityPool) -> DispatchResult {
        ensure!(
            Self::liquidity_pools(lpid.clone()).is_some(),
            Error::<T>::NoMatchingLiquidityPool
        );
        <LiquidityPools>::insert(lpid, new_liquidity_pool);
        Ok(())
    }
}

impl<T: Config> Module<T> {
    //LiquidityPoolProviders
    fn _add_liquidity_providers(lpid: Did, sender: T::AccountId) {
        let mut lps = Self::liquidity_pool_providers(lpid.clone());
        for i in 0..lps.len() {
            if lps[i] == sender {
                return;
            }
        }
        lps.push(sender.clone());
        <LiquidityPoolProviders<T>>::insert(lpid, lps);
    }

    fn _remove_liquidity_providers(lpid: Did, sender: T::AccountId) {
        let lps = Self::liquidity_pool_providers(lpid.clone());
        let mut lps1 = lps.clone();
        for i in 0..lps.len() {
            if lps[i] == sender {
                lps1.remove(i);
            }
        }
        <LiquidityPoolProviders<T>>::insert(lpid, lps1);
    }

    //OwnedLiquidityPools
    fn _add_owned_liquidity_pools(sender: T::AccountId, lpid: Did) {
        let mut lps = Self::owned_liquidity_pools(sender.clone());
        for i in 0..lps.len() {
            if lps[i] == lpid {
                return;
            }
        }
        lps.push(lpid.clone());
        <OwnedLiquidityPools<T>>::insert(sender, lps);
    }

    fn _remove_owned_liquidity_pools(sender: T::AccountId, lpid: Did) {
        let lp = Self::owned_liquidity_pools(sender.clone());
        let mut lp1 = lp.clone();
        for i in 0..lp.len() {
            if lp[i] == lpid {
                lp1.remove(i);
            }
        }
        <OwnedLiquidityPools<T>>::insert(sender, lp1);
    }

    //OwnedLiquidityPoolShares
    fn _add_owned_liquidity_pools_share(sender: T::AccountId, lpid: Did, share: u64) {
        let mut shares = Self::owned_liquidity_pool_share((sender.clone(), lpid.clone()));
        shares += share;
        <OwnedLiquidityPoolShare<T>>::insert((sender, lpid), shares);
    }

    fn _remove_owned_liquidity_pools_share(sender: T::AccountId, lpid: Did, share: u64) {
        let mut shares = Self::owned_liquidity_pool_share((sender.clone(), lpid.clone()));
        shares -= share;
        <OwnedLiquidityPoolShare<T>>::insert((sender, lpid), shares);
    }
    //OwnedAmmOrderIndex
    fn _add_owned_amm_order_index(sender: T::AccountId, index: u64) {
        let mut indexs = Self::owned_amm_order_index(sender.clone());
        for i in 0..indexs.len() {
            if indexs[i] == index {
                return;
            }
        }
        indexs.push(index.clone());
        <OwnedAmmOrderIndex<T>>::insert(sender, indexs);
    }
    //LiquidityPoolsOwnedAmmOrderIndex
    fn _add_liquidity_pools_owned_amm_order_index(lpid: Did, index: u64) {
        let mut indexs = Self::liquidity_pools_owned_amm_order_index(lpid.clone());
        for i in 0..indexs.len() {
            if indexs[i] == index {
                return;
            }
        }
        indexs.push(index.clone());
        <LiquidityPoolsOwnedAmmOrderIndex>::insert(lpid, indexs);
    }
}
