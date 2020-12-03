use sp_std::prelude::*;

use codec::FullCodec;
use core::marker::PhantomData;
use frame_support::{storage::StorageValue, traits::Get};
use core::cmp::Ord;

/// Queue backing data that is the backbone of the trait object.
pub struct BoundedPriorityQueue<Item, Storage, MaxLength>
where
	Item: FullCodec + Ord + Clone,
	Storage: StorageValue<Vec<Item>, Query = Vec<Item>>,
	MaxLength: Get<u64>,
{
	items: Vec<Item>,
	_phantom: PhantomData<(Storage, MaxLength)>,
}

impl<Item, Storage, MaxLength> BoundedPriorityQueue<Item, Storage, MaxLength>
where
	Item: FullCodec + Ord + Clone,
	Storage: StorageValue<Vec<Item>, Query = Vec<Item>>,
	MaxLength: Get<u64>,
{
	/// Create a new `BoundedPriorityQueue`.
	///
	/// Initializes itself from storage with the `Storage` type.
	pub fn new() -> BoundedPriorityQueue<Item, Storage, MaxLength> {
		let items = Storage::get();
		BoundedPriorityQueue {
			items,
			_phantom: PhantomData,
		}
	}

	/// Sort a new item into the queue according to its priority.
	/// 
	/// Will return the smallest (according to `Ord`) item if length increases
	/// over `MaxLength` otherwise.
	// TODO: This could be abused by an attacker kicking out other items with the same
	//       value.
	pub fn push(&mut self, item: Item) -> Option<Item> {
		let index = self
			.items
			.binary_search_by(|it| it.cmp(&item))
			.unwrap_or_else(|i| i);
		self.items.insert(index, item);
		if self.items.len() as u64 > MaxLength::get() {
			return Some(self.items.remove(0));
		}
		None
	}

	/// Pop the greatest item from the queue.
	///
	/// Returns `None` if the queue is empty.
	pub fn pop(&mut self) -> Option<Item> {
		self.items.pop()
	}

	/// Return whether the queue is empty.
	pub fn is_empty(&self) -> bool {
		self.items.is_empty()
	}

	/// Commit the potentially changed backing `Vec` to storage.
	pub fn commit(&mut self) {
		Storage::put(self.items.clone());
	}
}

impl<Item, Storage, MaxLength> Drop for BoundedPriorityQueue<Item, Storage, MaxLength>
where
	Item: FullCodec + Ord + Clone,
	Storage: StorageValue<Vec<Item>, Query = Vec<Item>>,
	MaxLength: Get<u64>,
{
	/// Commit on `drop`.
	fn drop(&mut self) {
		self.commit();
	}
}