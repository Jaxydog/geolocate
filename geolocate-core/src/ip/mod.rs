use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::RangeInclusive;

/// The IPv4-specific API.
pub mod v4;
/// The IPv6-specific API.
pub mod v6;

/// A trait that allows a type of be used within an [`IpAddrBlock<A>`].
pub trait Address: Copy + Ord {}

/// A type that allows values to be mapped to IP address blocks.
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct IpAddrBlockMap<A: Address, T> {
    inner: Vec<(IpAddrBlock<A>, T)>,
    dirty: bool,
}

impl<A: Address, T> IpAddrBlockMap<A, T> {
    /// Creates a new [`IpAddrBlockMap<A, T>`].
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { inner: Vec::new(), dirty: false }
    }

    /// Creates a new [`IpAddrBlockMap<A, T>`] with the given capacity.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { inner: Vec::with_capacity(capacity), dirty: false }
    }

    /// Returns whether this block map contains the given IP address.
    ///
    /// # Panics
    ///
    /// Panics if the map is unable to properly search through its inner IP blocks.
    pub fn contains_address(&self, address: A) -> bool {
        debug_assert!(!self.dirty, "attempted to read from the map without normalizing");

        self.inner.binary_search_by(|(b, _)| b.partial_cmp(&address).expect("unable to search")).is_ok()
    }

    /// Returns whether this block map contains the given IP address block.
    pub fn contains_block(&self, block: IpAddrBlock<A>) -> bool {
        debug_assert!(!self.dirty, "attempted to read from the map without normalizing");

        self.inner.binary_search_by_key(&block, |(b, _)| *b).is_ok()
    }

    /// Returns the number of entries within the map.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns whether the map is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns a value associated with the given IP address.
    ///
    /// # Panics
    ///
    /// Panics if the map is unable to properly search through its inner IP blocks.
    pub fn get_from_address(&self, address: A) -> Option<&T> {
        debug_assert!(!self.dirty, "attempted to read from the map without normalizing");

        let index = self.inner.binary_search_by(|(b, _)| {
            // This should never fail, assuming the PartialOrd impl is correct.
            b.partial_cmp(&address).expect("unable to search")
        });

        self.inner.get(index.ok()?).map(|(_, v)| v)
    }

    /// Returns a value associated with the given IP address.
    ///
    /// # Panics
    ///
    /// Panics if the map is unable to properly search through its inner IP blocks.
    pub fn get_from_address_mut(&mut self, address: A) -> Option<&mut T> {
        debug_assert!(!self.dirty, "attempted to read from the map without normalizing");

        let index = self.inner.binary_search_by(|(b, _)| {
            // This should never fail, assuming the PartialOrd impl is correct.
            b.partial_cmp(&address).expect("unable to search")
        });

        self.inner.get_mut(index.ok()?).map(|(_, v)| v)
    }

    /// Returns a value associated with the given IP address block.
    pub fn get_from_block(&self, block: IpAddrBlock<A>) -> Option<&T> {
        debug_assert!(!self.dirty, "attempted to read from the map without normalizing");

        let index = self.inner.binary_search_by_key(&block, |(b, _)| *b);

        self.inner.get(index.ok()?).map(|(_, v)| v)
    }

    /// Returns a value associated with the given IP address block.
    pub fn get_from_block_mut(&mut self, block: IpAddrBlock<A>) -> Option<&mut T> {
        debug_assert!(!self.dirty, "attempted to read from the map without normalizing");

        let index = self.inner.binary_search_by_key(&block, |(b, _)| *b);

        self.inner.get_mut(index.ok()?).map(|(_, v)| v)
    }

    /// Normalizes the internal map of this [`IpAddrBlockMap<A, T>`].
    pub fn normalize(&mut self) {
        self.inner.dedup_by(|(a, _), (b, _)| a == b);
        self.inner.sort_unstable_by_key(|(b, _)| (b.0, b.1));
        self.inner.shrink_to_fit();

        self.dirty = false;
    }

    /// Inserts a block-assigned value into the map, without ensuring that it is sorted afterwards.
    ///
    /// There is no guarantee that after this method is called the inner map will be sorted.
    ///
    /// This function will only ever return a value during insertion if the map *is* sorted properly.
    ///
    /// # Safety
    ///
    /// You must manually ensure that, before calling any method that attempts to search the map, that the inner map is
    /// sorted. This can be done using [`normalize`](<IpAddrBlockMap::normalize>).
    pub unsafe fn insert_unstable(&mut self, block: IpAddrBlock<A>, value: T) -> Option<T> {
        let index = if self.dirty { Err(0) } else { self.inner.binary_search_by_key(&block, |(b, _)| *b) };
        let previous = index.ok().map(|i| self.inner.swap_remove(i).1);

        self.inner.push((block, value));
        self.dirty = true;

        previous
    }

    /// Removes a block-assigned value from the map, without ensuring that it is sorted afterwards.
    ///
    /// There is no guarantee that after this method is called the inner map will be sorted.
    ///
    /// # Safety
    ///
    /// You must manually ensure that, both before calling this method, and before calling any method that attempts to
    /// search the map, that the inner map is sorted. This can be done using [`normalize`](<IpAddrBlockMap::normalize>).
    pub unsafe fn remove_unstable(&mut self, block: IpAddrBlock<A>) -> Option<T> {
        debug_assert!(!self.dirty, "attempted to read from the map without normalizing");

        let index = self.inner.binary_search_by_key(&block, |(b, _)| *b).ok()?;

        Some(self.inner.swap_remove(index).1)
    }

    /// Inserts a block-assigned value into the map, returning the previous value if present.
    pub fn insert(&mut self, block: IpAddrBlock<A>, value: T) -> Option<T> {
        // Ensure that we normalize so that the return value works properly.
        if self.dirty {
            self.normalize();
        }

        match self.inner.binary_search_by_key(&block, |(b, _)| *b) {
            Ok(index) => Some(std::mem::replace(&mut self.inner[index], (block, value)).1),
            Err(index) => {
                self.inner.insert(index, (block, value));
                None
            }
        }
    }

    /// Removes a block-assigned value from the map, returning it.
    pub fn remove(&mut self, block: IpAddrBlock<A>) -> Option<T> {
        // Ensure that we normalize so that the return value works properly.
        if self.dirty {
            self.normalize();
        }

        let index = self.inner.binary_search_by_key(&block, |(b, _)| *b).ok()?;

        Some(self.inner.remove(index).1)
    }

    /// Removes all elements from the map.
    pub fn clear(&mut self) {
        self.inner.clear();
        self.dirty = false;
    }

    /// Returns an iterator of references to the blocks within this map.
    pub fn blocks(&self) -> impl Iterator<Item = &IpAddrBlock<A>> {
        debug_assert!(!self.dirty, "attempted to read from the map without normalizing");

        self.inner.iter().map(|(b, _)| b)
    }

    /// Returns an iterator of references to the values within this map.
    pub fn values(&self) -> impl Iterator<Item = &T> {
        debug_assert!(!self.dirty, "attempted to read from the map without normalizing");

        self.inner.iter().map(|(_, v)| v)
    }

    /// Returns an iterator of mutable references to the values within this map.
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        debug_assert!(!self.dirty, "attempted to read from the map without normalizing");

        self.inner.iter_mut().map(|(_, v)| v)
    }

    /// Returns an iterator of references to the entries within this map.
    pub fn iter(&self) -> impl Iterator<Item = (&IpAddrBlock<A>, &T)> {
        debug_assert!(!self.dirty, "attempted to read from the map without normalizing");

        self.inner.iter().map(|(b, v)| (b, v))
    }

    /// Returns an iterator of mutable references to the entries within this map.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&IpAddrBlock<A>, &mut T)> {
        debug_assert!(!self.dirty, "attempted to read from the map without normalizing");

        self.inner.iter_mut().map(|(b, v)| (&*b, v))
    }
}

