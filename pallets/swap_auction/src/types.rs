use codec::{Decode, Encode};
use frame_support::{ensure, StorageMap};
use sp_runtime::DispatchResult as Result;
use sp_std::prelude::*;

pub use crate as amm;
type OrderType = amm::OrderType;
use support::Did;

#[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(PartialEq, Eq, Debug))]
pub struct LinkedItem {
    pub prev: Option<u64>,
    pub next: Option<u64>,
    pub price: Option<u64>,
    pub buy_amount: u64,
    pub sell_amount: u64,
    pub orders: Vec<Did>, // remove the item at 0 index will caused performance issue, should be optimized
}

pub struct LinkedList<T, S>(sp_std::marker::PhantomData<(T, S)>);

///             LinkedItem          LinkedItem			LinkedItem          LinkedItem          LinkedItem
///             Bottom              Buy Order			Head                Sell Order          Top
///   			Next	    ---->   Price: 8	<----	Prev                Next       ---->    Price: max
///   max <---- Prev				Next		---->	Price:None  <----   Prev                Next        ---->   Price: 0
///         	Price:0		<----   Prev     			Next        ---->   Price 10   <----    Prev
///                                 Orders									Orders
///                                 o1: Hash -> buy 1@8						o101: Hash -> sell 100@10
///                                 o2: Hash -> buy 5@8						o102: Hash -> sell 100@10
///                                 o3: Hash -> buy 100@8
///                                 o4: Hash -> buy 40@8
///                                 o5: Hash -> buy 1000@8
///
/// when do order matching, o1 will match before o2 and so on

