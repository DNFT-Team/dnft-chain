#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]
#![allow(clippy::string_lit_as_bytes)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::Vec, ensure, traits::Time,
    StorageMap, StorageValue,
};
use frame_system::ensure_signed;
use sp_runtime::DispatchResult;
use sp_std::{
    cmp::{Eq, PartialEq},
    prelude::*,
};
use utilities::{
    BufferIndex, CommonManager, Did, LimitOrder, OrderQueueInfo, OrderStatus, OrderType,
    TokenManager, TradePairManager, ValueStruct,
};

mod ringbuffer;

use ringbuffer::{RingBufferTrait, RingBufferTransient};

type MomentOf<T> = <<T as Config>::Time as Time>::Moment;

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Time: Time;
    type Token: TokenManager<Self::AccountId>;
    type Common: CommonManager<Self::AccountId>;
    type TradePair: TradePairManager<Self::AccountId>;
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
    {

        LimitOrderCreated(AccountId, Did, OrderType, u64, u64),

        OrderCanceled(AccountId, Did),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        BoundsCheckFailed,
        PriceLengthCheckFailed,
        NumberCastError,
        OverflowError,
        NoMatchingTradePair,
        BaseEqualQuote,
        TokenOwnerNotFound,
        SenderNotEqualToBaseOrQuoteOwner,
        TradePairExisted,
        OrderMatchGetPriceError,
        OrderMatchGetLinkedListItemError,
        OrderMatchGetOrderError,
        OrderMatchSubstractError,
        OrderMatchOrderIsNotFinished,
        NoMatchingOrder,
        CanOnlyCancelOwnOrder,
        CanOnlyCancelNotFinishedOrder,
    }
}

