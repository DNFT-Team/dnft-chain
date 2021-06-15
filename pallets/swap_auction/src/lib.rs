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

};

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;


}

decl_storage! {
    trait Store for Module<T: Config> as Amm {

  

    }
}

decl_event!(
    pub enum Event<T>
    where
        <T as frame_system::Config>::AccountId,
    {
        NewAuction(AccountId),
    }
);

decl_error! {
    /// Error for the trade module.
    pub enum Error for Module<T: Config> {

    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        type Error = Error<T>;
        #[weight = 1_000_000]
        pub fn new_auction(origin, base_amount: u64, quote_amount: u64){
            let sender = ensure_signed(origin)?;

        }
        #[weight = 1_000_000]
        pub fn bid(origin, auction_id: u64, liquidity_share: u64){
            let sender = ensure_signed(origin)?;
            
        }



    }
}
