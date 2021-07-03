#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{Currency, ExistenceRequirement, Get, Randomness, Time},
    StorageMap, StorageValue,
};
use frame_system::ensure_signed;
use pallet_randomness_collective_flip as randomness;
use sp_io::hashing::blake2_256;
use sp_runtime::{DispatchResult, RuntimeDebug};
use sp_std::prelude::*;
use utilities::{
    AIData, AIDataId, AIModel, AIModelHighlight, AIModelId, ClassId, CollectionId, DataIndustry,
    DataResource, DataTechnology, ModelLanguage, NFT2006Manager, NFTId,
};
type MomentOf<T> = <<T as Config>::Time as Time>::Moment;
type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type NFT2006: NFT2006Manager<Self::AccountId, BalanceOf<Self>>;
    type Currency: Currency<Self::AccountId>;
    type Time: Time;
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
    {
        CreateAIData(AccountId),
        CreateAIModel(AccountId),
        BoundAIDataWithNFT(AccountId),
        BoundAIDataWithCollection(AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        NoPermission,
        AIDataNotExist,
        NotAIDataOwner,
        NFTAlreadyBounded,
        NFTMintERR,
        CollectionAlreadyBounded,
        CollectionCreatERR,
        NotAIModelOwner,
        AIModelNotExist,

    }
}

decl_storage! {
    trait Store for Module<T: Config> as AI {

        // AIData
        pub AIDatas get(fn ai_datas): map hasher(twox_64_concat) AIDataId => Option<AIData<T::AccountId, MomentOf<T>>>;
        pub AIDataCount get(fn ai_data_count): u64;
        pub AIDataIndex get(fn ai_data_index): map hasher(blake2_128_concat) u64 => AIDataId;


        // AIModel
        pub AIModels get(fn ai_models): map hasher(twox_64_concat) AIModelId => Option<AIModel<T::AccountId, MomentOf<T>>>;
        pub AIModelCount get(fn ai_model_count): u64;
        pub AIModelIndex get(fn ai_model_index): map hasher(blake2_128_concat) u64 => AIModelId;


        // Nonce
        pub DNonce get(fn dnonce): u64;
        pub MNonce get(fn mnonce): u64;


    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,4)]
        pub fn create_ai_data(
            origin,
            industry:   DataIndustry,
            technology: DataTechnology,
            resource:   DataResource,
            timestamp:  MomentOf<T>,
        ) {
            let who = ensure_signed(origin)?;

            Self::_create_ai_data(industry, technology, resource, timestamp, who.clone())?;

            Self::deposit_event(RawEvent::CreateAIData(who));
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,4)]
        pub fn bound_ai_data_with_nft(
            origin,
            ai_data_id: AIDataId,
            class_id: ClassId,
            info: Vec<u8>,
            metadata: Vec<u8>,
            price: BalanceOf<T>,
        ) {
            let who = ensure_signed(origin)?;

            let mut ai_data = Self::ai_datas(ai_data_id.clone()).ok_or(Error::<T>::AIDataNotExist)?;

            ensure!(ai_data.creator == who.clone(), Error::<T>::NotAIDataOwner);

            ensure!(ai_data.nft_id == None, Error::<T>::NFTAlreadyBounded);

            let nft_id = T::NFT2006::mint_nft(class_id.clone(), info.clone(), metadata.clone(), price.clone(), who.clone());

            ensure!(nft_id != None, Error::<T>::NFTMintERR);

            Self::_bound_ai_data_nft(ai_data_id, nft_id.unwrap())?;

            Self::deposit_event(RawEvent::BoundAIDataWithNFT(who));
        }


        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,4)]
        pub fn bound_ai_data_with_collection(
            origin,
            ai_data_id: AIDataId,
            collection_id: CollectionId,
        ) {
            let who = ensure_signed(origin)?;

            let mut ai_data = Self::ai_datas(ai_data_id.clone()).ok_or(Error::<T>::AIDataNotExist)?;

            ensure!(ai_data.creator == who.clone(), Error::<T>::NotAIDataOwner);

            ensure!(ai_data.collection_id == None, Error::<T>::CollectionAlreadyBounded);

            let collection = T::NFT2006::get_collection(collection_id.clone());

            ensure!(collection != None, Error::<T>::CollectionCreatERR);

            ensure!(collection.unwrap().owner == who, Error::<T>::NoPermission);

            Self::_bound_ai_data_collection(ai_data_id, collection_id)?;

            Self::deposit_event(RawEvent::BoundAIDataWithCollection(who));
        }


        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,4)]
        pub fn create_ai_model(
            origin,
            title: Vec<u8>,
            language: ModelLanguage,
            framwork: Vec<u8>,
            timestamp: MomentOf<T>,
            highlight: Vec<AIModelHighlight>,
        ) {
            let who = ensure_signed(origin)?;

            Self::_create_ai_model(title, language, framwork, timestamp, highlight, who.clone())?;

            Self::deposit_event(RawEvent::CreateAIModel(who));
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,4)]
        pub fn bound_ai_model_with_nft(
            origin,
            ai_model_id: AIModelId,
            class_id: ClassId,
            info: Vec<u8>,
            metadata: Vec<u8>,
            price: BalanceOf<T>,
        ) {
            let who = ensure_signed(origin)?;

            let mut ai_model = Self::ai_models(ai_model_id.clone()).ok_or(Error::<T>::AIModelNotExist)?;

            ensure!(ai_model.creator == who.clone(), Error::<T>::NotAIModelOwner);

            ensure!(ai_model.nft_id == None, Error::<T>::NFTAlreadyBounded);

            let nft_id = T::NFT2006::mint_nft(class_id.clone(), info.clone(), metadata.clone(), price.clone(), who.clone());

            ensure!(nft_id != None, Error::<T>::NFTMintERR);

            Self::_bound_ai_model_nft(ai_model_id, nft_id.unwrap())?;

            Self::deposit_event(RawEvent::BoundAIDataWithNFT(who));
        }
    }
}