decl_storage! {
    trait Store for Module<T: Config> as OrderBook {

        /// OrderId => Order
        pub Orders get(fn order): map hasher(blake2_128_concat) Did => Option<LimitOrder<T::AccountId, MomentOf<T>>>;
        /// Index => OrderId
        pub OrderIdByIndex get(fn order_id_by_index): map hasher(blake2_128_concat) u32 => Option<Did>;
        /// Index
        pub OrderIndex get(fn order_index): u32;
        /// (AccoundId, Index) => OrderId
        pub OwnedOrders get(fn owned_order): map hasher(blake2_128_concat) (T::AccountId, u64) => Option<Did>;
        ///	AccountId => Index
        pub OwnedOrdersIndex get(fn owned_orders_index): map hasher(blake2_128_concat) T::AccountId => u64;
        /// (OrderId, u64) => TradeId
        pub OrderOwnedTrades get(fn order_owned_trades): map hasher(blake2_128_concat) (Did, u64) => Option<Did>;
        /// (OrderId, u64) => TradeId
        pub OrderOwnedTradesIndex get(fn order_owned_trades_index): map hasher(blake2_128_concat) Did => u64;


        /// (AccountId, TradePairHash) => Vec<OrderId>
        pub OwnedTPOpenedOrders get(fn owned_tp_opened_orders): map hasher(blake2_128_concat) (T::AccountId, Did) => Vec<Did>;

        /// (AccountId, TradePairHash) => Vec<OrderId>
        pub OwnedTPClosedOrders get(fn owned_tp_closed_orders): map hasher(blake2_128_concat) (T::AccountId, Did) => Vec<Did>;

        /// (TradePairId, Index) => OrderId
        pub TradePairOwnedOrders get(fn trade_pair_owned_order): map hasher(blake2_128_concat) (Did, u64) => Option<Did>;
        /// TradePairId => Index
        pub TradePairOwnedOrdersIndex get(fn trade_pair_owned_order_index): map hasher(blake2_128_concat) Did => u64;


        pub BuyOrderBufferMap get(fn get_buy_order_value): map hasher(twox_64_concat) BufferIndex => ValueStruct;
        pub BuyOrderBufferRange get(fn buy_order_range): (BufferIndex, BufferIndex) = (0, 0);
        pub SellOrderBufferMap get(fn get_sell_order_value): map hasher(twox_64_concat) BufferIndex => ValueStruct;
        pub SellOrderBufferRange get(fn sell_order_range): (BufferIndex, BufferIndex) = (0, 0);

        pub BuyOrderBook get(fn buy_order_book): Vec<OrderQueueInfo>;
        pub SellOrderBook get(fn sell_order_book): Vec<OrderQueueInfo>;
        pub Nonce: u64;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;
       #[weight = 10_000]
        pub fn create_limit_order(origin, tpid: Did, otype: OrderType, price: u64, sell_amount: u64)  {
            let sender = ensure_signed(origin)?;

            Self::_create_limit_order(sender, tpid, otype, price, sell_amount)?;
        }

        #[weight = 10_000]
        pub fn cancel_limit_order(origin, order_id: Did) {
            let sender = ensure_signed(origin)?;

            Self::_cancel_limit_order(sender, order_id)?;
        }

        // fn on_finalize() {        }
    }
}
/// new
impl<T: Config> Module<T> {
    fn _create_limit_order(
        sender: T::AccountId,
        tpid: Did,
        otype: OrderType,
        price: u64,
        amount: u64,
    ) -> DispatchResult {
        let trade_pair_raw = T::TradePair::get_trade_pair(tpid.clone());
        ensure!(trade_pair_raw.is_some(), Error::<T>::NoMatchingTradePair);
        let trade_pair = trade_pair_raw.unwrap().clone();

        Self::_ensure_bounds_of_limit_order_create(
            sender.clone(),
            trade_pair.quote.clone(),
            price.clone(),
            amount.clone(),
        )?;
        let nonce = Nonce::get();
        let new_order_id = T::Common::generate_did(sender.clone(), nonce.clone());
        let now = T::Time::now();

        let new_order = LimitOrder {
            tpid: tpid.clone(),
            owner: sender.clone(),
            price: price.clone(),
            amount: amount.clone(),
            created_time: now.clone(),
            remained_amount: Default::default(),
            otype: otype.clone(),
            status: OrderStatus::Created,
        };

        <Orders<T>>::insert(new_order_id.clone(), new_order.clone());
        Nonce::mutate(|n| *n += 1);
        let index = Self::order_index();
        <OrderIdByIndex>::insert(index.clone(), new_order_id.clone());
        <OrderIndex>::mutate(|n| *n += 1);

        Self::deposit_event(RawEvent::LimitOrderCreated(
            sender.clone(),
            tpid.clone(),
            otype.clone(),
            price.clone(),
            amount.clone(),
        ));
        match otype {
            OrderType::Sell => Self::_add_to_sell_order_book(index.clone()),
            OrderType::Buy => Self::_add_to_buy_order_book(index.clone()),
        }
        // Self::_add_to_owned_order(sender.clone(), new_order_id.clone());
        Self::_add_to_owned_tp_opened_order(sender.clone(), tpid.clone(), new_order_id.clone());
        Self::_add_to_tp_owned_order(tpid.clone(), new_order_id.clone());

        Ok(())
    }

