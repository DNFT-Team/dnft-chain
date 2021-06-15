#![cfg_attr(not(feature = "std"), no_std)]

use utilities::{
	Collection, CollectionId, CollectionStatus, LifeStage, NFTId, NFTManager, NFTSId, NFTSource,
	NFTStatus, NFT, NFTS,
};
use codec::Encode;
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure,
	traits::{Get, Randomness},
	StorageMap, StorageValue,
};
use frame_system::ensure_signed;
use randomness;
use sp_io::hashing::blake2_256;
use sp_runtime::DispatchResult;
use sp_std::prelude::*;

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Config>::AccountId,
	{
		IssueNFTS(AccountId),

		MintNFT(AccountId),

		TransferSingleNFT(AccountId),

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
		IndexExceedTotalSupply,
		NotNFTOwner,
		CanNotApproveToSelf,
		NotEnoughtNFT,
		CollectionNotExist,
		NotCollectionOwner,
	}
}

decl_storage! {
	trait Store for Module<T: Config> as NFT1155 {

		// NFTS
		pub NFTSs get(fn nftss): map hasher(twox_64_concat) NFTSId => Option<NFTS<T::AccountId, T::BlockNumber>>;
		pub NFTSsCount get(fn nftss_count): u64;
		pub NFTSsIndex get(fn nftss_index): map hasher(blake2_128_concat) u64 => NFTSId;

		// Nonce
		pub Nonce get(fn nonce): u64;

		// NFT
		pub NFTs get(fn nfts): map hasher(twox_64_concat) NFTId => Option<NFT<T::AccountId, NFTSId, NFTStatus>>;
		pub NFTsCount get(fn nfts_count): u64;
		pub NFTsIndex get(fn nfts_index): map hasher(blake2_128_concat) u64 => NFTId;
		pub NFTSMemberIndex get(fn nfts_member_index): map hasher(blake2_128_concat) NFTSId => u64;
		pub MemberNFTSId get(fn member_nfts_id):
		double_map hasher(blake2_128_concat) NFTSId, hasher(blake2_128_concat) u64 => Option<NFTId>;

		// TNonce
		pub TNonce get(fn tnonce): u64;

		// Collection
		pub Collections get(fn collections): map hasher(twox_64_concat) CollectionId => Option<Collection<T::AccountId, CollectionStatus, NFTSId>>;
		pub CollectionsCount get(fn collections_count): u64;
		pub CollectionsIndex get(fn collections_index): map hasher(blake2_128_concat) u64 => CollectionId;

		// CNonce
		pub CNonce get(fn cnonce): u64;

		// owned NFT
		pub OwnedNFTS get(fn owned_nfts):  map hasher(twox_64_concat) T::AccountId => Vec<NFTSource<NFTSId>>;

		pub OwnedCollections get(fn owned_collections):  map hasher(twox_64_concat) T::AccountId => Vec<CollectionId>;

		pub OwnerToApprove get(fn is_approved_for_all): map hasher(twox_64_concat) (T::AccountId, T::AccountId) => bool;

	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,4)]
		pub fn issue_nfts(
			origin,
			name: Vec<u8>,
			symbol: Vec<u8>,
			info: Vec<u8>,
			supply: u64,
			stage: Vec<LifeStage<T::BlockNumber>>,
		) {
			let who = ensure_signed(origin)?;

			Self::_issue_nfts(name, symbol, info, supply, stage, who.clone())?;

			Self::deposit_event(RawEvent::IssueNFTS(who));
		}

		#[weight = 10_000 + T::DbWeight::get().reads_writes(5,6)]
		pub fn mint_nft(
			origin,
			nfts_id: NFTSId,
			info: Vec<u8>,
		) {
			let who = ensure_signed(origin)?;

			Self::_mint_nft(nfts_id.clone(), who.clone(), info.clone());

			Self::deposit_event(RawEvent::MintNFT(who));

		}

		#[weight = 10_000 + T::DbWeight::get().reads_writes(4,2)]
		pub fn transfer_single_nft(
			origin,
			from: T::AccountId,
			to: T::AccountId,
			nft_id: NFTId,
		) {
			let who = ensure_signed(origin)?;

			Self::_transfer_single_nft(who.clone(), from.clone(), to.clone(), nft_id.clone())?;

			Self::deposit_event(RawEvent::TransferSingleNFT(who));

		}

		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn approve_single_nft(
			origin,
			to: T::AccountId,
			nft_id: NFTId,
		) {
			let who = ensure_signed(origin)?;

			Self::_approve_single_nft(who.clone(), to.clone(), nft_id.clone())?;

			Self::deposit_event(RawEvent::ApproveSingleNFT(who));

		}

		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn destroy_single_nft(
			origin,
			nft_id: NFTId,
		) {
			let who = ensure_signed(origin)?;

			Self::_destroy_single_nft(who.clone(), nft_id.clone())?;

			Self::deposit_event(RawEvent::DestroySingleNFT(who));

		}

		#[weight = 10_000 + T::DbWeight::get().reads_writes(8,3)]
		pub fn transfer_batch_nft(
			origin,
			from: T::AccountId,
			to: T::AccountId,
			nfts_id: NFTSId,
			amount: u64,
		) {
			let who = ensure_signed(origin)?;

			Self::_transfer_batch_nft(who.clone(), from.clone(), to.clone(), nfts_id.clone(), amount.clone())?;

			Self::deposit_event(RawEvent::TransferBatchNFT(who));

		}

		#[weight = 10_000 + T::DbWeight::get().reads_writes(3,1)]
		pub fn approve_batch_nft(
			origin,
			to: T::AccountId,
			nfts_id: NFTSId,
			amount: u64,
		) {
			let who = ensure_signed(origin)?;

			Self::_approve_batch_nft(who.clone(), to.clone(), nfts_id.clone(), amount.clone())?;

			Self::deposit_event(RawEvent::ApproveBatchNFT(who));

		}

		#[weight = 10_000 + T::DbWeight::get().reads_writes(3,1)]
		pub fn destroy_batch_nft(
			origin,
			nfts_id: NFTSId,
			amount: u64,
		) {
			let who = ensure_signed(origin)?;

			Self::_destroy_batch_nft(who.clone(), nfts_id.clone(), amount.clone())?;

			Self::deposit_event(RawEvent::DestroyBatchNFT(who));

		}

		#[weight = 10_000 + T::DbWeight::get().reads_writes(2,3)]
		pub fn coupled_collection(
			origin,
			name: Vec<u8>,
			symbol: Vec<u8>,
			info: Vec<u8>,
			source: Vec<NFTSource<NFTSId>>,
		) {
			let who = ensure_signed(origin)?;

			Self::_coupled_collection(name.clone(), symbol.clone(), who.clone(), info.clone(), source.clone())?;

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


	}
}

