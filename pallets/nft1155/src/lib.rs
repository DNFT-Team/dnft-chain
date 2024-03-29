#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{Currency, ExistenceRequirement, Get, Randomness},
    StorageMap, StorageValue,
};
use frame_system::ensure_signed;
use randomness;
use sp_io::hashing::blake2_256;
use sp_runtime::DispatchResult;
use sp_std::prelude::*;
use utilities::{ClassId, ClassInfo, NFT1155Manager, NFTId, NFTInfo, NFTSource, NFTStatus};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Currency: Currency<Self::AccountId>;
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
    {
        CreateClass(AccountId),

        MintNFT(AccountId),

        TransferSingleNFT(AccountId),

        OfferNFT(AccountId),

        BuyNFT(AccountId),

        BurnNFT(AccountId),

        TransferBatchNFT(AccountId),
        ApproveSingleNFT(AccountId),

        ApproveBatchNFT(AccountId),

        DestroySingleNFT(AccountId),

        DestroyBatchNFT(AccountId),

        CoupledCollection(AccountId),

        TransferCollection(AccountId),

        ApproveCollection(AccountId),

        DestroyCollection(AccountId),

        ApprovalForAll(AccountId, AccountId, bool),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        NoPermission,
        NFTNotExist,
        NFTSNotExist,
        NFTSExists,
        NFTExists,
        NFTBurned,
        IndexExceedTotalSupply,
        NotNFTOwner,
        CanNotApproveToSelf,
        NotEnoughtNFT,
        CollectionNotExist,
        NotCollectionOwner,
        AlreadlyApproved,
        NFTNotForBuy,
        NFTAlreadyOwned,
        NFTNotOwned,
    }
}
type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

decl_storage! {
    trait Store for Module<T: Config> as NFT1155 {

        // Class
        pub ClassInfos get(fn class_infos): map hasher(twox_64_concat) ClassId => Option<ClassInfo<T::AccountId>>;
        pub ClassCount get(fn class_count): u64;
        pub ClassIndex get(fn class_index): map hasher(blake2_128_concat) u64 => ClassId;
        pub ClassMintIndex get(fn class_mint_index): map hasher(blake2_128_concat) ClassId => u64;

        // NFT
        pub NFTInfos get(fn nft_infos): map hasher(twox_64_concat) NFTId => Option<NFTInfo<T::AccountId, BalanceOf<T>> >;
        pub NFTsCount get(fn nfts_count): u64;
        pub NFTsIndex get(fn nfts_index): map hasher(blake2_128_concat) u64 => NFTId;

        pub NFTByClassIndex get(fn nft_by_class_index):
        double_map hasher(blake2_128_concat) ClassId, hasher(blake2_128_concat) u64 => Option<NFTId>;

        // CNonce
        pub CNonce get(fn cnonce): u64;
        // TNonce
        pub TNonce get(fn tnonce): u64;

        // owned NFT
        pub OwnedNFTIds get(fn owned_nft_ids): map hasher(blake2_128_concat) T::AccountId => Vec<NFTId>;
        pub OwnedNFTSource get(fn owned_nft_source):  map hasher(twox_64_concat) T::AccountId => Vec<NFTSource<ClassId>>;

        //Approvers
        pub NFTApprovers get(fn nft_approvers): map hasher(twox_64_concat) (T::AccountId, NFTId) => bool;
        pub OwnerToApprove get(fn is_approved_for_all): map hasher(twox_64_concat) (T::AccountId, T::AccountId) => bool;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,4)]
        pub fn create_class(
            origin,
            name: Vec<u8>,
            info: Vec<u8>,
            total_supply: u64,
        ) {
            let who = ensure_signed(origin)?;

            Self::_create_class(name, info, total_supply, who.clone())?;

            Self::deposit_event(RawEvent::CreateClass(who));
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,4)]
        pub fn mint_nft(
            origin,
            class_id: ClassId,
            info: Vec<u8>,
            metadata: Vec<u8>,
            price: BalanceOf<T>,
        ) {
            let who = ensure_signed(origin)?;

            Self::_mint_nft(class_id.clone(), info.clone(), metadata.clone(), price.clone(), who.clone());

            Self::deposit_event(RawEvent::MintNFT(who));

        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,4)]
        pub fn transfer_nft(
            origin,
            from: T::AccountId,
            to: T::AccountId,
            nft_id: NFTId,
        ) {
            let who = ensure_signed(origin)?;

            Self::_transfer_single_nft( from.clone(), to.clone(), nft_id.clone())?;

            Self::deposit_event(RawEvent::TransferSingleNFT(who));

        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,4)]
         pub fn offer_nft(
            origin,
            nft_id: NFTId,
            new_price: BalanceOf<T>,
        ) {
            let who = ensure_signed(origin)?;

            Self::_offer_nft( who.clone(), nft_id.clone(), new_price.clone())?;

            Self::deposit_event(RawEvent::OfferNFT(who));

        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,4)]
         pub fn buy_nft(
            origin,
            nft_id: NFTId,
        ) {
            let who = ensure_signed(origin)?;

            Self::_buy_nft( who.clone(), nft_id.clone())?;

            Self::deposit_event(RawEvent::BuyNFT(who));

        }


        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,4)]
        pub fn burn_nft(
            origin,
            nft_id: NFTId,
        ) {
            let who = ensure_signed(origin)?;

            Self::_burn_nft(who.clone(), nft_id.clone())?;

            Self::deposit_event(RawEvent::BurnNFT(who));

        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(8,3)]
        pub fn transfer_batch_nft(
            origin,
            to: T::AccountId,
            class_id: ClassId,
            amount: u64,
        ) {
            let who = ensure_signed(origin)?;

            Self::_transfer_batch_nft(who.clone(), to.clone(), class_id.clone(), amount.clone())?;

            Self::deposit_event(RawEvent::TransferBatchNFT(who));

        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(3,1)]
        pub fn approve_batch_nft(
            origin,
            to: T::AccountId,
            class_id: ClassId,
            amount: u64,
        ) {
            let who = ensure_signed(origin)?;

            Self::_approve_batch_nft(who.clone(), to.clone(), class_id.clone(), amount.clone())?;

            Self::deposit_event(RawEvent::ApproveBatchNFT(who));

        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(3,1)]
        pub fn destroy_batch_nft(
            origin,
            class_id: ClassId,
            amount: u64,
        ) {
            let who = ensure_signed(origin)?;

            Self::_destroy_batch_nft(who.clone(), class_id.clone(), amount.clone())?;

            Self::deposit_event(RawEvent::DestroyBatchNFT(who));

        }

    }
}