//AIData
impl<T: Config> Module<T> {
    fn _create_ai_data(
        industry: DataIndustry,
        technology: DataTechnology,
        resource: DataResource,
        timestamp: MomentOf<T>,
        creator: T::AccountId,
    ) -> DispatchResult {
        let nonce = Self::get_dnonce();
        let random_seed = <randomness::Module<T>>::random_seed();
        let encoded = (random_seed, creator.clone(), nonce).encode();
        let did = blake2_256(&encoded);
        let new_ai_data_id = AIDataId { did };
        let new_ai_data = AIData {
            creator: creator.clone(),
            industry: industry.clone(),
            technology: technology.clone(),
            resource: resource.clone(),
            stars: 0,
            timestamp: timestamp.clone(),
            nft_id: None,
            collection_id: None,
        };

        <AIDatas<T>>::insert(new_ai_data_id.clone(), &new_ai_data);
        <AIDataCount>::put(nonce.clone() + 1);
        <AIDataIndex>::insert(nonce.clone(), new_ai_data_id.clone());

        Ok(())
    }

    fn _bound_ai_data_nft(ai_data_id: AIDataId, nft_id: NFTId) -> DispatchResult {
        let mut ai_data = Self::ai_datas(ai_data_id.clone()).ok_or(Error::<T>::AIDataNotExist)?;
        ai_data.nft_id = Some(nft_id.clone());

        <AIDatas<T>>::insert(ai_data_id.clone(), &ai_data);

        Ok(())
    }

    fn _bound_ai_data_collection(
        ai_data_id: AIDataId,
        collection_id: CollectionId,
    ) -> DispatchResult {
        let mut ai_data = Self::ai_datas(ai_data_id.clone()).ok_or(Error::<T>::AIDataNotExist)?;
        ai_data.collection_id = Some(collection_id.clone());

        <AIDatas<T>>::insert(ai_data_id.clone(), &ai_data);

        Ok(())
    }

    // donce
    fn get_dnonce() -> u64 {
        let nonce = <DNonce>::get();
        <DNonce>::mutate(|n| *n += 1u64);
        nonce
    }
}

//AIModel
impl<T: Config> Module<T> {
    fn _create_ai_model(
        title: Vec<u8>,
        language: ModelLanguage,
        framwork: Vec<u8>,
        timestamp: MomentOf<T>,
        highlight: Vec<AIModelHighlight>,
        creator: T::AccountId,
    ) -> DispatchResult {
        let nonce = Self::get_mnonce();
        let random_seed = <randomness::Module<T>>::random_seed();
        let encoded = (random_seed, creator.clone(), nonce).encode();
        let did = blake2_256(&encoded);
        let new_ai_model_id = AIModelId { did };
        let new_ai_model = AIModel {
            creator: creator.clone(),
            title: title.clone(),
            language: language.clone(),
            framwork: framwork.clone(),
            stars: 0,
            timestamp: timestamp.clone(),
            highlight: highlight.clone(),
            nft_id: None,
        };

        <AIModels<T>>::insert(new_ai_model_id.clone(), &new_ai_model);
        <AIModelCount>::put(nonce.clone() + 1);
        <AIModelIndex>::insert(nonce.clone(), new_ai_model_id.clone());

        Ok(())
    }

    fn _bound_ai_model_nft(ai_model_id: AIModelId, nft_id: NFTId) -> DispatchResult {
        let mut ai_model =
            Self::ai_models(ai_model_id.clone()).ok_or(Error::<T>::AIModelNotExist)?;
        ai_model.nft_id = Some(nft_id.clone());

        <AIModels<T>>::insert(ai_model_id.clone(), &ai_model);

        Ok(())
    }

    // monce
    fn get_mnonce() -> u64 {
        let nonce = <MNonce>::get();
        <MNonce>::mutate(|n| *n += 1u64);
        nonce
    }
}