// NFTS
impl<T: Config> Module<T> {
	fn _issue_nfts(
		name: Vec<u8>,
		symbol: Vec<u8>,
		info: Vec<u8>,
		supply: u64,
		stage: Vec<LifeStage<T::BlockNumber>>,
		issuer: T::AccountId,
	) -> DispatchResult {
		let nonce = Self::get_nonce();
		let random_seed = <randomness::Module<T>>::random_seed();
		let encoded = (random_seed, issuer.clone(), nonce).encode();
		let did = blake2_256(&encoded);
		let new_nfts_id = NFTSId { did };
		let new_nfts = NFTS {
			name: name.clone(),
			symbol: symbol.clone(),
			info: info.clone(),
			supply: supply.clone(),
			stage: stage.clone(),
			issuer: issuer.clone(),
		};

		<NFTSs<T>>::insert(new_nfts_id.clone(), &new_nfts);
		<NFTSsCount>::put(nonce.clone() + 1);
		<NFTSsIndex>::insert(nonce.clone(), new_nfts_id.clone());

		Ok(())
	}

	// nonce
	fn get_nonce() -> u64 {
		let nonce = <Nonce>::get();
		<Nonce>::mutate(|n| *n += 1u64);
		nonce
	}
}

// NFT
impl<T: Config> Module<T> {
	fn _mint_nft(nfts_id: NFTSId, miner: T::AccountId, info: Vec<u8>) -> Option<NFTId> {
		if let Some(nfts) = Self::nftss(nfts_id.clone()) {
			let nfts_index = Self::nfts_member_index(nfts_id.clone()) + 1;
			if nfts.supply >= nfts_index {
				let tnonce = Self::get_tnonce();
				let random_seed = <randomness::Module<T>>::random_seed();
				let encoded = (random_seed, miner.clone(), tnonce).encode();
				let did = blake2_256(&encoded);
				let new_nft_id = NFTId { did };

				let new_nft = NFT {
					nfts_id: nfts_id.clone(),
					nfts_index: nfts_index.clone(),
					info: info.clone(),
					owner: miner.clone(),
					status: NFTStatus::Normal,
					approvers: Vec::new(),
				};

				<NFTs<T>>::insert(new_nft_id.clone(), &new_nft);
				<NFTsCount>::put(tnonce.clone() + 1);
				<NFTsIndex>::insert(tnonce.clone(), new_nft_id.clone());
				<NFTSMemberIndex>::insert(nfts_id.clone(), nfts_index.clone());
				<MemberNFTSId>::insert(nfts_id.clone(), nfts_index.clone(), new_nft_id.clone());

				Self::_add_nft_to_owned_nfts(miner.clone(), nfts_id.clone(), nfts_index.clone());
				return Some(new_nft_id);
			}
		}
		None
	}

