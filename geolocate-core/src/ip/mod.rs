use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::RangeInclusive;

/// The IPv4-specific API.
pub mod v4;
/// The IPv6-specific API.
pub mod v6;

/// A trait that ensures that [`IpAddrBlock<A>`] can only be used for IP types.
trait SealedIpAddr: Copy + Ord {}

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
pub struct IpAddrBlock<A: SealedIpAddr>(A, A);

impl<A: SealedIpAddr> IpAddrBlock<A> {
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

impl<A: SealedIpAddr> PartialEq<A> for IpAddrBlock<A> {
    #[inline]
    fn eq(&self, other: &A) -> bool {
        self.range().contains(other)
    }
}

impl<A: SealedIpAddr> PartialOrd<A> for IpAddrBlock<A> {
    fn partial_cmp(&self, other: &A) -> Option<Ordering> {
        Some(match other {
            v if self.eq(v) => Ordering::Equal,
            v if &self.1 < v => Ordering::Less,
            v if &self.0 > v => Ordering::Greater,
            _ => unreachable!("value could not be compared"),
        })
    }
}

impl<A: SealedIpAddr> From<A> for IpAddrBlock<A> {
    #[inline]
    fn from(value: A) -> Self {
        // Because blocks use inclusive ranges, this is not considered empty.
        Self(value, value)
    }
}

impl<A: SealedIpAddr> TryFrom<(A, A)> for IpAddrBlock<A> {
    type Error = EmptyBlockError;

    #[inline]
    fn try_from((start, end): (A, A)) -> Result<Self, Self::Error> {
        Self::try_new(start, end)
    }
}

impl<A: SealedIpAddr, const N: usize> TryFrom<[A; N]> for IpAddrBlock<A> {
    type Error = EmptyBlockError;

    #[inline]
    fn try_from(mut value: [A; N]) -> Result<Self, Self::Error> {
        Self::from_mut_slice(&mut value)
    }
}

impl<A: SealedIpAddr> TryFrom<&mut [A]> for IpAddrBlock<A> {
    type Error = EmptyBlockError;

    #[inline]
    fn try_from(value: &mut [A]) -> Result<Self, Self::Error> {
        Self::from_mut_slice(value)
    }
}

impl<A: SealedIpAddr> TryFrom<Box<[A]>> for IpAddrBlock<A> {
    type Error = EmptyBlockError;

    #[inline]
    fn try_from(mut value: Box<[A]>) -> Result<Self, Self::Error> {
        Self::from_mut_slice(&mut value)
    }
}
