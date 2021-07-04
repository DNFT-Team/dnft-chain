#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use codec::Encode;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::Vec,
    ensure,
    traits::{Currency, Get, Randomness, Time},
    StorageMap, StorageValue,
};
use frame_system::ensure_signed;
use randomness;
use sp_io::hashing::blake2_256;
use sp_runtime::DispatchResult;
use sp_std::{
    cmp::{Eq, PartialEq},
    prelude::*,
};
use utilities::{
    Auction, AuctionId, AuctionStatus, AuctionType, BidInfo, NFT1155Manager, NFT2006Manager,
    NFT721Manager, NFTId, NFTType,
};

type MomentOf<T> = <<T as Config>::Time as Time>::Moment;
type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Currency: Currency<Self::AccountId>;
    type Time: Time;
    type NFT721: NFT721Manager<Self::AccountId, BalanceOf<Self>>;
    type NFT1155: NFT1155Manager<Self::AccountId, BalanceOf<Self>>;
    type NFT2006: NFT2006Manager<Self::AccountId, BalanceOf<Self>>;
}

decl_storage! {
    trait Store for Module<T: Config> as Auction {
        // Auction
        pub Auctions get(fn auctions): map hasher(twox_64_concat) AuctionId => Option<Auction<T::AccountId, BalanceOf<T>, MomentOf<T>>>;
        pub AuctionCount get(fn auction_count): u64;
        pub AuctionIndex get(fn auction_index): map hasher(blake2_128_concat) u64 => AuctionId;

        pub Bids get(fn bids): map hasher(blake2_128_concat) AuctionId => Vec<BidInfo<T::AccountId, BalanceOf<T>, MomentOf<T>>>;

        // Nonce
        pub ANonce get(fn anonce): u64;

    }
}

decl_event!(
    pub enum Event<T>
    where
        <T as frame_system::Config>::AccountId,
    {
        LanuchAuction(AccountId),
        BidAuction(AccountId),
        ConfirmAuction(AccountId),
        CancelAuction(AccountId),
    }
);

decl_error! {
    /// Error for the trade module.
    pub enum Error for Module<T: Config> {
        AuctionNotExist,
        NotAuctionOwner,

    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        type Error = Error<T>;
        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,4)]
        pub fn lanuch_auction(
            origin,
            auction_type: AuctionType,
            nft_type: NFTType,
            nft_id: NFTId,
            base_price: Option<BalanceOf<T>>,
            start_time: Option<MomentOf<T>>,
            end_time: Option<MomentOf<T>>,
        ){
            let sender = ensure_signed(origin)?;

            Self::_lanuch_auction(auction_type, nft_type, nft_id, base_price, start_time, end_time, sender.clone())?;

            Self::deposit_event(RawEvent::LanuchAuction(sender));

        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,4)]
        pub fn bid(origin, auction_id: AuctionId, price: BalanceOf<T>, time: MomentOf<T>){
            let sender = ensure_signed(origin)?;

            Self::_bid_auction(auction_id, price, time, sender.clone())?;

            Self::deposit_event(RawEvent::BidAuction(sender));

        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,4)]
        pub fn confirm_bid_result(origin, auction_id: AuctionId, winner: T::AccountId){
            let sender = ensure_signed(origin)?;

            let auction = Self::auctions(auction_id.clone()).ok_or(Error::<T>::AuctionNotExist)?;

            ensure!(auction.owner == sender.clone(), Error::<T>::NotAuctionOwner);


            Self::_confirm_auction(auction_id, sender.clone(), winner.clone())?;

            Self::deposit_event(RawEvent::ConfirmAuction(sender));

        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,4)]
        pub fn cancel_bid(origin, auction_id: AuctionId){
            let sender = ensure_signed(origin)?;

            let mut auction = Self::auctions(auction_id.clone()).ok_or(Error::<T>::AuctionNotExist)?;

            ensure!(auction.owner == sender.clone(), Error::<T>::NotAuctionOwner);

            auction.status = AuctionStatus::Canceled;

            <Auctions<T>>::insert(auction_id.clone(), &auction);

            Self::deposit_event(RawEvent::CancelAuction(sender));

        }



    }
}

//AIData
impl<T: Config> Module<T> {
    fn _lanuch_auction(
        auction_type: AuctionType,
        nft_type: NFTType,
        nft_id: NFTId,
        base_price: Option<BalanceOf<T>>,
        start_time: Option<MomentOf<T>>,
        end_time: Option<MomentOf<T>>,
        sender: T::AccountId,
    ) -> DispatchResult {
        let nonce = Self::get_anonce();
        let random_seed = <randomness::Module<T>>::random_seed();
        let encoded = (random_seed, sender.clone(), nonce).encode();
        let did = blake2_256(&encoded);
        let new_auction_id = AuctionId { did };
        let new_auction = Auction {
            owner: sender.clone(),
            auction_type: auction_type.clone(),
            nft_type: nft_type.clone(),
            nft_id: nft_id.clone(),
            base_price: base_price.clone(),
            start_time: start_time.clone(),
            end_time: end_time.clone(),
            status: AuctionStatus::Created,
        };

        <Auctions<T>>::insert(new_auction_id.clone(), &new_auction);
        <AuctionCount>::put(nonce.clone() + 1);
        <AuctionIndex>::insert(nonce.clone(), new_auction_id.clone());

        Ok(())
    }

    fn _bid_auction(
        auction_id: AuctionId,
        price: BalanceOf<T>,
        time: MomentOf<T>,
        sender: T::AccountId,
    ) -> DispatchResult {
        let _auction = Self::auctions(auction_id.clone()).ok_or(Error::<T>::AuctionNotExist)?;
        //To Do
        //check bid action legal
        let new_bid = BidInfo {
            bidder: sender.clone(),
            price: price.clone(),
            time: time.clone(),
            is_legal: true,
            is_winner: false,
        };

        let mut bids = Self::bids(&auction_id);

        bids.push(new_bid.clone());

        <Bids<T>>::insert(auction_id.clone(), &bids);

        Ok(())
    }

    fn _confirm_auction(
        auction_id: AuctionId,
        sender: T::AccountId,
        winner: T::AccountId,
    ) -> DispatchResult {
        // to do
        // this action has a coditdion that the winner bidder should have send money to the NFT holder
        // this should be autoly handled here in the future.

        let mut auction = Self::auctions(auction_id.clone()).ok_or(Error::<T>::AuctionNotExist)?;

        auction.status = AuctionStatus::Confirmed;

        <Auctions<T>>::insert(auction_id.clone(), &auction);

        match auction.nft_type {
            NFTType::NFT721 => T::NFT721::transfer_single_nft(
                sender.clone(),
                winner.clone(),
                auction.nft_id.clone(),
            )?,
            NFTType::NFT1155 => T::NFT1155::transfer_single_nft(
                sender.clone(),
                winner.clone(),
                auction.nft_id.clone(),
            )?,
            NFTType::NFT2006 => T::NFT2006::transfer_single_nft(
                sender.clone(),
                winner.clone(),
                auction.nft_id.clone(),
            )?,
        };

        Ok(())
    }

    // nonce
    fn get_anonce() -> u64 {
        let nonce = <ANonce>::get();
        <ANonce>::mutate(|n| *n += 1u64);
        nonce
    }
}