    fn _cancel_limit_order(sender: T::AccountId, order_id: Did) -> DispatchResult {
        let mut order = Self::order(order_id.clone()).ok_or(Error::<T>::NoMatchingOrder)?;

        Self::_ensure_bounds_of_limit_order_cancel(sender.clone(), order.clone())?;

        let trade_pair_raw = T::TradePair::get_trade_pair(order.tpid.clone());
        ensure!(trade_pair_raw.is_some(), Error::<T>::NoMatchingTradePair);
        let trade_pair = trade_pair_raw.unwrap().clone();

        order.status = OrderStatus::Canceled;
        <Orders<T>>::insert(order_id.clone(), order.clone());

        Self::_remove_from_owned_tp_opened_order(
            sender.clone(),
            order.tpid.clone(),
            order_id.clone(),
        );
        Self::_add_to_owned_tp_closed_order(sender.clone(), order.tpid.clone(), order_id.clone());

        T::Token::unfreeze(
            sender.clone(),
            trade_pair.quote.clone(),
            order.remained_amount,
        )?;

        Self::deposit_event(RawEvent::OrderCanceled(sender, order_id));

        Ok(())
    }
}
/// check
impl<T: Config> Module<T> {
    /// param bounds check
    fn _ensure_bounds_of_limit_order_create(
        sender: T::AccountId,
        quote: Did,
        price: u64,
        amount: u64,
    ) -> DispatchResult {
        ensure!(
            price > 0 && price <= u64::max_value(),
            Error::<T>::BoundsCheckFailed
        );
        ensure!(
            amount > 0 && amount <= u64::max_value(),
            Error::<T>::BoundsCheckFailed
        );
        let balance = T::Token::balance_of(sender.clone(), quote.clone());
        ensure!(balance >= amount * price, Error::<T>::BoundsCheckFailed);
        T::Token::ensure_free_balance(sender.clone(), quote.clone(), amount.clone())?;
        T::Token::freeze(sender.clone(), quote.clone(), amount.clone())?;
        Ok(())
    }
    ///param bounds check
    fn _ensure_bounds_of_limit_order_cancel(
        sender: T::AccountId,
        order: LimitOrder<T::AccountId, MomentOf<T>>,
    ) -> DispatchResult {
        ensure!(order.owner == sender, Error::<T>::CanOnlyCancelOwnOrder);

        ensure!(
            !Self::is_limit_order_finished(order.clone()),
            Error::<T>::CanOnlyCancelNotFinishedOrder
        );
        Ok(())
    }
    pub fn is_limit_order_finished(order: LimitOrder<T::AccountId, MomentOf<T>>) -> bool {
        (order.remained_amount == 0 && order.status == OrderStatus::Filled)
            || order.status == OrderStatus::Canceled
    }
}
/// order match engine
impl<T: Config> Module<T> {
    fn _limit_order_book_match_engine() {
        // let buy_order_lastest_raw = Self::_get_oldest_value_from_buy_order_book();
        // ensure!(
        //     buy_order_lastest_raw.is_some(),
        //     Error::<T>::NoMatchingTradePair
        // );
        // let trade_pair = buy_order_lastest_raw.unwrap().clone();
        // let sell_order_lastest_raw = Self::_get_oldest_value_from_sell_order_book();
    }
}
// /// buy order queue
// impl<T: Trait> Module<T> {
//     fn _add_one_to_buy_queue(price: u32, time: u32, volume: u32, order_index: u32) {
//         let mut buy_queue = Self::buy_order_book();
//         let new_order = OrderQueueInfo {
//             oindex: order_index.clone(),
//             price: price.clone(),
//             time: time.clone(),
//             volume: volume.clone(),
//         };
//         buy_queue.push(new_order);
//     }
//     fn _get_one_from_buy_queue() -> Option<OrderQueueInfo> {}
//     fn _update_one_from_buy_queue(price: u32, volume: u32, order_index: u32) {}
//     fn _remove_one_from_buy_queue() -> Option<u32> {}
//     fn _remove_many_from_buy_queue() -> Option<u32> {}
//     fn _match_from_buy_queue(volume: u32, price: u32) -> Vec<u32> {}
// }
// /// sell order queue
// impl<T: Trait> Module<T> {
//     fn _add_one_to_sell_queue(price: u32, time: u32, volume: u32, order_index: u32) {}
//     fn _get_one_from_sell_queue() -> Option<OrderQueueInfo> {}
//     fn _update_one_from_sell_queue(price: u32, volume: u32, order_index: u32) {}
//     fn _remove_one_from_sell_queue() -> Option<u32> {}
//     fn _remove_many_from_sell_queue() -> Option<u32> {}
//     fn _match_from_sell_queue(volume: u32, price: u32) -> Vec<u32> {}
// }
/// buy order book
impl<T: Config> Module<T> {
    fn _add_to_buy_order_book(integer: u32) {
        let boolean = true;
        let mut queue = Self::_buy_order_queue_transient();
        queue.push(ValueStruct { integer, boolean });
    }
    fn _add_multiple_buy_order(integers: Vec<u32>) {
        let boolean = true;
        let mut queue = Self::_buy_order_queue_transient();
        for integer in integers {
            queue.push(ValueStruct { integer, boolean });
        }
    }
    fn _remove_from_buy_order_book() {
        let mut queue = Self::_buy_order_queue_transient();
        queue.pop();
    }
    fn _get_oldest_value_from_buy_order_book() -> Option<u32> {
        let (start, end) = Self::buy_order_range();
        if start != end {
            let value = Self::get_buy_order_value(start.clone());
            if value.boolean == true {
                return Some(value.integer);
            }
        }
        None
    }
    /// Constructor function so we don't have to specify the types every time.
    ///
    /// Constructs a ringbuffer transient and returns it as a boxed trait object.
    /// See [this part of the Rust book](https://doc.rust-lang.org/book/ch17-02-trait-objects.html#trait-objects-perform-dynamic-dispatch)
    fn _buy_order_queue_transient() -> Box<dyn RingBufferTrait<ValueStruct>> {
        Box::new(RingBufferTransient::<
            ValueStruct,
            <Self as Store>::BuyOrderBufferRange,
            <Self as Store>::BuyOrderBufferMap,
            BufferIndex,
        >::new())
    }
}
/// sell order book
impl<T: Config> Module<T> {
    fn _add_to_sell_order_book(integer: u32) {
        let boolean = true;
        let mut queue = Self::_sell_order_queue_transient();
        queue.push(ValueStruct { integer, boolean });
    }
    fn _add_multiple_sell_order(integers: Vec<u32>) {
        let boolean = true;
        let mut queue = Self::_sell_order_queue_transient();
        for integer in integers {
            queue.push(ValueStruct { integer, boolean });
        }
    }
    fn _remove_from_sell_order_book() {
        let mut queue = Self::_sell_order_queue_transient();
        queue.pop();
    }
    fn _get_oldest_value_from_sell_order_book() -> Option<u32> {
        let (start, end) = Self::sell_order_range();
        if start != end {
            let value = Self::get_sell_order_value(start.clone());
            if value.boolean == true {
                return Some(value.integer);
            }
        }
        None
    }
    /// Constructor function so we don't have to specify the types every time.
    ///
    /// Constructs a ringbuffer transient and returns it as a boxed trait object.
    /// See [this part of the Rust book](https://doc.rust-lang.org/book/ch17-02-trait-objects.html#trait-objects-perform-dynamic-dispatch)
    fn _sell_order_queue_transient() -> Box<dyn RingBufferTrait<ValueStruct>> {
        Box::new(RingBufferTransient::<
            ValueStruct,
            <Self as Store>::SellOrderBufferRange,
            <Self as Store>::SellOrderBufferMap,
            BufferIndex,
        >::new())
    }
}