impl<A: Address, T> IntoIterator for IpAddrBlockMap<A, T> {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = (IpAddrBlock<A>, T);

    fn into_iter(self) -> Self::IntoIter {
        debug_assert!(!self.dirty, "attempted to read from the map without normalizing");

        self.inner.into_iter()
    }
}

impl<A: Address, T> FromIterator<(IpAddrBlock<A>, T)> for IpAddrBlockMap<A, T> {
    fn from_iter<I: IntoIterator<Item = (IpAddrBlock<A>, T)>>(iter: I) -> Self {
        let mut map = Self { inner: Vec::from_iter(iter), dirty: true };

        map.normalize();

        map
    }
}

impl<A: Address, T> Extend<(IpAddrBlock<A>, T)> for IpAddrBlockMap<A, T> {
    fn extend<I: IntoIterator<Item = (IpAddrBlock<A>, T)>>(&mut self, iter: I) {
        self.inner.extend(iter);
        self.normalize();
    }
}

/// An error that is returned when trying to create an [`IpAddrBlock<A>`] using an empty or overlapping address range.
#[repr(transparent)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EmptyBlockError;

impl std::error::Error for EmptyBlockError {}

impl Display for EmptyBlockError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the given range is empty or overlapping")
    }
}

/// An IP address block.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct IpAddrBlock<A: Address>(A, A);