// Self: StorageMap, Key1: TradePairHash, Key2: Price, Value: OrderHash
impl<T, S> LinkedList<T, S>
where
    T: amm::Trait,
    S: StorageMap<(Did, Option<u64>), LinkedItem, Query = Option<LinkedItem>>,
{
    pub fn read_head(key: Did) -> LinkedItem {
        Self::read(key, None)
    }

    #[allow(dead_code)]
    pub fn read_bottom(key: Did) -> LinkedItem {
        Self::read(key, Some(u64::min_value()))
    }

    #[allow(dead_code)]
    pub fn read_top(key: Did) -> LinkedItem {
        Self::read(key, Some(u64::max_value()))
    }

    pub fn read(key1: Did, key2: Option<u64>) -> LinkedItem {
        S::get((key1.clone(), key2.clone())).unwrap_or_else(|| {
            let bottom = LinkedItem {
                prev: Some(u64::max_value()),
                next: None,
                price: Some(u64::min_value()),
                orders: Vec::<Did>::new(),
                buy_amount: Default::default(),
                sell_amount: Default::default(),
            };

            let top = LinkedItem {
                prev: None,
                next: Some(u64::min_value()),
                price: Some(u64::max_value()),
                orders: Vec::<Did>::new(),
                buy_amount: Default::default(),
                sell_amount: Default::default(),
            };

            let head = LinkedItem {
                prev: Some(u64::min_value()),
                next: Some(u64::max_value()),
                price: None,
                orders: Vec::<Did>::new(),
                buy_amount: Default::default(),
                sell_amount: Default::default(),
            };

            Self::write(key1.clone(), bottom.price, bottom);
            Self::write(key1.clone(), top.price, top);
            Self::write(key1, head.price, head.clone());
            head
        })
    }

    pub fn write(key1: Did, key2: Option<u64>, item: LinkedItem) {
        S::insert((key1, key2), item);
    }

    pub fn append(
        key1: Did,
        key2: u64,
        value: Did,
        sell_amount: u64,
        buy_amount: u64,
        otype: OrderType,
    ) {
        let item = S::get((key1.clone(), Some(key2.clone())));
        match item {
            Some(mut item) => {
                item.orders.push(value);
                item.buy_amount = item.buy_amount + buy_amount;
                item.sell_amount = item.sell_amount + sell_amount;
                Self::write(key1.clone(), Some(key2.clone()), item);
                return;
            }
            None => {
                let start_item;
                let end_item;

                match otype {
                    OrderType::Buy => {
                        start_item = Some(u64::min_value());
                        end_item = None;
                    }
                    OrderType::Sell => {
                        start_item = None;
                        end_item = Some(u64::max_value());
                    }
                }

                let mut item = Self::read(key1.clone(), start_item);

                while item.next != end_item {
                    match item.next {
                        None => {}
                        Some(price) => {
                            if key2 < price {
                                break;
                            }
                        }
                    }

                    item = Self::read(key1.clone(), item.next);
                }

                // update new_prev
                let new_prev = LinkedItem {
                    next: Some(key2.clone()),
                    ..item
                };
                Self::write(key1.clone(), new_prev.price, new_prev.clone());

                // update new_next
                let next = Self::read(key1.clone(), item.next);
                let new_next = LinkedItem {
                    prev: Some(key2.clone()),
                    ..next
                };
                Self::write(key1.clone(), new_next.price, new_next.clone());

                // update key2
                let mut v = Vec::new();
                v.push(value);
                let item = LinkedItem {
                    prev: new_prev.price,
                    next: new_next.price,
                    buy_amount,
                    sell_amount,
                    orders: v,
                    price: Some(key2.clone()),
                };
                Self::write(key1, Some(key2), item);
            }
        };
    }

    pub fn next_match_price(item: &LinkedItem, otype: OrderType) -> Option<u64> {
        if otype == OrderType::Buy {
            item.prev
        } else {
            item.next
        }
    }

    pub fn update_amount(key1: Did, key2: u64, sell_amount: u64, buy_amount: u64) {
        let mut item = Self::read(key1.clone(), Some(key2));
        item.buy_amount = item.buy_amount - buy_amount;
        item.sell_amount = item.sell_amount - sell_amount;
        Self::write(key1, Some(key2), item);
    }

    pub fn remove_all(key1: Did, otype: OrderType) {
        let end_item;

        if otype == OrderType::Buy {
            end_item = Some(u64::min_value());
        } else {
            end_item = Some(u64::max_value());
        }

        let mut head = Self::read_head(key1.clone());

        loop {
            let key2 = Self::next_match_price(&head, otype);
            if key2 == end_item {
                break;
            }

            match Self::remove_orders_in_one_item(key1.clone(), key2.unwrap()) {
                Err(_) => break,
                _ => {}
            };

            head = Self::read_head(key1.clone());
        }
    }

    pub fn remove_order(
        key1: Did,
        key2: u64,
        order_id: Did,
        sell_amount: u64,
        buy_amount: u64,
    ) -> Result {
        match S::get((key1.clone(), Some(key2.clone()))) {
            Some(mut item) => {
                ensure!(
                    item.orders.contains(&order_id),
                    "cancel the order but not in market order list"
                );

                // item.orders.retain(|&x| x != order_id);
                for i in 0..item.orders.len() {
                    if item.orders[i] == order_id {
                        item.orders.remove(i);
                    }
                }
                item.buy_amount = item.buy_amount - buy_amount;
                item.sell_amount = item.sell_amount - sell_amount;
                Self::write(key1.clone(), Some(key2.clone()), item.clone());

                if item.orders.len() == 0 {
                    Self::remove_item(key1, key2);
                }
            }
            None => {}
        }

        Ok(())
    }

    pub fn remove_item(key1: Did, key2: u64) {
        if let Some(item) = S::take((key1.clone(), Some(key2))) {
            S::mutate((key1.clone(), item.prev), |x| {
                if let Some(x) = x {
                    x.next = item.next;
                }
            });

            S::mutate((key1.clone(), item.next), |x| {
                if let Some(x) = x {
                    x.prev = item.prev;
                }
            });
        }
    }

    // when the order is canceled, it should be remove from Sell / Buy orders
    pub fn remove_orders_in_one_item(key1: Did, key2: u64) -> Result {
        match S::get((key1.clone(), Some(key2.clone()))) {
            Some(mut item) => {
                while item.orders.len() > 0 {
                    let order_hash = item.orders.get(0).ok_or("can not get order hash")?;

                    let order = <amm::Module<T>>::order(order_hash).ok_or("can not get order")?;
                    ensure!(
                        <amm::Module<T>>::is_limit_order_finished(order),
                        "try to remove not finished order"
                    );

                    item.orders.remove(0);

                    Self::write(key1.clone(), Some(key2.clone()), item.clone());
                }

                if item.orders.len() == 0 {
                    Self::remove_item(key1, key2);
                }
            }
            None => {}
        }

        Ok(())
    }
}
