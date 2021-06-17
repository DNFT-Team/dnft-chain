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
    trait Store for Module<T: Config> as NFT721 {

        // Class
        pub Class get(fn class): map hasher(twox_64_concat) ClassId => Option<ClassInfo<T::AccountId>>;
        pub ClassCount get(fn class_count): u64;
        pub ClassIndex get(fn class_index): map hasher(blake2_128_concat) u64 => ClassId;
        pub ClassList get(fn class_list):  Vec<ClassId>;


        // NFT
        pub NFTs get(fn nfts): map hasher(twox_64_concat) NFTId => Option<TokenInfo<T::AccountId, BalanceOf<T>> >;
        pub NFTsCount get(fn nfts_count): u64;
        pub NFTsIndex get(fn nfts_index): map hasher(blake2_128_concat) u64 => NFTId;
        pub OwnedNFTs get(fn owned_nfts): map hasher(blake2_128_concat) T::AccountId => Vec<NFTId>;
        pub NFTHolders get(fn nft_holders):  Vec<T::AccountId>;


        // Tax
        pub NFTInTax get(fn nft_in_tax): map hasher(blake2_128_concat) T::AccountId => Vec<NFTId>;

        // DNFTDAO
        pub DAOAcc get(fn dao_acc): T::AccountId;
        pub DAOTax get(fn dao_tax): BalanceOf<T>;

        // CNonce
        pub CNonce get(fn cnonce): u64;
        pub TNonce get(fn tnonce): u64;


    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        #[weight = 10_000 ]
        pub fn set_dao_acc(
            origin,
        ) {
            let who = ensure_signed(origin)?;

            <DAOAcc<T>>::put(&who);

            Self::deposit_event(RawEvent::SetDAOAcc(who));
        }
        #[weight = 10_000 ]
        pub fn set_dao_tax(
            origin,
            price: BalanceOf<T>,
        ) {
            let who = ensure_signed(origin)?;

            <DAOTax<T>>::put(&price);

            Self::deposit_event(RawEvent::SetDAOTax(who));
        }

        #[weight = 10_000 ]
        pub fn create_class(
            origin,
            metadata: Vec<u8>,
            total_issuance: u64,
            data: Vec<u8>,
        ) {
            let who = ensure_signed(origin)?;

            Self::_create_class(metadata, total_issuance, who.clone(), data)?;

            Self::deposit_event(RawEvent::CreateClass(who));
        }

        #[weight = 10_000]
        pub fn mint_nft(
            origin,
            class_id: ClassId,
            metadata: Vec<u8>,
            data: Vec<u8>,
            price: BalanceOf<T>,
        ) {
            let who = ensure_signed(origin)?;

            Self::_mint_nft(class_id.clone(), who.clone(), metadata.clone(), data.clone(), price.clone());

            Self::deposit_event(RawEvent::MintNFT(who));

        }

        #[weight = 10_000 ]
        pub fn transfer_nft(
            origin,
            from: T::AccountId,
            to: T::AccountId,
            nft_id: NFTId,
        ) {
            let who = ensure_signed(origin)?;

            Self::_transfer_nft( from.clone(), to.clone(), nft_id.clone())?;

            Self::deposit_event(RawEvent::TransferNFT(who));

        }

        #[weight = 10_000]
         pub fn offer_nft(
            origin,
            nft_id: NFTId,
            new_price: BalanceOf<T>,
        ) {
            let who = ensure_signed(origin)?;

            Self::_offer_nft( who.clone(), nft_id.clone(), new_price.clone())?;

            Self::deposit_event(RawEvent::OfferNFT(who));

        }

        #[weight = 10_000]
         pub fn buy_nft(
            origin,
            nft_id: NFTId,
        ) {
            let who = ensure_signed(origin)?;

            Self::_buy_nft( who.clone(), nft_id.clone())?;

            Self::deposit_event(RawEvent::BuyNFT(who));

        }


        #[weight = 10_000 ]
        pub fn burn_nft(
            origin,
            nft_id: NFTId,
        ) {
            let who = ensure_signed(origin)?;

            Self::_burn_nft(who.clone(), nft_id.clone())?;

            Self::deposit_event(RawEvent::BurnNFT(who));

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
    fn _create_class(
        metadata: Vec<u8>,
        total_issuance: u64,
        creator: T::AccountId,
        data: Vec<u8>,
    ) -> DispatchResult {
        let nonce = Self::get_cnonce();
        let random_seed = <randomness::Module<T>>::random_seed();
        let encoded = (random_seed, creator.clone(), nonce).encode();
        let id = blake2_256(&encoded);
        let new_class_id = ClassId { id };
        let new_class = ClassInfo {
            metadata: metadata.clone(),
            total_issuance: total_issuance.clone(),
            owner: creator.clone(),
            data: data.clone(),
        };

        <Class<T>>::insert(new_class_id.clone(), &new_class);
        <ClassCount>::put(nonce.clone() + 1);
        <ClassIndex>::insert(nonce.clone(), new_class_id.clone());
        Self::_add_class_to_class_list(new_class_id)?;

        Ok(())
    }
    fn _mint_nft(
        class_id: ClassId,
        miner: T::AccountId,
        metadata: Vec<u8>,
        data: Vec<u8>,
        price: BalanceOf<T>,
    ) -> Option<NFTId> {
        if let Some(class) = Self::class(class_id.clone()) {
            let tnonce = Self::get_tnonce();
            let random_seed = <randomness::Module<T>>::random_seed();
            let encoded = (random_seed, miner.clone(), tnonce).encode();
            let id = blake2_256(&encoded);
            let new_nft_id = NFTId { id };

            let new_nft = TokenInfo {
                metadata: metadata.clone(),
                owner: miner.clone(),
                data: data.clone(),
                price: price.clone(),
                status: NFTStatus::Normal,
            };

            <NFTs<T>>::insert(new_nft_id.clone(), &new_nft);
            <NFTsCount>::put(tnonce.clone() + 1);
            <NFTsIndex>::insert(tnonce.clone(), new_nft_id.clone());
            let _err = Self::_add_nft_to_owned_nfts(miner.clone(), new_nft_id.clone());

            let mut new_class = class.clone();
            new_class.total_issuance = class.total_issuance - 1;
            <Class<T>>::insert(class_id.clone(), &new_class);
            return Some(new_nft_id);
        }
        None
    }

    fn _transfer_nft(from: T::AccountId, to: T::AccountId, nft_id: NFTId) -> DispatchResult {
        let mut nft = Self::nfts(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;
        ensure!(nft.owner == from.clone(), Error::<T>::NotNFTOwner);
        ensure!(nft.status != NFTStatus::Burned, Error::<T>::NFTBurned);

        nft.owner = to.clone();
        <NFTs<T>>::insert(nft_id.clone(), &nft);
        Self::_remove_nft_from_owned_nfts(from.clone(), nft_id.clone())?;
        Self::_add_nft_to_owned_nfts(to.clone(), nft_id.clone())?;

        Ok(())
    }

    fn _offer_nft(from: T::AccountId, nft_id: NFTId, new_price: BalanceOf<T>) -> DispatchResult {
        let mut nft = Self::nfts(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;
        ensure!(nft.owner == from.clone(), Error::<T>::NotNFTOwner);
        ensure!(nft.status != NFTStatus::Burned, Error::<T>::NFTBurned);

        nft.price = new_price.clone();
        nft.status = NFTStatus::Offered;
        <NFTs<T>>::insert(nft_id.clone(), &nft);

        Ok(())
    }

    fn _buy_nft(who: T::AccountId, nft_id: NFTId) -> DispatchResult {
        let mut nft = Self::nfts(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;
        let from = nft.owner.clone();
        ensure!(nft.owner != who.clone(), Error::<T>::NoPermission);
        ensure!(nft.status == NFTStatus::Offered, Error::<T>::NFTNotForBuy);
        T::Currency::transfer(&who, &nft.owner, nft.price, ExistenceRequirement::KeepAlive)?;
        nft.status = NFTStatus::Normal;
        nft.owner = who.clone();
        Self::_remove_nft_from_owned_nfts(from.clone(), nft_id.clone())?;
        Self::_add_nft_to_owned_nfts(who.clone(), nft_id.clone())?;
        <NFTs<T>>::insert(nft_id.clone(), &nft);

        Ok(())
    }

    fn _burn_nft(who: T::AccountId, nft_id: NFTId) -> DispatchResult {
        let mut nft = Self::nfts(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;
        ensure!(nft.owner == who.clone(), Error::<T>::NoPermission);
        ensure!(nft.status != NFTStatus::Burned, Error::<T>::NFTBurned);
        nft.status = NFTStatus::Burned;

        <NFTs<T>>::insert(nft_id.clone(), &nft);

        Ok(())
    }

    fn _pay_nft_tax(who: T::AccountId, nft_id: NFTId) -> DispatchResult {
        let nft = Self::nfts(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;
        let nfts = Self::nft_in_tax(who.clone());
        let dao = Self::dao_acc();
        let tax = Self::dao_tax();
        ensure!(nft.owner == who.clone(), Error::<T>::NoPermission);
        ensure!(nfts.contains(&nft_id) == true, Error::<T>::NoPermission);
        ensure!(nft.status == NFTStatus::Offered, Error::<T>::NFTNotForBuy);
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

    // nonce
    fn get_cnonce() -> u64 {
        let nonce = <CNonce>::get();
        <CNonce>::mutate(|n| *n += 1u64);
        nonce
    }
    fn get_tnonce() -> u64 {
        let nonce = <TNonce>::get();
        <TNonce>::mutate(|n| *n += 1u64);
        nonce
    }
}

impl<T: Config> Module<T> {
    pub fn _add_class_to_class_list(class_id: ClassId) -> DispatchResult {
        ensure!(
            !Self::class_list().contains(&class_id),
            Error::<T>::ClassAlreadyOwned
        );

        let mut class_list = Self::class_list();

        class_list.push(class_id.clone());

        <ClassList>::put(class_list);

        Ok(())
    }
    pub fn _add_nft_to_owned_nfts(owner: T::AccountId, nft_id: NFTId) -> DispatchResult {
        ensure!(
            !Self::owned_nfts(owner.clone()).contains(&nft_id),
            Error::<T>::NFTAlreadyOwned
        );

        let mut owned_nfts = Self::owned_nfts(owner.clone());

        owned_nfts.push(nft_id.clone());

        <OwnedNFTs<T>>::insert(owner, owned_nfts);

        Ok(())
    }
    pub fn _remove_nft_from_owned_nfts(owner: T::AccountId, nft_id: NFTId) -> DispatchResult {
        ensure!(
            Self::owned_nfts(owner.clone()).contains(&nft_id),
            Error::<T>::NFTNotOwned
        );

        let mut owned_nfts = Self::owned_nfts(owner.clone());

        let mut j = 0;

        for i in &owned_nfts {
            if *i == nft_id.clone() {
                owned_nfts.remove(j);

                break;
            }

            j += 1;
        }

        <OwnedNFTs<T>>::insert(owner, owned_nfts);

        Ok(())
    }

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