// Class
impl<T: Config> Module<T> {
    fn _create_class(
        name: Vec<u8>,
        info: Vec<u8>,
        total_supply: u64,
        issuer: T::AccountId,
    ) -> DispatchResult {
        let nonce = Self::get_cnonce();
        let random_seed = <randomness::Module<T>>::random_seed();
        let encoded = (random_seed, issuer.clone(), nonce).encode();
        let did = blake2_256(&encoded);
        let new_class_id = ClassId { did };
        let new_class = ClassInfo {
            name: name.clone(),
            info: info.clone(),
            total_supply: total_supply.clone(),
            issuer: issuer.clone(),
        };

        <ClassInfos<T>>::insert(new_class_id.clone(), &new_class);
        <ClassCount>::put(nonce.clone() + 1);
        <ClassIndex>::insert(nonce.clone(), new_class_id.clone());
        // Self::_add_class_to_class_list(new_class_id)?;

        Ok(())
    }

    // nonce
    fn get_cnonce() -> u64 {
        let nonce = <CNonce>::get();
        <CNonce>::mutate(|n| *n += 1u64);
        nonce
    }
}

// NFT
impl<T: Config> Module<T> {
    fn _mint_nft(
        class_id: ClassId,
        info: Vec<u8>,
        metadata: Vec<u8>,
        price: BalanceOf<T>,
        miner: T::AccountId,
    ) -> Option<NFTId> {
        if let Some(class_info) = Self::class_infos(class_id.clone()) {
            let class_mint_index = Self::class_mint_index(class_id.clone()) + 1;
            if class_info.total_supply >= class_mint_index {
                let tnonce = Self::get_tnonce();
                let random_seed = <randomness::Module<T>>::random_seed();
                let encoded = (random_seed, miner.clone(), tnonce).encode();
                let did = blake2_256(&encoded);
                let new_nft_id = NFTId { did };

                let new_nft = NFTInfo {
                    class_id: class_id.clone(),
                    index: class_mint_index.clone(),
                    info: info.clone(),
                    metadata: metadata.clone(),
                    owner: miner.clone(),
                    issuer: miner.clone(),
                    price: price.clone(),
                    status: NFTStatus::Normal,
                };

                <NFTInfos<T>>::insert(new_nft_id.clone(), &new_nft);
                <NFTsCount>::put(tnonce.clone() + 1);
                <NFTsIndex>::insert(tnonce.clone(), new_nft_id.clone());
                <ClassMintIndex>::insert(class_id.clone(), class_mint_index.clone());
                <NFTByClassIndex>::insert(
                    class_id.clone(),
                    class_mint_index.clone(),
                    new_nft_id.clone(),
                );

                return Some(new_nft_id);
            }
        }
        None
    }