impl<A: Address> IpAddrBlock<A> {
    /// Creates a new [`IpAddrBlock<A>`].
    ///
    /// # Panics
    ///
    /// Panics if the given start address is greater than the given end address.
    pub fn new(start: A, end: A) -> Self {
        debug_assert!(start <= end);

        Self(start, end)
    }

    /// Creates a new [`IpAddrBlock<A>`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the given start address is greater than the given end address.
    pub fn try_new(start: A, end: A) -> Result<Self, EmptyBlockError> {
        if start <= end { Ok(Self(start, end)) } else { Err(EmptyBlockError) }
    }

    /// Creates a new [`IpAddrBlock<A>`] from the given slice.
    ///
    /// # Errors
    ///
    /// This function will return an error if the the slice is empty.
    ///
    /// # Safety
    ///
    /// This function assumes that the given slice is sorted, meaning that the structure may be malformed if the slice
    /// is not sorted beforehand.
    pub const unsafe fn from_slice(slice: &[A]) -> Result<Self, EmptyBlockError> {
        let Some(start) = slice.first() else { return Err(EmptyBlockError) };
        let Some(end) = slice.first() else { return Err(EmptyBlockError) };

        Ok(Self(*start, *end))
    }

    /// Creates a new [`IpAddrBlock<A>`] from the given slice.
    ///
    /// This is a safe abstraction over [`from_slice`](<IpAddrBlock::from_slice>), but must
    /// potentially mutate the given slice to ensure safety.
    ///
    /// # Errors
    ///
    /// This function will return an error if the given slice is empty.
    pub fn from_mut_slice(slice: &mut [A]) -> Result<Self, EmptyBlockError> {
        slice.sort_unstable();

        unsafe { Self::from_slice(slice) }
    }

    /// Returns the start address of this [`IpAddrBlock<A>`].
    #[inline]
    pub const fn start(&self) -> A {
        self.0
    }

    /// Returns the end address of this [`IpAddrBlock<A>`].
    #[inline]
    pub const fn end(&self) -> A {
        self.1
    }

    /// Returns the IP range of this [`IpAddrBlock<A>`].
    #[inline]
    pub const fn range(&self) -> RangeInclusive<A> {
        self.start() ..= self.end()
    }
}

impl<A: Address> PartialEq<A> for IpAddrBlock<A> {
    #[inline]
    fn eq(&self, other: &A) -> bool {
        self.range().contains(other)
    }
}

impl<A: Address> PartialOrd<A> for IpAddrBlock<A> {
    fn partial_cmp(&self, other: &A) -> Option<Ordering> {
        Some(match other {
            v if self.eq(v) => Ordering::Equal,
            v if &self.1 < v => Ordering::Less,
            v if &self.0 > v => Ordering::Greater,
            _ => unreachable!("value could not be compared"),
        })
    }
}

impl<A: Address> From<A> for IpAddrBlock<A> {
    #[inline]
    fn from(value: A) -> Self {
        // Because blocks use inclusive ranges, this is not considered empty.
        Self(value, value)
    }
}

impl<A: Address> TryFrom<(A, A)> for IpAddrBlock<A> {
    type Error = EmptyBlockError;

    #[inline]
    fn try_from((start, end): (A, A)) -> Result<Self, Self::Error> {
        Self::try_new(start, end)
    }
}

impl<A: Address, const N: usize> TryFrom<[A; N]> for IpAddrBlock<A> {
    type Error = EmptyBlockError;

    #[inline]
    fn try_from(mut value: [A; N]) -> Result<Self, Self::Error> {
        Self::from_mut_slice(&mut value)
    }
}

impl<A: Address> TryFrom<&mut [A]> for IpAddrBlock<A> {
    type Error = EmptyBlockError;

    #[inline]
    fn try_from(value: &mut [A]) -> Result<Self, Self::Error> {
        Self::from_mut_slice(value)
    }
}

impl<A: Address> TryFrom<Box<[A]>> for IpAddrBlock<A> {
    type Error = EmptyBlockError;

    #[inline]
    fn try_from(mut value: Box<[A]>) -> Result<Self, Self::Error> {
        Self::from_mut_slice(&mut value)
    }
}
