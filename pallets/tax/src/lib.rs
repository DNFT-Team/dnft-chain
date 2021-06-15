#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{Currency, ExistenceRequirement, Randomness},
    StorageMap, StorageValue,
};
use frame_system::ensure_signed;
use pallet_randomness_collective_flip as randomness;
use sp_io::hashing::blake2_256;
use sp_runtime::{DispatchResult, RuntimeDebug};
use sp_std::prelude::*;

/// Class info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ClassInfo<AccountId> {
    /// Class metadata
    pub metadata: Vec<u8>,
    /// Total issuance for the class
    pub total_issuance: u64,
    /// Class owner
    pub owner: AccountId,
    /// Class Properties
    pub data: Vec<u8>,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum NFTStatus {
    Normal = 0,
    Offered,
    Collected,
    Burned,
}

/// Token info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct TokenInfo<AccountId, Balance> {
    /// Token metadata
    pub metadata: Vec<u8>,
    /// Token owner
    pub owner: AccountId,
    /// Token Properties
    pub data: Vec<u8>,
    /// Balance Properties
    pub price: Balance,
    /// Balance Properties
    pub status: NFTStatus,
}

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct NFTId {
    pub id: [u8; 32],
}

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct ClassId {
    pub id: [u8; 32],
}

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Currency: Currency<Self::AccountId>;
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
    {
        SetDAOAcc(AccountId),

        SetDAOTax(AccountId),

        CreateClass(AccountId),

        MintNFT(AccountId),

        TransferNFT(AccountId),

        OfferNFT(AccountId),

        BuyNFT(AccountId),

        BurnNFT(AccountId),

        PayNFTTax(AccountId),

        PayTotalNFTTax(AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        NoPermission,
        NFTNotExist,
        ClassNotExist,
        ClassExists,
        ExceedTotalIssuance,
        NotNFTOwner,
        NFTBurned,
        NFTAlreadyOwned,
        NFTNotOwned,
        ClassAlreadyOwned,
        NFTNotForBuy,
    }
}
type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
decl_storage! {
    trait Store for Module<T: Config> as Tax {

        

        // Tax
        pub NFTInTax get(fn nft_in_tax): map hasher(blake2_128_concat) T::AccountId => Vec<NFTId>;

        // DNFTDAO
        pub DAOAcc get(fn dao_acc): T::AccountId;
        pub DAOTax get(fn dao_tax): BalanceOf<T>;



    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        

        #[weight = 10_000]
         pub fn pay_tax(
            origin,
            nft_id: NFTId,
        ) {
            let who = ensure_signed(origin)?;

            Self::_pay_nft_tax( who.clone(), nft_id.clone())?;

            Self::deposit_event(RawEvent::PayNFTTax(who));

        }

        #[weight = 10_000]
         pub fn pay_total_tax(
            origin,
        ) {
            let who = ensure_signed(origin)?;

            Self::_pay_total_tax(who.clone())?;

            Self::deposit_event(RawEvent::PayTotalNFTTax(who));

        }
    //     fn on_initialize(block_number: T::BlockNumber) -> Weight {
    //         let number: T::BlockNumber = <<T as frame_system::Trait>::BlockNumber as From<_>>::from(100);

    //         if block_number % number == <<T as frame_system::Config>::BlockNumber as From<_>>::from(0){
    //             for acc in Self::nft_holders() {
    //                 let mut nids = Self::owned_nfts(acc.clone());
    //                 for i in 0..nids.len() {
    //                     if let Some(mut nft) = Self::nfts(nids[i].clone()){
    //                         nft.status = NFTStatus::Collected;
    //                         <NFTs<T>>::insert(nids[i].clone(), &nft);
    //                     }
    //                     nids.remove(i);
    //                 }

    //                 <NFTInTax<T>>::insert(&acc, nids);
    //             }
    //             return 100_000
    //         }

    //         1000
    //     }
    //       fn on_finalize(block_number: T::BlockNumber) {
    //         let number: T::BlockNumber = <<T as frame_system::Trait>::BlockNumber as From<_>>::from(110);

    //         if block_number % number == <<T as frame_system::Trait>::BlockNumber as From<_>>::from(0){
    //             for acc in Self::nft_holders() {
    //                 <NFTInTax<T>>::insert(&acc, Self::owned_nfts(acc.clone()));
    //             }
    //         }
    //       }

    }
}

impl<T: Config> Module<T> {


    fn _pay_nft_tax(who: T::AccountId, nft_id: NFTId) -> DispatchResult {
        // let nft = Self::nfts(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;
        let nfts = Self::nft_in_tax(who.clone());
        let dao = Self::dao_acc();
        let tax = Self::dao_tax();
        // ensure!(nft.owner == who.clone(), Error::<T>::NoPermission);
        ensure!(nfts.contains(&nft_id) == true, Error::<T>::NoPermission);
        // ensure!(nft.status == NFTStatus::Offered, Error::<T>::NFTNotForBuy);
        T::Currency::transfer(&who, &dao, tax, ExistenceRequirement::KeepAlive)?;

        Self::_remove_nft_from_nft_in_tax(who, nft_id)?;

        Ok(())
    }

    fn _pay_total_tax(who: T::AccountId) -> DispatchResult {
        let nfts = Self::nft_in_tax(who.clone());
        for i in nfts {
            Self::_pay_nft_tax(who.clone(), i)?;
        }

        Ok(())
    }

}

impl<T: Config> Module<T> {

    pub fn _remove_nft_from_nft_in_tax(owner: T::AccountId, nft_id: NFTId) -> DispatchResult {
        ensure!(
            Self::nft_in_tax(owner.clone()).contains(&nft_id),
            Error::<T>::NFTNotOwned
        );

        let mut owned_nfts_in_tax = Self::nft_in_tax(owner.clone());

        let mut j = 0;

        for i in &owned_nfts_in_tax {
            if *i == nft_id.clone() {
                owned_nfts_in_tax.remove(j);

                break;
            }

            j += 1;
        }

        <NFTInTax<T>>::insert(owner, owned_nfts_in_tax);

        Ok(())
    }
}