    fn _transfer_single_nft(from: T::AccountId, to: T::AccountId, nft_id: NFTId) -> DispatchResult {
        let mut nft = Self::nft_infos(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;
        ensure!(nft.owner == from.clone(), Error::<T>::NotNFTOwner);
        ensure!(nft.status != NFTStatus::Burned, Error::<T>::NFTBurned);

        nft.owner = to.clone();
        <NFTInfos<T>>::insert(nft_id.clone(), &nft);
        Self::_remove_nft_from_owned_nftids(from.clone(), nft_id.clone())?;
        Self::_add_nft_to_owned_nftids(to.clone(), nft_id.clone())?;

        Ok(())
    }

    fn _offer_nft(from: T::AccountId, nft_id: NFTId, new_price: BalanceOf<T>) -> DispatchResult {
        let mut nft = Self::nft_infos(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;
        ensure!(nft.owner == from.clone(), Error::<T>::NotNFTOwner);
        ensure!(nft.status != NFTStatus::Burned, Error::<T>::NFTBurned);

        nft.price = new_price.clone();
        nft.status = NFTStatus::Offered;
        <NFTInfos<T>>::insert(nft_id.clone(), &nft);

        Ok(())
    }

    fn _buy_nft(who: T::AccountId, nft_id: NFTId) -> DispatchResult {
        let mut nft = Self::nft_infos(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;
        let from = nft.owner.clone();
        ensure!(nft.owner != who.clone(), Error::<T>::NoPermission);
        ensure!(nft.status == NFTStatus::Offered, Error::<T>::NFTNotForBuy);
        T::Currency::transfer(&who, &nft.owner, nft.price, ExistenceRequirement::KeepAlive)?;
        nft.status = NFTStatus::Normal;
        nft.owner = who.clone();
        Self::_remove_nft_from_owned_nftids(from.clone(), nft_id.clone())?;
        Self::_add_nft_to_owned_nftids(who.clone(), nft_id.clone())?;
        <NFTInfos<T>>::insert(nft_id.clone(), &nft);

        Ok(())
    }

    fn _burn_nft(who: T::AccountId, nft_id: NFTId) -> DispatchResult {
        let mut nft = Self::nft_infos(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;
        ensure!(nft.owner == who.clone(), Error::<T>::NoPermission);
        ensure!(nft.status != NFTStatus::Burned, Error::<T>::NFTBurned);
        nft.status = NFTStatus::Burned;

        <NFTInfos<T>>::insert(nft_id.clone(), &nft);

        Ok(())
    }

    // nonce
    fn get_tnonce() -> u64 {
        let nonce = <TNonce>::get();
        <TNonce>::mutate(|n| *n += 1u64);
        nonce
    }

    fn _approve_single_nft(from: T::AccountId, to: T::AccountId, nft_id: NFTId) -> DispatchResult {
        let nft = Self::nft_infos(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;
        ensure!(nft.owner == from.clone(), Error::<T>::NoPermission);
        ensure!(
            Self::nft_approvers((to.clone(), nft_id.clone())) == false,
            Error::<T>::AlreadlyApproved
        );
        <NFTApprovers<T>>::insert((to.clone(), nft_id.clone()), true);
        Ok(())
    }

    fn _transfer_batch_nft(
        from: T::AccountId,
        to: T::AccountId,
        class_id: ClassId,
        amount: u64,
    ) -> DispatchResult {
        let owned_nfts = Self::owned_nft_source(from.clone());
        for i in 0..owned_nfts.len() {
            if owned_nfts[i].class_id == class_id.clone() {
                ensure!(
                    owned_nfts[i].amount >= amount.clone(),
                    Error::<T>::NotEnoughtNFT
                );
                for j in 0..owned_nfts[i].nfts_indexs.len() {
                    let nft_id = Self::nft_by_class_index(
                        class_id.clone(),
                        owned_nfts[i].nfts_indexs[j].clone(),
                    )
                    .ok_or(Error::<T>::NFTNotExist)?;

                    Self::_transfer_single_nft(from.clone(), to.clone(), nft_id)?;
                }
            }
        }
        Ok(())
    }

    fn _approve_batch_nft(
        who: T::AccountId,
        to: T::AccountId,
        class_id: ClassId,
        amount: u64,
    ) -> DispatchResult {
        let owned_nfts = Self::owned_nft_source(who.clone());
        for i in 0..owned_nfts.len() {
            if owned_nfts[i].class_id == class_id.clone() {
                ensure!(
                    owned_nfts[i].amount >= amount.clone(),
                    Error::<T>::NotEnoughtNFT
                );
                for j in 0..owned_nfts[i].nfts_indexs.len() {
                    let nft_id = Self::nft_by_class_index(
                        class_id.clone(),
                        owned_nfts[i].nfts_indexs[j].clone(),
                    )
                    .ok_or(Error::<T>::NFTNotExist)?;

                    Self::_approve_single_nft(who.clone(), to.clone(), nft_id)?;
                }
            }
        }
        Ok(())
    }

    fn _destroy_batch_nft(who: T::AccountId, class_id: ClassId, amount: u64) -> DispatchResult {
        let owned_nfts = Self::owned_nft_source(who.clone());
        for i in 0..owned_nfts.len() {
            if owned_nfts[i].class_id == class_id.clone() {
                ensure!(
                    owned_nfts[i].amount >= amount.clone(),
                    Error::<T>::NotEnoughtNFT
                );
                for j in 0..owned_nfts[i].nfts_indexs.len() {
                    let nft_id = Self::nft_by_class_index(
                        class_id.clone(),
                        owned_nfts[i].nfts_indexs[j].clone(),
                    )
                    .ok_or(Error::<T>::NFTNotExist)?;

                    Self::_burn_nft(who.clone(), nft_id)?;
                }
            }
        }
        Ok(())
    }
}

impl<T: Config> Module<T> {
    pub fn _add_nft_to_owned_nftids(owner: T::AccountId, nft_id: NFTId) -> DispatchResult {
        ensure!(
            !Self::owned_nft_ids(owner.clone()).contains(&nft_id),
            Error::<T>::NFTAlreadyOwned
        );

        let mut owned_nfts = Self::owned_nft_ids(owner.clone());

        owned_nfts.push(nft_id.clone());

        <OwnedNFTIds<T>>::insert(owner, owned_nfts);

        Ok(())
    }
    pub fn _remove_nft_from_owned_nftids(owner: T::AccountId, nft_id: NFTId) -> DispatchResult {
        ensure!(
            Self::owned_nft_ids(owner.clone()).contains(&nft_id),
            Error::<T>::NFTNotOwned
        );

        let mut owned_nfts = Self::owned_nft_ids(owner.clone());

        let mut j = 0;

        for i in &owned_nfts {
            if *i == nft_id.clone() {
                owned_nfts.remove(j);

                break;
            }

            j += 1;
        }

        <OwnedNFTIds<T>>::insert(owner, owned_nfts);

        Ok(())
    }

    fn _add_nft_to_owned_nft_sources(owner: T::AccountId, class_id: ClassId, nfts_index: u64) {
        if let Some(_nft_id) = Self::nft_by_class_index(class_id.clone(), nfts_index.clone()) {
            let mut owned_nfts = Self::owned_nft_source(owner.clone());
            let mut nfts_exist_flag = false;
            let mut nft_exist_flag = false;
            for i in owned_nfts.clone() {
                if i.class_id == class_id.clone() {
                    nfts_exist_flag = true;
                    for j in i.nfts_indexs.clone() {
                        if j == nfts_index.clone() {
                            nft_exist_flag = true;
                        }
                    }
                }
            }
            if !nft_exist_flag {
                if nfts_exist_flag {
                    for i in 0..owned_nfts.len() {
                        if owned_nfts[i].class_id == class_id.clone() {
                            owned_nfts[i].amount += 1;
                            owned_nfts[i].nfts_indexs.push(nfts_index.clone());
                        }
                    }
                    <OwnedNFTSource<T>>::insert(owner, owned_nfts);
                } else {
                    let new_owned_nfts = NFTSource {
                        class_id: class_id.clone(),
                        amount: 1,
                        nfts_indexs: vec![nfts_index.clone()],
                    };
                    owned_nfts.push(new_owned_nfts);
                    <OwnedNFTSource<T>>::insert(owner, owned_nfts);
                }
            }
        }
    }

    fn _remove_nft_from_owned_nft_sources(
        owner: T::AccountId,
        class_id: ClassId,
        nfts_index: u64,
    ) -> DispatchResult {
        let _nft = Self::nft_by_class_index(class_id.clone(), nfts_index.clone())
            .ok_or(Error::<T>::NFTNotExist)?;
        let mut owned_nfts = Self::owned_nft_source(owner.clone());
        let mut nfts_exist_flag = false;
        let mut nft_exist_flag = false;
        for i in owned_nfts.clone() {
            if i.class_id == class_id.clone() {
                nfts_exist_flag = true;
                for j in i.nfts_indexs.clone() {
                    if j == nfts_index.clone() {
                        nft_exist_flag = true;
                    }
                }
            }
        }
        ensure!(nfts_exist_flag == true, Error::<T>::NFTNotExist);
        ensure!(nft_exist_flag == true, Error::<T>::NFTNotExist);

        for i in 0..owned_nfts.len() {
            if owned_nfts[i].class_id == class_id.clone() {
                let mut k = 0;
                for j in owned_nfts[i].nfts_indexs.clone() {
                    if j == nfts_index.clone() {
                        owned_nfts[i].nfts_indexs.remove(k);
                        owned_nfts[i].amount -= 1;
                        break;
                    }
                    k += 1;
                }
            }
        }
        <OwnedNFTSource<T>>::insert(owner, owned_nfts);

        Ok(())
    }
}

impl<T: Config> NFT1155Manager<T::AccountId, BalanceOf<T>> for Module<T> {
    // Class
    fn issue_nft_class(
        name: Vec<u8>,
        info: Vec<u8>,
        total_supply: u64,
        issuer: T::AccountId,
    ) -> DispatchResult {
        Self::_create_class(name, info, total_supply, issuer)
    }

    fn get_class(class_id: ClassId) -> Option<ClassInfo<T::AccountId>> {
        Self::class_infos(class_id)
    }

    // NFT
    fn mint_nft(
        class_id: ClassId,
        info: Vec<u8>,
        metadata: Vec<u8>,
        price: BalanceOf<T>,
        miner: T::AccountId,
    ) -> Option<NFTId> {
        Self::_mint_nft(class_id, info, metadata, price, miner)
    }

    fn get_nft(nft_id: NFTId) -> Option<NFTInfo<T::AccountId, BalanceOf<T>>> {
        Self::nft_infos(nft_id)
    }

    fn get_nft_by_index(class_id: ClassId, index: u64) -> Option<NFTId> {
        Self::nft_by_class_index(class_id, index)
    }

    // Todo safeTransfer
    fn transfer_single_nft(from: T::AccountId, to: T::AccountId, nft_id: NFTId) -> DispatchResult {
        Self::_transfer_single_nft(from, to, nft_id)
    }

    fn transfer_batch_nft(
        from: T::AccountId,
        to: T::AccountId,
        class_id: ClassId,
        amount: u64,
    ) -> DispatchResult {
        Self::_transfer_batch_nft(from, to, class_id, amount)
    }

    fn approve_single_nft(who: T::AccountId, to: T::AccountId, nft_id: NFTId) -> DispatchResult {
        Self::_approve_single_nft(who, to, nft_id)
    }

    fn approve_batch_nft(
        who: T::AccountId,
        to: T::AccountId,
        class_id: ClassId,
        amount: u64,
    ) -> DispatchResult {
        Self::_approve_batch_nft(who, to, class_id, amount)
    }

    fn destroy_single_nft(who: T::AccountId, nft_id: NFTId) -> DispatchResult {
        Self::_burn_nft(who, nft_id)
    }

    fn destroy_batch_nft(who: T::AccountId, class_id: ClassId, amount: u64) -> DispatchResult {
        Self::_destroy_batch_nft(who, class_id, amount)
    }
}