	fn _transfer_single_nft(
		who: T::AccountId,
		from: T::AccountId,
		to: T::AccountId,
		nft_id: NFTId,
	) -> DispatchResult {
		let mut nft = Self::nfts(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;
		ensure!(nft.owner == from.clone(), Error::<T>::NotNFTOwner);

		let is_legal = NFTStatus::Normal == nft.status;
		let is_owner = who == nft.owner;
		let is_approved = nft.approvers.contains(&who);
		let is_approved_for_all = Self::is_approved_for_all((from.clone(), who.clone()));

		ensure!(
			is_legal || is_owner || is_approved || is_approved_for_all,
			Error::<T>::NoPermission
		);

		nft.owner = to.clone();
		<NFTs<T>>::insert(nft_id.clone(), &nft);

		Self::_remove_nft_from_owned_nfts(
			from.clone(),
			nft.nfts_id.clone(),
			nft.nfts_index.clone(),
		)?;
		Self::_add_nft_to_owned_nfts(to.clone(), nft.nfts_id.clone(), nft.nfts_index.clone());
		Ok(())
	}

	fn _approve_single_nft(from: T::AccountId, to: T::AccountId, nft_id: NFTId) -> DispatchResult {
		let mut nft = Self::nfts(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;
		ensure!(nft.owner == from.clone(), Error::<T>::NoPermission);
		nft.approvers.push(to.clone());
		<NFTs<T>>::insert(nft_id.clone(), &nft);

		Ok(())
	}

	fn _destroy_single_nft(who: T::AccountId, nft_id: NFTId) -> DispatchResult {
		let mut nft = Self::nfts(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;
		ensure!(nft.owner == who.clone(), Error::<T>::NoPermission);
		nft.status = NFTStatus::Destroyed;
		<NFTs<T>>::insert(nft_id.clone(), &nft);

		Ok(())
	}

	fn _collection_single_nft(who: T::AccountId, nft_id: NFTId) -> DispatchResult {
		let mut nft = Self::nfts(nft_id.clone()).ok_or(Error::<T>::NFTNotExist)?;
		ensure!(nft.owner == who.clone(), Error::<T>::NoPermission);
		nft.status = NFTStatus::InCollection;
		<NFTs<T>>::insert(nft_id.clone(), &nft);

		Ok(())
	}

	fn _transfer_batch_nft(
		who: T::AccountId,
		from: T::AccountId,
		to: T::AccountId,
		nfts_id: NFTSId,
		amount: u64,
	) -> DispatchResult {
		let owned_nfts = Self::owned_nfts(from.clone());
		for i in 0..owned_nfts.len() {
			if owned_nfts[i].nfts_id == nfts_id.clone() {
				ensure!(
					owned_nfts[i].amount >= amount.clone(),
					Error::<T>::NotEnoughtNFT
				);
				for j in 0..owned_nfts[i].nfts_indexs.len() {
					let nft_id =
						Self::member_nfts_id(nfts_id.clone(), owned_nfts[i].nfts_indexs[j].clone())
							.ok_or(Error::<T>::NFTNotExist)?;

					Self::_transfer_single_nft(who.clone(), from.clone(), to.clone(), nft_id)?;
				}
			}
		}
		Ok(())
	}

	fn _approve_batch_nft(
		who: T::AccountId,
		to: T::AccountId,
		nfts_id: NFTSId,
		amount: u64,
	) -> DispatchResult {
		let owned_nfts = Self::owned_nfts(who.clone());
		for i in 0..owned_nfts.len() {
			if owned_nfts[i].nfts_id == nfts_id.clone() {
				ensure!(
					owned_nfts[i].amount >= amount.clone(),
					Error::<T>::NotEnoughtNFT
				);
				for j in 0..owned_nfts[i].nfts_indexs.len() {
					let nft_id =
						Self::member_nfts_id(nfts_id.clone(), owned_nfts[i].nfts_indexs[j].clone())
							.ok_or(Error::<T>::NFTNotExist)?;

					Self::_approve_single_nft(who.clone(), to.clone(), nft_id)?;
				}
			}
		}
		Ok(())
	}

	fn _destroy_batch_nft(who: T::AccountId, nfts_id: NFTSId, amount: u64) -> DispatchResult {
		let owned_nfts = Self::owned_nfts(who.clone());
		for i in 0..owned_nfts.len() {
			if owned_nfts[i].nfts_id == nfts_id.clone() {
				ensure!(
					owned_nfts[i].amount >= amount.clone(),
					Error::<T>::NotEnoughtNFT
				);
				for j in 0..owned_nfts[i].nfts_indexs.len() {
					let nft_id =
						Self::member_nfts_id(nfts_id.clone(), owned_nfts[i].nfts_indexs[j].clone())
							.ok_or(Error::<T>::NFTNotExist)?;

					Self::_destroy_single_nft(who.clone(), nft_id)?;
				}
			}
		}
		Ok(())
	}

	fn _collection_batch_nft(who: T::AccountId, nfts_id: NFTSId, amount: u64) -> DispatchResult {
		let owned_nfts = Self::owned_nfts(who.clone());
		for i in 0..owned_nfts.len() {
			if owned_nfts[i].nfts_id == nfts_id.clone() {
				ensure!(
					owned_nfts[i].amount >= amount.clone(),
					Error::<T>::NotEnoughtNFT
				);
				for j in 0..owned_nfts[i].nfts_indexs.len() {
					let nft_id =
						Self::member_nfts_id(nfts_id.clone(), owned_nfts[i].nfts_indexs[j].clone())
							.ok_or(Error::<T>::NFTNotExist)?;

					Self::_collection_single_nft(who.clone(), nft_id)?;
				}
			}
		}
		Ok(())
	}

	fn _add_nft_to_owned_nfts(owner: T::AccountId, nfts_id: NFTSId, nfts_index: u64) {
		if let Some(_nft_id) = Self::member_nfts_id(nfts_id.clone(), nfts_index.clone()) {
			let mut owned_nfts = Self::owned_nfts(owner.clone());
			let mut nfts_exist_flag = false;
			let mut nft_exist_flag = false;
			for i in owned_nfts.clone() {
				if i.nfts_id == nfts_id.clone() {
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
						if owned_nfts[i].nfts_id == nfts_id.clone() {
							owned_nfts[i].amount += 1;
							owned_nfts[i].nfts_indexs.push(nfts_index.clone());
						}
					}
					<OwnedNFTS<T>>::insert(owner, owned_nfts);
				} else {
					let new_owned_nfts = NFTSource {
						nfts_id: nfts_id.clone(),
						amount: 1,
						nfts_indexs: vec![nfts_index.clone()],
					};
					owned_nfts.push(new_owned_nfts);
					<OwnedNFTS<T>>::insert(owner, owned_nfts);
				}
			}
		}
	}

	fn _remove_nft_from_owned_nfts(
		owner: T::AccountId,
		nfts_id: NFTSId,
		nfts_index: u64,
	) -> DispatchResult {
		let _nft = Self::member_nfts_id(nfts_id.clone(), nfts_index.clone())
			.ok_or(Error::<T>::NFTNotExist)?;
		let mut owned_nfts = Self::owned_nfts(owner.clone());
		let mut nfts_exist_flag = false;
		let mut nft_exist_flag = false;
		for i in owned_nfts.clone() {
			if i.nfts_id == nfts_id.clone() {
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
			if owned_nfts[i].nfts_id == nfts_id.clone() {
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
		<OwnedNFTS<T>>::insert(owner, owned_nfts);

		Ok(())
	}

	// tnonce
	fn get_tnonce() -> u64 {
		let tnonce = <TNonce>::get();
		<TNonce>::mutate(|n| *n += 1u64);
		tnonce
	}
}

// collection
impl<T: Config> Module<T> {
	fn _coupled_collection(
		name: Vec<u8>,
		symbol: Vec<u8>,
		owner: T::AccountId,
		info: Vec<u8>,
		source: Vec<NFTSource<NFTSId>>,
	) -> DispatchResult {
		for i in 0..source.len() {
			for j in 0..source[i].nfts_indexs.len() {
				let nft_id = Self::member_nfts_id(
					source[i].nfts_id.clone(),
					source[i].nfts_indexs[j].clone(),
				)
				.ok_or(Error::<T>::NFTNotExist)?;

				Self::_collection_single_nft(owner.clone(), nft_id)?;
			}
		}
		let cnonce = Self::get_cnonce();
		let random_seed = <randomness::Module<T>>::random_seed();
		let encoded = (random_seed, owner.clone(), cnonce).encode();
		let did = blake2_256(&encoded);
		let new_collection_id = CollectionId { did };
		let new_collection = Collection {
			name: name.clone(),
			symbol: symbol.clone(),
			info: info.clone(),
			owner: owner.clone(),
			source: source.clone(),
			status: CollectionStatus::Normal,
			approvers: Vec::new(),
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
		let is_approved = collection.approvers.contains(&who);
		let is_approved_for_all = Self::is_approved_for_all((from.clone(), who.clone()));

		ensure!(
			is_legal || is_owner || is_approved || is_approved_for_all,
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
		let mut collection =
			Self::collections(collection_id.clone()).ok_or(Error::<T>::CollectionNotExist)?;
		ensure!(collection.owner == from.clone(), Error::<T>::NoPermission);
		collection.approvers.push(to.clone());
		<Collections<T>>::insert(collection_id.clone(), &collection);

		Ok(())
	}

	fn _destroy_collection(from: T::AccountId, collection_id: CollectionId) -> DispatchResult {
		let mut collection =
			Self::collections(collection_id.clone()).ok_or(Error::<T>::CollectionNotExist)?;
		ensure!(collection.owner == from.clone(), Error::<T>::NoPermission);
		collection.status = CollectionStatus::Destroyed;
		<Collections<T>>::insert(collection_id.clone(), &collection);
		// todo
		// destroy nft

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

	// cnonce
	fn get_cnonce() -> u64 {
		let cnonce = <CNonce>::get();
		<CNonce>::mutate(|n| *n += 1u64);
		cnonce
	}
}

impl<T: Config> NFTManager<T::AccountId, T::BlockNumber> for Module<T> {
	// NFTS
	fn issue_nfts(
		name: Vec<u8>,
		symbol: Vec<u8>,
		info: Vec<u8>,
		supply: u64,
		stage: Vec<LifeStage<T::BlockNumber>>,
		issuer: T::AccountId,
	) -> DispatchResult {
		Self::_issue_nfts(name, symbol, info, supply, stage, issuer)
	}

	fn get_nfts(nfts_id: NFTSId) -> Option<NFTS<T::AccountId, T::BlockNumber>> {
		Self::nftss(nfts_id)
	}

	// NFT
	// Todo safeTransfer
	fn mint_nft(nfts_id: NFTSId, miner: T::AccountId, info: Vec<u8>) -> Option<NFTId> {
		Self::_mint_nft(nfts_id, miner, info)
	}

	fn get_nft(nft_id: NFTId) -> Option<NFT<T::AccountId, NFTSId, NFTStatus>> {
		Self::nfts(nft_id)
	}

	fn get_nfts_member_index(nfts_id: NFTSId) -> u64 {
		Self::nfts_member_index(nfts_id)
	}

	fn get_nft_by_index(nfts_id: NFTSId, nfts_index: u64) -> Option<NFTId> {
		Self::member_nfts_id(nfts_id, nfts_index)
	}

	fn owned_nfts(account: T::AccountId) -> Vec<NFTSource<NFTSId>> {
		Self::owned_nfts(account)
	}

	fn transfer_single_nft(
		who: T::AccountId,
		from: T::AccountId,
		to: T::AccountId,
		nft_id: NFTId,
	) -> DispatchResult {
		Self::_transfer_single_nft(who, from, to, nft_id)
	}

	fn transfer_batch_nft(
		who: T::AccountId,
		from: T::AccountId,
		to: T::AccountId,
		nfts_id: NFTSId,
		amount: u64,
	) -> DispatchResult {
		Self::_transfer_batch_nft(who, from, to, nfts_id, amount)
	}

	fn approve_single_nft(who: T::AccountId, to: T::AccountId, nft_id: NFTId) -> DispatchResult {
		Self::_approve_single_nft(who, to, nft_id)
	}

	fn approve_batch_nft(
		who: T::AccountId,
		to: T::AccountId,
		nfts_id: NFTSId,
		amount: u64,
	) -> DispatchResult {
		Self::_approve_batch_nft(who, to, nfts_id, amount)
	}

	fn destroy_single_nft(who: T::AccountId, nft_id: NFTId) -> DispatchResult {
		Self::_destroy_single_nft(who, nft_id)
	}

	fn destroy_batch_nft(who: T::AccountId, nfts_id: NFTSId, amount: u64) -> DispatchResult {
		Self::_destroy_batch_nft(who, nfts_id, amount)
	}

	// Collection
	fn coupled_collection(
		name: Vec<u8>,
		symbol: Vec<u8>,
		owner: T::AccountId,
		info: Vec<u8>,
		source: Vec<NFTSource<NFTSId>>,
	) -> DispatchResult {
		Self::_coupled_collection(name, symbol, owner, info, source)
	}

	fn get_collection(
		collection_id: CollectionId,
	) -> Option<Collection<T::AccountId, CollectionStatus, NFTSId>> {
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
