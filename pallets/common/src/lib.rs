#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use codec::Encode;
use frame_support::{decl_error, decl_event, decl_module, decl_storage, traits::Randomness};
use frame_system::ensure_signed;
use randomness;
use sp_core::H256;
use sp_io::hashing::blake2_256;
use sp_std::{
	cmp::{Eq, PartialEq},
	prelude::*,
};
use utilities::{BufferIndex, CommonManager, Did, ValueStruct};

mod ringbuffer;

use ringbuffer::{RingBufferTrait, RingBufferTransient};

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_storage! {
	trait Store for Module<T: Config> as Common {
		pub GenDid get(fn gen_did): Did;
		pub GenHash get(fn gen_hash): H256;
		pub BufferCount get(fn get_buffer_count): u32;
		pub BufferMap get(fn get_value): map hasher(twox_64_concat) BufferIndex => ValueStruct;
		pub BufferRange get(fn range): (BufferIndex, BufferIndex) = (0, 0);
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Config>::AccountId,
	{
		GenerateDid(AccountId, u64, Did),
		GenerateHash(AccountId, u64, H256),
		Popped(u32, bool),
		DummyEvent(AccountId),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		// AlreadyInitialized,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		#[weight = 10_000]
		fn generate_did(origin, nonce: u64)  {
			let sender = ensure_signed(origin)?;
			let did = Self::_generate_did(sender.clone(), nonce.clone());
			GenDid::put(did.clone());
			Self::deposit_event(RawEvent::GenerateDid(sender, nonce, did));
		}
		#[weight = 10_000]
		fn generate_hash(origin, nonce: u64)  {
			let sender = ensure_signed(origin)?;
			let hash = Self::_generate_hash(sender.clone(), nonce.clone());
			GenHash::put(hash.clone());
			Self::deposit_event(RawEvent::GenerateHash(sender, nonce, hash));
		}
		/// Add an item to the queue
		#[weight = 10_000]
		pub fn add_to_queue(origin, id: u32, integer: u32, boolean: bool)  {
			// only a user can push into the queue
			let _user = ensure_signed(origin)?;

			Self::_add_to_queue(id, integer, boolean);
		}

		/// Add several items to the queue
		#[weight = 10_000]
		pub fn add_multiple(origin, id: u32, integers: Vec<u32>, boolean: bool)  {
			// only a user can push into the queue
			let _user = ensure_signed(origin)?;

			Self::_add_multiple(id, integers, boolean);

		}

		/// Remove and return an item from the queue
		#[weight = 10_000]
		pub fn pop_from_queue(origin, id: u32)  {
			// only a user can pop from the queue
			let _user = ensure_signed(origin)?;

			Self::_pop_from_queue(id);

		}
	}
}
/// RingBuffer
impl<T: Config> Module<T> {
	fn _add_to_queue(_id: u32, integer: u32, boolean: bool) {
		let mut queue = Self::_queue_transient();
		queue.push(ValueStruct { integer, boolean });
	}
	fn _add_multiple(_id: u32, integers: Vec<u32>, boolean: bool) {
		let mut queue = Self::_queue_transient();
		for integer in integers {
			queue.push(ValueStruct { integer, boolean });
		}
	}
	fn _pop_from_queue(_id: u32) {
		let mut queue = Self::_queue_transient();
		if let Some(ValueStruct { integer, boolean }) = queue.pop() {
			Self::deposit_event(RawEvent::Popped(integer, boolean));
		}
	}
	/// Constructor function so we don't have to specify the types every time.
	///
	/// Constructs a ringbuffer transient and returns it as a boxed trait object.
	/// See [this part of the Rust book](https://doc.rust-lang.org/book/ch17-02-trait-objects.html#trait-objects-perform-dynamic-dispatch)
	fn _queue_transient() -> Box<dyn RingBufferTrait<ValueStruct>> {
		Box::new(RingBufferTransient::<
			ValueStruct,
			<Self as Store>::BufferRange,
			<Self as Store>::BufferMap,
			BufferIndex,
		>::new())
	}
}
/// Did
impl<T: Config> Module<T> {
	fn _generate_did(from: T::AccountId, nonce: u64) -> Did {
		let random_seed = <randomness::Module<T>>::random_seed();
		let encoded = (random_seed, from.clone(), nonce).encode();
		let did = blake2_256(&encoded);
		Did { did }
	}
	fn _generate_hash(from: T::AccountId, nonce: u64) -> H256 {
		let random_seed = <randomness::Module<T>>::random_seed();
		let encoded = (random_seed, from.clone(), nonce).encode();
		H256::from_slice(&encoded)
	}
}
impl<T: Config> CommonManager<T::AccountId> for Module<T> {
	/// did
	fn generate_did(from: T::AccountId, nonce: u64) -> Did {
		Self::_generate_did(from, nonce)
	}
	fn generate_hash(from: T::AccountId, nonce: u64) -> H256 {
		Self::_generate_hash(from, nonce)
	}
	/// ringbuffer
	fn add_to_queue(id: u32, integer: u32, boolean: bool) {
		Self::_add_to_queue(id, integer, boolean);
	}
	fn add_multiple(id: u32, integers: Vec<u32>, boolean: bool) {
		Self::_add_multiple(id, integers, boolean);
	}
	fn pop_from_queue(id: u32) {
		Self::_pop_from_queue(id);
	}
	fn get_buffer_range(_id: u32) -> (BufferIndex, BufferIndex) {
		Self::range()
	}
	fn get_buffer_value(_id: u32, index: BufferIndex) -> ValueStruct {
		Self::get_value(index)
	}
}
