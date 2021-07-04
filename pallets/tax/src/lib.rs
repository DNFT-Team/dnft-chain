#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{Currency, ExistenceRequirement},
    StorageMap,
};
use frame_system::ensure_signed;
use sp_runtime::DispatchResult;
use sp_std::prelude::*;
use utilities::{ClassId, DAOManager, NFT1155Manager, NFT2006Manager, NFT721Manager, NFTId};
pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Currency: Currency<Self::AccountId>;
    type NFT721: NFT721Manager<Self::AccountId, BalanceOf<Self>>;
    type NFT1155: NFT1155Manager<Self::AccountId, BalanceOf<Self>>;
    type NFT2006: NFT2006Manager<Self::AccountId, BalanceOf<Self>>;
    type DAO: DAOManager<Self::AccountId, BalanceOf<Self>>;
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
    {

        MintNFT721WithTax(AccountId),

        MintNFT1155WithTax(AccountId),

        MintNFT2006WithTax(AccountId),

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
        NFTMintERR,
    }
}

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
decl_storage! {
    trait Store for Module<T: Config> as Tax {

        // Tax
        pub NFTInTax get(fn nft_in_tax): map hasher(blake2_128_concat) T::AccountId => Vec<NFTId>;

    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        #[weight = 10_000]
         pub fn mint_nft721_with_tax(
            origin,
            class_id: ClassId,
            info: Vec<u8>,
            metadata: Vec<u8>,
            price: BalanceOf<T>,
        ) {
            let who = ensure_signed(origin)?;

            let nft_id = T::NFT721::mint_nft(class_id.clone(), info.clone(), metadata.clone(), price.clone(), who.clone());

            ensure!(nft_id != None, Error::<T>::NFTMintERR);

            let mut nids = Self::nft_in_tax(who.clone());

             nids.push(nft_id.unwrap());

            <NFTInTax<T>>::insert(&who, &nids);

            Self::deposit_event(RawEvent::MintNFT721WithTax(who));

        }

        #[weight = 10_000]
         pub fn mint_nft1155_with_tax(
            origin,
            class_id: ClassId,
            info: Vec<u8>,
            metadata: Vec<u8>,
            price: BalanceOf<T>,
        ) {
            let who = ensure_signed(origin)?;

            let nft_id = T::NFT1155::mint_nft(class_id.clone(), info.clone(), metadata.clone(), price.clone(), who.clone());

            ensure!(nft_id != None, Error::<T>::NFTMintERR);

            let mut nids = Self::nft_in_tax(who.clone());

            nids.push(nft_id.unwrap());

            <NFTInTax<T>>::insert(&who, &nids);

            Self::deposit_event(RawEvent::MintNFT721WithTax(who));

        }


        #[weight = 10_000]
         pub fn mint_nft2006_with_tax(
            origin,
            class_id: ClassId,
            info: Vec<u8>,
            metadata: Vec<u8>,
            price: BalanceOf<T>,
        ) {
            let who = ensure_signed(origin)?;

            let nft_id = T::NFT2006::mint_nft(class_id.clone(), info.clone(), metadata.clone(), price.clone(), who.clone());

            ensure!(nft_id != None, Error::<T>::NFTMintERR);

            let mut nids = Self::nft_in_tax(who.clone());

            nids.push(nft_id.unwrap());

            <NFTInTax<T>>::insert(&who, &nids);

            Self::deposit_event(RawEvent::MintNFT721WithTax(who));

        }


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
        let dao = T::DAO::get_dao_account();
        let tax = T::DAO::get_dao_tax();
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