///
impl<T: Config> Module<T> {
    //OwnedTPOpenedOrders
    fn _add_to_owned_tp_opened_order(account_id: T::AccountId, tpid: Did, order_id: Did) {
        let mut ts = Self::owned_tp_opened_orders((account_id.clone(), tpid.clone()));
        for i in 0..ts.len() {
            if ts[i] == order_id {
                return;
            }
        }
        ts.insert(0, order_id.clone());

        <OwnedTPOpenedOrders<T>>::insert((account_id, tpid), ts);
    }

    fn _remove_from_owned_tp_opened_order(account_id: T::AccountId, tpid: Did, order_id: Did) {
        let ts = Self::owned_tp_opened_orders((account_id.clone(), tpid.clone()));
        let mut ts1 = ts.clone();
        for i in 0..ts.len() {
            if ts[i] == order_id {
                ts1.remove(i);
            }
        }
        <OwnedTPOpenedOrders<T>>::insert((account_id, tpid), ts1);
    }

    ///OwnedOrders
    fn _add_to_owned_order(sender: T::AccountId, order_id: Did) {
        let owned_index = Self::owned_orders_index(sender.clone());
        <OwnedOrders<T>>::insert((sender.clone(), owned_index.clone()), order_id.clone());
        <OwnedOrdersIndex<T>>::insert(sender.clone(), owned_index.clone() + 1);
    }

    ///TradePairOwnedOrders
    fn _add_to_tp_owned_order(tpid: Did, order_id: Did) {
        let tp_owned_index = Self::trade_pair_owned_order_index(tpid.clone());
        TradePairOwnedOrders::insert((tpid.clone(), tp_owned_index.clone()), order_id.clone());
        TradePairOwnedOrdersIndex::insert(tpid.clone(), tp_owned_index.clone() + 1);
    }

    //OwnedTPClosedOrders
    fn _add_to_owned_tp_closed_order(account_id: T::AccountId, tpid: Did, order_id: Did) {
        let mut ts = Self::owned_tp_closed_orders((account_id.clone(), tpid.clone()));
        for i in 0..ts.len() {
            if ts[i] == order_id {
                return;
            }
        }
        ts.insert(0, order_id);

        <OwnedTPClosedOrders<T>>::insert((account_id, tpid), ts);
    }
}
