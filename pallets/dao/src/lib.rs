#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{Currency, ExistenceRequirement, Randomness},
    StorageMap, StorageValue,Parameter,
};
use sp_runtime::{
    traits::{AtLeast32Bit, MaybeSerializeDeserialize, Member, One},
    DispatchResult,RuntimeDebug
};
use frame_system::ensure_signed;
use pallet_randomness_collective_flip as randomness;
use sp_io::hashing::blake2_256;
use sp_std::prelude::*;
use utilities::{
	Proposal
};
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
#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct ProposalId {
    pub id: [u8; 16],
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

        NewProposal(AccountId),
        
        VoteProposal(AccountId),
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
    trait Store for Module<T: Config> as Dao {

        // DNFTDAO
        pub DAOAcc get(fn dao_acc): T::AccountId;
        pub DAOTax get(fn dao_tax): BalanceOf<T>;

        // Proposal
        pub Proposals get(fn proposals): map hasher(blake2_128_concat) ProposalId => Option<Proposal<T::AccountId>>;
        pub ProposalsCount get(fn proposals_count): u64;
        pub ProposalsIndex get(fn proposals_index): map hasher(blake2_128_concat) u64 => ProposalId;
        
        pub MemberProposals get(fn member_proposals):
		double_map hasher(blake2_128_concat) ProposalId, hasher(blake2_128_concat) T::AccountId => bool;
        pub PNonce get(fn pnonce): u64;
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


        #[weight = 10_000]
        fn create_proposal(
            origin,
            name: Vec<u8>,
            content: Vec<u8>,
            min_to_succeed: u64,
            deadline: u64,
        ) {
            let from = ensure_signed(origin)?;

            let nonce = <PNonce>::get();
            <PNonce>::mutate(|n| *n += 1u64);
            let random_seed = <randomness::Module<T>>::random_seed();
            let encoded = (random_seed, creator.clone(), nonce).encode();
            let id = blake2_256(&encoded);
            let new_class_id = ProposalId { id };
            let new_proposal = Proposal {
                owner: from.clone(),
                name: name.clone(),
                content: content.clone(),
                min_to_succeed: min_to_succeed.clone(),
                vote_yes: 0,
                vote_no: 0,
                deadline: deadline.clone(),
            };

            <Proposals<T>>::insert(new_proposal_id.clone(), new_proposal.clone());
            <ProposalsCount<T>>::put(nonce.clone() + 1);
            <ProposalsIndex<T>>::insert(nonce.clone(), new_proposal_id.clone());
            Self::deposit_event(RawEvent::NewProposal(
                from,
            ));
        }

        #[weight = 10_000 ]
        fn vote(
            origin,
            pid: ProposalId,
            vote: bool,
        ) {
            let voter = ensure_signed(origin)?;
            ensure!(Self::member_proposals(pid.clone(),voter.clone())==false, Error::<T>::NoPermission);
            if let Some(mut proposal) = Self::proposals(pid.clone()) {
                if vote{
                    proposal.vote_yes +=1;
                }else{
                    proposal.vote_no +=1;
                }
                <Proposals<T>>::insert(&pid, &proposal);
            }
            <MemberProposals<T>>::insert(pid.clone(), voter.clone(), true);
            Self::deposit_event(RawEvent::VoteProposal(
                voter,
            ));
        }

    }
}

