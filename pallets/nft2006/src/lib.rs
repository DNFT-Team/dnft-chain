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

use utilities::{
    ClassId, ClassInfo, Collection, CollectionId, CollectionStatus, Did, NFT2006Manager, NFTId,
    NFTInfo, NFTSource, NFTStatus, TokenManager,
};

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Currency: Currency<Self::AccountId>;
    type Token: TokenManager<Self::AccountId>;
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

        NFTFragmentation(AccountId),
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
        NFTAlreadyShiftINFragmentation,
    }
}

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

decl_storage! {
    trait Store for Module<T: Config> as NFT2006 {

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


        //Approvers
        pub NFTApprovers get(fn nft_approvers): map hasher(twox_64_concat) (T::AccountId, NFTId) => bool;
        pub CollectionApprovers get(fn collection_approvers): map hasher(twox_64_concat) (T::AccountId, CollectionId) => bool;
        pub OwnerToApprove get(fn is_approved_for_all): map hasher(twox_64_concat) (T::AccountId, T::AccountId) => bool;

        // Collection
        pub Collections get(fn collections): map hasher(twox_64_concat) CollectionId => Option<Collection<T::AccountId, BalanceOf<T>, CollectionStatus, ClassId>>;
        pub CollectionsCount get(fn collections_count): u64;
        pub CollectionsIndex get(fn collections_index): map hasher(blake2_128_concat) u64 => CollectionId;

        // CoNonce
        pub CoNonce get(fn cononce): u64;

        // owned NFT
        pub OwnedNFTIds get(fn owned_nft_ids): map hasher(blake2_128_concat) T::AccountId => Vec<NFTId>;
        pub OwnedNFTSource get(fn owned_nft_source):  map hasher(twox_64_concat) T::AccountId => Vec<NFTSource<ClassId>>;
        pub OwnedCollections get(fn owned_collections):  map hasher(twox_64_concat) T::AccountId => Vec<CollectionId>;



        /// NFT2006 fragmentation NFTId --> TokenId
        pub NFTShiftToken get(fn nft_shift_token): map hasher(blake2_128_concat) NFTId => Option<Did>;



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

        #[weight = 10_000 + T::DbWeight::get().reads_writes(2,3)]
        pub fn coupled_collection(
            origin,
            name: Vec<u8>,
            symbol: Vec<u8>,
            info: Vec<u8>,
            price: BalanceOf<T>,
            source: Vec<NFTSource<ClassId>>,
        ) {
            let who = ensure_signed(origin)?;

            Self::_coupled_collection(name.clone(), symbol.clone(), who.clone(), info.clone(), price.clone(), source.clone())?;

            Self::deposit_event(RawEvent::CoupledCollection(who));

        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(4,2)]
        pub fn transfer_collection(
            origin,
            from: T::AccountId,
            to: T::AccountId,
            collection_id: CollectionId,
        ) {
            let who = ensure_signed(origin)?;

            Self::_transfer_collection(who.clone(), from.clone(), to.clone(), collection_id.clone())?;

            Self::deposit_event(RawEvent::TransferCollection(who));

        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn approve_collection(
            origin,
            to: T::AccountId,
            collection_id: CollectionId,
        ) {
            let who = ensure_signed(origin)?;

            Self::_approve_collection(who.clone(), to.clone(), collection_id.clone())?;

            Self::deposit_event(RawEvent::ApproveCollection(who));

        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn destroy_collection(
            origin,
            collection_id: CollectionId,
        ) {
            let who = ensure_signed(origin)?;

            Self::_destroy_collection(who.clone(), collection_id.clone())?;

            Self::deposit_event(RawEvent::DestroyCollection(who));

        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_approval_for_all(
            origin,
            to: T::AccountId,
            approved: bool,
        ) {
            let sender = ensure_signed(origin)?;

            Self::_set_approval_for_all(sender.clone(), to.clone(), approved)?;

            Self::deposit_event(RawEvent::ApprovalForAll(sender, to, approved));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn nft_fragmentation(
            origin,
            nft_id: NFTId,
            total_supply: u64,
            symbol: Vec<u8>,
        ) {
            let sender = ensure_signed(origin)?;

            let nft = Self::nft_infos(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;

            ensure!(nft.owner == sender.clone(), Error::<T>::NotNFTOwner);

            let token_id = T::Token::issue(sender.clone(), total_supply.clone(), symbol.clone());

            let bound_token_id = Self::nft_shift_token(nft_id.clone());

            ensure!(bound_token_id == None, Error::<T>::NFTAlreadyShiftINFragmentation);

            <NFTShiftToken>::insert(&nft_id, &token_id);

            Self::deposit_event(RawEvent::NFTFragmentation(sender));
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

// collection
impl<T: Config> Module<T> {
    fn _coupled_collection(
        name: Vec<u8>,
        symbol: Vec<u8>,
        owner: T::AccountId,
        info: Vec<u8>,
        price: BalanceOf<T>,
        source: Vec<NFTSource<ClassId>>,
    ) -> DispatchResult {
        for i in 0..source.len() {
            for j in 0..source[i].nfts_indexs.len() {
                let nft_id = Self::nft_by_class_index(
                    source[i].class_id.clone(),
                    source[i].nfts_indexs[j].clone(),
                )
                .ok_or(Error::<T>::NFTNotExist)?;

                Self::_collection_single_nft(owner.clone(), nft_id)?;
            }
        }
        let cnonce = Self::get_cononce();
        let random_seed = <randomness::Module<T>>::random_seed();
        let encoded = (random_seed, owner.clone(), cnonce).encode();
        let did = blake2_256(&encoded);
        let new_collection_id = CollectionId { did };
        let new_collection = Collection {
            name: name.clone(),
            symbol: symbol.clone(),
            info: info.clone(),
            owner: owner.clone(),
            issuer: owner.clone(),
            price: price.clone(),
            source: source.clone(),
            status: CollectionStatus::Normal,
        };

        <Collections<T>>::insert(new_collection_id.clone(), &new_collection);
        <CollectionsCount>::put(cnonce.clone() + 1);
        <CollectionsIndex>::insert(cnonce.clone(), new_collection_id.clone());

        Self::_add_collection_to_owned_collections(owner.clone(), new_collection_id.clone())?;

        Ok(())
    }

    fn _decoupled_collection(from: T::AccountId, collection_id: CollectionId) -> DispatchResult {
        let mut collection =
            Self::collections(collection_id.clone()).ok_or(Error::<T>::CollectionNotExist)?;
        ensure!(collection.owner == from.clone(), Error::<T>::NoPermission);
        collection.status = CollectionStatus::Decoupled;
        <Collections<T>>::insert(collection_id.clone(), &collection);

        Ok(())
    }

    fn _transfer_collection(
        who: T::AccountId,
        from: T::AccountId,
        to: T::AccountId,
        collection_id: CollectionId,
    ) -> DispatchResult {
        let mut collection =
            Self::collections(collection_id.clone()).ok_or(Error::<T>::CollectionNotExist)?;
        ensure!(
            collection.owner == from.clone(),
            Error::<T>::NotCollectionOwner
        );

        let is_legal = CollectionStatus::Normal == collection.status;
        let is_owner = who == collection.owner;
        let is_approved_for_all = Self::is_approved_for_all((from.clone(), who.clone()));

        ensure!(
            is_legal || is_owner || is_approved_for_all,
            Error::<T>::NoPermission
        );

        collection.owner = to.clone();
        <Collections<T>>::insert(collection_id.clone(), &collection);

        Self::_remove_collection_from_owned_collections(from.clone(), collection_id.clone())?;
        Self::_add_collection_to_owned_collections(to.clone(), collection_id.clone())?;
        Ok(())
    }

    fn _approve_collection(
        from: T::AccountId,
        to: T::AccountId,
        collection_id: CollectionId,
    ) -> DispatchResult {
        let collection =
            Self::collections(collection_id.clone()).ok_or(Error::<T>::CollectionNotExist)?;
        ensure!(collection.owner == from.clone(), Error::<T>::NoPermission);
        <CollectionApprovers<T>>::insert((to.clone(), collection_id.clone()), true);

        Ok(())
    }

    fn _destroy_collection(from: T::AccountId, collection_id: CollectionId) -> DispatchResult {
        let mut collection =
            Self::collections(collection_id.clone()).ok_or(Error::<T>::CollectionNotExist)?;
        ensure!(collection.owner == from.clone(), Error::<T>::NoPermission);
        collection.status = CollectionStatus::Burned;
        <Collections<T>>::insert(collection_id.clone(), &collection);
        // todo
        // destroy nft

        Ok(())
    }

    fn _collection_single_nft(who: T::AccountId, nft_id: NFTId) -> DispatchResult {
        let mut nft = Self::nft_infos(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;
        ensure!(nft.owner == who.clone(), Error::<T>::NoPermission);
        nft.status = NFTStatus::InCollection;
        <NFTInfos<T>>::insert(nft_id.clone(), &nft);

        Ok(())
    }

    fn _add_collection_to_owned_collections(
        owner: T::AccountId,
        collection_id: CollectionId,
    ) -> DispatchResult {
        let _collection =
            Self::collections(collection_id.clone()).ok_or(Error::<T>::CollectionNotExist)?;
        let mut owned_collections = Self::owned_collections(owner.clone());
        owned_collections.push(collection_id.clone());
        <OwnedCollections<T>>::insert(owner, owned_collections);
        Ok(())
    }

    fn _remove_collection_from_owned_collections(
        owner: T::AccountId,
        collection_id: CollectionId,
    ) -> DispatchResult {
        let _collection =
            Self::collections(collection_id.clone()).ok_or(Error::<T>::CollectionNotExist)?;
        let mut owned_collections = Self::owned_collections(owner.clone());
        for i in 0..owned_collections.len() {
            if owned_collections[i] == collection_id.clone() {
                owned_collections.remove(i);
            }
        }
        <OwnedCollections<T>>::insert(owner, owned_collections);
        Ok(())
    }

    fn _set_approval_for_all(
        owner: T::AccountId,
        to: T::AccountId,
        approved: bool,
    ) -> DispatchResult {
        ensure!(owner.clone() != to.clone(), Error::<T>::CanNotApproveToSelf);
        <OwnerToApprove<T>>::insert((&owner, &to), approved);
        Ok(())
    }

    // cononce
    fn get_cononce() -> u64 {
        let cononce = <CoNonce>::get();
        <CoNonce>::mutate(|n| *n += 1u64);
        cononce
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

impl<T: Config> NFT2006Manager<T::AccountId, BalanceOf<T>> for Module<T> {
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

    // Collection
    fn coupled_collection(
        name: Vec<u8>,
        symbol: Vec<u8>,
        owner: T::AccountId,
        info: Vec<u8>,
        price: BalanceOf<T>,
        source: Vec<NFTSource<ClassId>>,
    ) -> DispatchResult {
        Self::_coupled_collection(name, symbol, owner, info, price, source)
    }

    fn get_collection(
        collection_id: CollectionId,
    ) -> Option<Collection<T::AccountId, BalanceOf<T>, CollectionStatus, ClassId>> {
        Self::collections(collection_id)
    }

    fn owned_collections(account: T::AccountId) -> Vec<CollectionId> {
        Self::owned_collections(account)
    }

    fn decoupled_collection(who: T::AccountId, collection_id: CollectionId) -> DispatchResult {
        Self::_decoupled_collection(who, collection_id)
    }

    fn transfer_collection(
        who: T::AccountId,
        from: T::AccountId,
        to: T::AccountId,
        collection_id: CollectionId,
    ) -> DispatchResult {
        Self::_transfer_collection(who, from, to, collection_id)
    }

    fn destroy_collection(who: T::AccountId, collection_id: CollectionId) -> DispatchResult {
        Self::_destroy_collection(who, collection_id)
    }

    fn approve_collection(
        who: T::AccountId,
        to: T::AccountId,
        collection_id: CollectionId,
    ) -> DispatchResult {
        Self::_approve_collection(who, to, collection_id)
    }

    fn set_approval_for_all(who: T::AccountId, to: T::AccountId, approved: bool) -> DispatchResult {
        Self::_set_approval_for_all(who, to, approved)
    }
}
