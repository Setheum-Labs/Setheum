use codec::FullCodec;
use core::marker::PhantomData;
use frame_support::storage::{StorageMap, StorageValue};
use num_traits::{WrappingAdd, WrappingSub};

type DefaultIdx = u16;
/// Transient ringbuffer that sits on top of storage.
pub struct BoundedDeque<Item, B, M, Index = DefaultIdx>
where
	Item: FullCodec,
	B: StorageValue<(Index, Index), Query = (Index, Index)>,
	M: StorageMap<Index, Item, Query = Item>,
	Index: FullCodec + Eq + Ord + WrappingAdd + WrappingSub + From<u8> + Copy,
{
	start: Index,
	length: Index,
	_phantom: PhantomData<(Item, B, M)>,
}

/// Ringbuffer implementation.
// B, M and Index are generic typs that satisfu the storage type traits
impl<Item, B, M, Index> BoundedDeque<Item, B, M, Index>
where
	Item: FullCodec,
	B: StorageValue<(Index, Index), Query = (Index, Index)>,
	M: StorageMap<Index, Item, Query = Item>,
	Index: FullCodec + Eq + Ord + WrappingAdd + WrappingSub + From<u8> + Copy,
{
	/// Create a new `BoundedDeque` based on the storage types.
	///
	/// Initializes itself from the `Bounds` storage.
	pub fn new() -> BoundedDeque<Item, B, M, Index> {
		let (start, length) = B::get();
		BoundedDeque {
			start,
			length,
			_phantom: PhantomData,
		}
	}

	/// Create a new `BoundedDeque` based on passed bounds.
	/// 
	/// Bounds are assumed to be valid.
	pub fn from_bounds(start: Index, length: Index) -> BoundedDeque<Item, B, M, Index> {
		BoundedDeque {
			start,
			length,
			_phantom: PhantomData,
		}
	}

	/// Get the index past the last element.
	fn end(&self) -> Index {
		self.start.wrapping_add(&self.length)
	}

	/// Push an item onto the back of the queue.
	///
	/// + Will write over the item at the front if the queue is full.
	/// + Will insert the new item into storage, but will not update the bounds in storage.
	pub fn push_back(&mut self, item: Item) {
		let index = self.end();
		M::insert(index, item);
		// this will intentionally overflow and wrap around when the end
		// reaches `Index::max_value` because we want a ringbuffer.
		let new_end = index.wrapping_add(&Index::from(1));
		if new_end == self.start {
			// queue is full and thus writing over the front item
			self.start = self.start.wrapping_add(&Index::from(1));
		}
		// simulate saturating add
		self.length = Index::max(self.length, self.length.wrapping_add(&Index::from(1)));
	}

	/// Push an item onto the front of the queue.
	/// 
	/// + Will write over the item at the back if the queue is full.
	/// + Will insert the new item into storage, but will not update the bounds in storage.
	pub fn push_front(&mut self, item: Item) {
		let index = self.start.wrapping_sub(&Index::from(1));
		M::insert(index, item);
		self.start = index;
		// simulate saturating add
		self.length = Index::max(self.length, self.length.wrapping_add(&Index::from(1)));
	}

	/// Pop an item from the back of the queue.
	/// 
	/// Will remove the item from storage, but will not update the bounds in storage.
	pub fn pop_back(&mut self) -> Option<Item> {
		if self.is_empty() {
			return None;
		}
		let item = M::take(self.end().wrapping_sub(&Index::from(1)));
		self.length = self.length - Index::from(1);

		item.into()
	}

	/// Pop an item from the front of the queue.
	/// 
	/// Will remove the item from storage, but will not update the bounds in storage.
	pub fn pop_front(&mut self) -> Option<Item> {
		if self.is_empty() {
			return None;
		}
		let item = M::take(self.start);
		self.start = self.start.wrapping_add(&Index::from(1));
		self.length = self.length - Index::from(1);

		item.into()
	}

	/// Return whether to consider the queue empty.
	pub fn is_empty(&self) -> bool {
		self.length == Index::from(0)
	}

	/// Commit the potentially changed bounds to storage.
	/// 
	/// Note: Is called on `drop`, so usually does need to be called explicitly.
	pub fn commit(&self) {
		B::put((self.start, self.length));
	}
}

impl<Item, B, M, Index> Drop for BoundedDeque<Item, B, M, Index>
where
	Item: FullCodec,
	B: StorageValue<(Index, Index), Query = (Index, Index)>,
	M: StorageMap<Index, Item, Query = Item>,
	Index: FullCodec + Eq + Ord + WrappingAdd + WrappingSub + From<u8> + Copy,
{
	/// Commit on `drop`.
	fn drop(&mut self) {
		self.commit();
	}
}