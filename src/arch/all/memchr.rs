/*!
Provides architecture independent implementations of `memchr` and friends.

The main types in this module are [`One`], [`Two`] and [`Three`]. They are for
searching for one, two or three distinct bytes, respectively, in a haystack.
Each type also has corresponding double ended iterators. These searchers
are typically slower than hand-coded vector routines accomplishing the same
task, but are also typically faster than naive scalar code. These routines
effectively work by treating a `usize` as a vector of 8-bit lanes, and thus
achieves some level of data parallelism even without explicit vector support.

The `One` searcher also provides a [`One::count`] routine for efficiently
counting the number of times a single byte occurs in a haystack. This is
useful, for example, for counting the number of lines in a haystack. This
routine exists because it is usually faster, especially with a high match
count, than using [`One::find`] repeatedly. ([`OneIter`] specializes its
`Iterator::count` implementation to use this routine.)

Only one, two and three bytes are supported because three bytes is about
the point where one sees diminishing returns. Beyond this point and it's
probably (but not necessarily) better to just use a simple `[bool; 256]` array
or similar. However, it depends mightily on the specific work-load and the
expected match frequency.
*/

use crate::{arch::generic::memchr as generic, ext::Pointer};

/// The number of bytes in a single `usize` value.
const USIZE_BYTES: usize = (usize::BITS / 8) as usize;
/// The bits that must be zero for a `*const usize` to be properly aligned.
const USIZE_ALIGN: usize = USIZE_BYTES - 1;

/// Finds all occurrences of a single byte in a haystack.
#[derive(Clone, Copy, Debug)]
pub struct One {
    s1: u8,
    v1: usize,
}

impl One {
    /// The number of bytes we examine per each iteration of our search loop.
    const LOOP_BYTES: usize = 2 * USIZE_BYTES;

    /// Create a new searcher that finds occurrences of the byte given.
    #[inline]
    pub fn new(needle: u8) -> One {
        One { s1: needle, v1: splat(needle) }
    }

    /// A test-only routine so that we can bundle a bunch of quickcheck
    /// properties into a single macro. Basically, this provides a constructor
    /// that makes it identical to most other memchr implementations, which
    /// have fallible constructors.
    #[cfg(test)]
    pub(crate) fn try_new(needle: u8) -> Option<One> {
        Some(One::new(needle))
    }

    /// Return the first occurrence of the needle in the given haystack. If no
    /// such occurrence exists, then `None` is returned.
    ///
    /// The occurrence is reported as an offset into `haystack`. Its maximum
    /// value for a non-empty haystack is `haystack.len() - 1`.
    #[inline]
    pub fn find(&self, haystack: &[u8]) -> Option<usize> {
        // SAFETY: `find_raw` guarantees that if a pointer is returned, it
        // falls within the bounds of the start and end pointers.
        unsafe {
            generic::search_slice_with_raw(haystack, |s, e| {
                self.find_raw(s, e)
            })
        }
    }

    /// Return the last occurrence of the needle in the given haystack. If no
    /// such occurrence exists, then `None` is returned.
    ///
    /// The occurrence is reported as an offset into `haystack`. Its maximum
    /// value for a non-empty haystack is `haystack.len() - 1`.
    #[inline]
    pub fn rfind(&self, haystack: &[u8]) -> Option<usize> {
        // SAFETY: `find_raw` guarantees that if a pointer is returned, it
        // falls within the bounds of the start and end pointers.
        unsafe {
            generic::search_slice_with_raw(haystack, |s, e| {
                self.rfind_raw(s, e)
            })
        }
    }

    /// Counts all occurrences of this byte in the given haystack.
    #[inline]
    pub fn count(&self, haystack: &[u8]) -> usize {
        // SAFETY: All of our pointers are derived directly from a borrowed
        // slice, which is guaranteed to be valid.
        unsafe {
            let start = haystack.as_ptr();
            let end = start.add(haystack.len());
            self.count_raw(start, end)
        }
    }

    /// Like `find`, but accepts and returns raw pointers.
    ///
    /// When a match is found, the pointer returned is guaranteed to be
    /// `>= start` and `< end`.
    ///
    /// This routine is useful if you're already using raw pointers and would
    /// like to avoid converting back to a slice before executing a search.
    ///
    /// # Safety
    ///
    /// * Both `start` and `end` must be valid for reads.
    /// * Both `start` and `end` must point to an initialized value.
    /// * Both `start` and `end` must point to the same allocated object and
    /// must either be in bounds or at most one byte past the end of the
    /// allocated object.
    /// * Both `start` and `end` must be _derived from_ a pointer to the same
    /// object.
    /// * The distance between `start` and `end` must not overflow `isize`.
    /// * The distance being in bounds must not rely on "wrapping around" the
    /// address space.
    ///
    /// Note that callers may pass a pair of pointers such that `start >= end`.
    /// In that case, `None` will always be returned.
    #[inline]
    pub unsafe fn find_raw(
        &self,
        start: *const u8,
        end: *const u8,
    ) -> Option<*const u8> {
        if start >= end {
            return None;
        }
        let confirm = |b| self.confirm(b);
        let len = end.distance(start);
        if len < USIZE_BYTES {
            return generic::fwd_byte_by_byte(start, end, confirm);
        }

        // The start of the search may not be aligned to `*const usize`,
        // so we do an unaligned load here.
        let chunk = start.cast::<usize>().read_unaligned();
        if let Some(index) = self.index_of_needle(chunk) {
            return Some(start.add(index));
        }

        // And now we start our search at a guaranteed aligned position.
        // The first iteration of the loop below will overlap with the the
        // unaligned chunk above in cases where the search starts at an
        // unaligned offset, but that's okay as we're only here if that
        // above didn't find a match.
        let mut cur =
            start.add(USIZE_BYTES - (start.as_usize() & USIZE_ALIGN));
        debug_assert!(cur > start);
        while end.distance(cur) >= One::LOOP_BYTES {
            debug_assert_eq!(0, cur.as_usize() % USIZE_BYTES);

            let a = cur.cast::<usize>().read();
            let b = cur.add(USIZE_BYTES).cast::<usize>().read();
            if let Some(index) = self.index_of_needle(a) {
                return Some(cur.add(index));
            }
            if let Some(index) = self.index_of_needle(b) {
                return Some(cur.add(USIZE_BYTES + index));
            }
            cur = cur.add(One::LOOP_BYTES);
        }
        if end.distance(cur) > USIZE_BYTES {
            let chunk = cur.cast::<usize>().read();
            if let Some(index) = self.index_of_needle(chunk) {
                return Some(cur.add(index));
            }
            cur = cur.add(USIZE_BYTES);
        }
        debug_assert!(cur >= end.sub(USIZE_BYTES));
        cur = end.sub(USIZE_BYTES);
        let chunk = cur.cast::<usize>().read_unaligned();
        if let Some(index) = self.index_of_needle(chunk) {
            return Some(cur.add(index));
        }
        None
    }

    /// Like `rfind`, but accepts and returns raw pointers.
    ///
    /// When a match is found, the pointer returned is guaranteed to be
    /// `>= start` and `< end`.
    ///
    /// This routine is useful if you're already using raw pointers and would
    /// like to avoid converting back to a slice before executing a search.
    ///
    /// # Safety
    ///
    /// * Both `start` and `end` must be valid for reads.
    /// * Both `start` and `end` must point to an initialized value.
    /// * Both `start` and `end` must point to the same allocated object and
    /// must either be in bounds or at most one byte past the end of the
    /// allocated object.
    /// * Both `start` and `end` must be _derived from_ a pointer to the same
    /// object.
    /// * The distance between `start` and `end` must not overflow `isize`.
    /// * The distance being in bounds must not rely on "wrapping around" the
    /// address space.
    ///
    /// Note that callers may pass a pair of pointers such that `start >= end`.
    /// In that case, `None` will always be returned.
    #[inline]
    pub unsafe fn rfind_raw(
        &self,
        start: *const u8,
        end: *const u8,
    ) -> Option<*const u8> {
        if start >= end {
            return None;
        }
        let confirm = |b| self.confirm(b);
        let len = end.distance(start);
        if len < USIZE_BYTES {
            return generic::rev_byte_by_byte(start, end, confirm);
        }

        let chunk = end.sub(USIZE_BYTES).cast::<usize>().read_unaligned();
        if let Some(index) = self.rindex_of_needle(chunk) {
            return Some(end.sub(USIZE_BYTES).add(index));
        }

        let mut cur = end.sub(end.as_usize() & USIZE_ALIGN);
        debug_assert!(start <= cur && cur <= end);
        while cur.distance(start) >= One::LOOP_BYTES {
            debug_assert_eq!(0, cur.as_usize() % USIZE_BYTES);

            let a = cur.sub(2 * USIZE_BYTES).cast::<usize>().read();
            let b = cur.sub(1 * USIZE_BYTES).cast::<usize>().read();
            if let Some(index) = self.rindex_of_needle(b) {
                return Some(cur.sub(1 * USIZE_BYTES).add(index));
            }
            if let Some(index) = self.rindex_of_needle(a) {
                return Some(cur.sub(2 * USIZE_BYTES).add(index));
            }
            cur = cur.sub(One::LOOP_BYTES);
        }
        if cur > start.add(USIZE_BYTES) {
            let chunk = cur.sub(USIZE_BYTES).cast::<usize>().read();
            if let Some(index) = self.rindex_of_needle(chunk) {
                return Some(cur.sub(USIZE_BYTES).add(index));
            }
            cur = cur.sub(USIZE_BYTES);
        }
        debug_assert!(start.add(USIZE_BYTES) >= cur);
        cur = start;
        let chunk = cur.cast::<usize>().read_unaligned();
        if let Some(index) = self.rindex_of_needle(chunk) {
            return Some(cur.add(index));
        }
        None
    }

    /// Counts all occurrences of this byte in the given haystack represented
    /// by raw pointers.
    ///
    /// This routine is useful if you're already using raw pointers and would
    /// like to avoid converting back to a slice before executing a search.
    ///
    /// # Safety
    ///
    /// * Both `start` and `end` must be valid for reads.
    /// * Both `start` and `end` must point to an initialized value.
    /// * Both `start` and `end` must point to the same allocated object and
    /// must either be in bounds or at most one byte past the end of the
    /// allocated object.
    /// * Both `start` and `end` must be _derived from_ a pointer to the same
    /// object.
    /// * The distance between `start` and `end` must not overflow `isize`.
    /// * The distance being in bounds must not rely on "wrapping around" the
    /// address space.
    ///
    /// Note that callers may pass a pair of pointers such that `start >= end`.
    /// In that case, `0` will always be returned.
    #[inline]
    pub unsafe fn count_raw(&self, start: *const u8, end: *const u8) -> usize {
        if start >= end {
            return 0;
        }
        // Sadly I couldn't get the SWAR approach to work here, so we just do
        // one byte at a time for now. PRs to improve this are welcome.
        let mut ptr = start;
        let mut count = 0;
        while ptr < end {
            count += (ptr.read() == self.s1) as usize;
            ptr = ptr.offset(1);
        }
        count
    }

    /// Returns an iterator over all occurrences of the needle byte in the
    /// given haystack.
    ///
    /// The iterator returned implements `DoubleEndedIterator`. This means it
    /// can also be used to find occurrences in reverse order.
    pub fn iter<'a, 'h>(&'a self, haystack: &'h [u8]) -> OneIter<'a, 'h> {
        OneIter { searcher: self, it: generic::Iter::new(haystack) }
    }

    #[inline(always)]
    fn index_of_needle(&self, chunk: usize) -> Option<usize> {
        find_zero_in_chunk(self.v1 ^ chunk)
    }

    #[inline(always)]
    fn rindex_of_needle(&self, chunk: usize) -> Option<usize> {
        rfind_zero_in_chunk(self.v1 ^ chunk)
    }

    #[inline(always)]
    fn confirm(&self, haystack_byte: u8) -> bool {
        self.s1 == haystack_byte
    }
}

/// An iterator over all occurrences of a single byte in a haystack.
///
/// This iterator implements `DoubleEndedIterator`, which means it can also be
/// used to find occurrences in reverse order.
///
/// This iterator is created by the [`One::iter`] method.
///
/// The lifetime parameters are as follows:
///
/// * `'a` refers to the lifetime of the underlying [`One`] searcher.
/// * `'h` refers to the lifetime of the haystack being searched.
#[derive(Clone, Debug)]
pub struct OneIter<'a, 'h> {
    /// The underlying memchr searcher.
    searcher: &'a One,
    /// Generic iterator implementation.
    it: generic::Iter<'h>,
}

impl<'a, 'h> Iterator for OneIter<'a, 'h> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        // SAFETY: We rely on the generic iterator to provide valid start
        // and end pointers, but we guarantee that any pointer returned by
        // 'find_raw' falls within the bounds of the start and end pointer.
        unsafe { self.it.next(|s, e| self.searcher.find_raw(s, e)) }
    }

    #[inline]
    fn count(self) -> usize {
        self.it.count(|s, e| {
            // SAFETY: We rely on our generic iterator to return valid start
            // and end pointers.
            unsafe { self.searcher.count_raw(s, e) }
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

impl<'a, 'h> DoubleEndedIterator for OneIter<'a, 'h> {
    #[inline]
    fn next_back(&mut self) -> Option<usize> {
        // SAFETY: We rely on the generic iterator to provide valid start
        // and end pointers, but we guarantee that any pointer returned by
        // 'rfind_raw' falls within the bounds of the start and end pointer.
        unsafe { self.it.next_back(|s, e| self.searcher.rfind_raw(s, e)) }
    }
}

/// Finds all occurrences of two bytes in a haystack.
///
/// That is, this reports matches of one of two possible bytes. For example,
/// searching for `a` or `b` in `afoobar` would report matches at offsets `0`,
/// `4` and `5`.
#[derive(Clone, Copy, Debug)]
pub struct Two {
    s1: u8,
    s2: u8,
    v1: usize,
    v2: usize,
}

impl Two {
    /// Create a new searcher that finds occurrences of the two needle bytes
    /// given.
    #[inline]
    pub fn new(needle1: u8, needle2: u8) -> Two {
        Two {
            s1: needle1,
            s2: needle2,
            v1: splat(needle1),
            v2: splat(needle2),
        }
    }

    /// A test-only routine so that we can bundle a bunch of quickcheck
    /// properties into a single macro. Basically, this provides a constructor
    /// that makes it identical to most other memchr implementations, which
    /// have fallible constructors.
    #[cfg(test)]
    pub(crate) fn try_new(needle1: u8, needle2: u8) -> Option<Two> {
        Some(Two::new(needle1, needle2))
    }

    /// Return the first occurrence of one of the needle bytes in the given
    /// haystack. If no such occurrence exists, then `None` is returned.
    ///
    /// The occurrence is reported as an offset into `haystack`. Its maximum
    /// value for a non-empty haystack is `haystack.len() - 1`.
    #[inline]
    pub fn find(&self, haystack: &[u8]) -> Option<usize> {
        // SAFETY: `find_raw` guarantees that if a pointer is returned, it
        // falls within the bounds of the start and end pointers.
        unsafe {
            generic::search_slice_with_raw(haystack, |s, e| {
                self.find_raw(s, e)
            })
        }
    }

    /// Return the last occurrence of one of the needle bytes in the given
    /// haystack. If no such occurrence exists, then `None` is returned.
    ///
    /// The occurrence is reported as an offset into `haystack`. Its maximum
    /// value for a non-empty haystack is `haystack.len() - 1`.
    #[inline]
    pub fn rfind(&self, haystack: &[u8]) -> Option<usize> {
        // SAFETY: `find_raw` guarantees that if a pointer is returned, it
        // falls within the bounds of the start and end pointers.
        unsafe {
            generic::search_slice_with_raw(haystack, |s, e| {
                self.rfind_raw(s, e)
            })
        }
    }

    /// Like `find`, but accepts and returns raw pointers.
    ///
    /// When a match is found, the pointer returned is guaranteed to be
    /// `>= start` and `< end`.
    ///
    /// This routine is useful if you're already using raw pointers and would
    /// like to avoid converting back to a slice before executing a search.
    ///
    /// # Safety
    ///
    /// * Both `start` and `end` must be valid for reads.
    /// * Both `start` and `end` must point to an initialized value.
    /// * Both `start` and `end` must point to the same allocated object and
    /// must either be in bounds or at most one byte past the end of the
    /// allocated object.
    /// * Both `start` and `end` must be _derived from_ a pointer to the same
    /// object.
    /// * The distance between `start` and `end` must not overflow `isize`.
    /// * The distance being in bounds must not rely on "wrapping around" the
    /// address space.
    ///
    /// Note that callers may pass a pair of pointers such that `start >= end`.
    /// In that case, `None` will always be returned.
    #[inline]
    pub unsafe fn find_raw(
        &self,
        start: *const u8,
        end: *const u8,
    ) -> Option<*const u8> {
        if start >= end {
            return None;
        }
        let confirm = |b| self.confirm(b);
        let len = end.distance(start);
        if len < USIZE_BYTES {
            return generic::fwd_byte_by_byte(start, end, confirm);
        }

        // The start of the search may not be aligned to `*const usize`,
        // so we do an unaligned load here.
        let chunk = start.cast::<usize>().read_unaligned();
        if let Some(index) = self.index_of_needle(chunk) {
            return Some(start.add(index));
        }

        // And now we start our search at a guaranteed aligned position.
        // The first iteration of the loop below will overlap with the
        // unaligned chunk above in cases where the search starts at an
        // unaligned offset, but that's okay as we're only here if that
        // above didn't find a match.
        let mut cur =
            start.add(USIZE_BYTES - (start.as_usize() & USIZE_ALIGN));
        debug_assert!(cur > start);
        debug_assert!(end.sub(USIZE_BYTES) >= start);
        while cur < end.sub(USIZE_BYTES) {
            debug_assert_eq!(0, cur.as_usize() % USIZE_BYTES);

            let chunk = cur.cast::<usize>().read();
            if let Some(index) = self.index_of_needle(chunk) {
                return Some(cur.add(index));
            }
            cur = cur.add(USIZE_BYTES);
        }
        debug_assert!(cur >= end.sub(USIZE_BYTES) && cur <= end);
        cur = end.sub(USIZE_BYTES);
        let chunk = cur.cast::<usize>().read_unaligned();
        if let Some(index) = self.index_of_needle(chunk) {
            return Some(cur.add(index));
        }
        None
    }

    /// Like `rfind`, but accepts and returns raw pointers.
    ///
    /// When a match is found, the pointer returned is guaranteed to be
    /// `>= start` and `< end`.
    ///
    /// This routine is useful if you're already using raw pointers and would
    /// like to avoid converting back to a slice before executing a search.
    ///
    /// # Safety
    ///
    /// * Both `start` and `end` must be valid for reads.
    /// * Both `start` and `end` must point to an initialized value.
    /// * Both `start` and `end` must point to the same allocated object and
    /// must either be in bounds or at most one byte past the end of the
    /// allocated object.
    /// * Both `start` and `end` must be _derived from_ a pointer to the same
    /// object.
    /// * The distance between `start` and `end` must not overflow `isize`.
    /// * The distance being in bounds must not rely on "wrapping around" the
    /// address space.
    ///
    /// Note that callers may pass a pair of pointers such that `start >= end`.
    /// In that case, `None` will always be returned.
    #[inline]
    pub unsafe fn rfind_raw(
        &self,
        start: *const u8,
        end: *const u8,
    ) -> Option<*const u8> {
        if start >= end {
            return None;
        }
        let confirm = |b| self.confirm(b);
        let len = end.distance(start);
        if len < USIZE_BYTES {
            return generic::rev_byte_by_byte(start, end, confirm);
        }

        let chunk = end.sub(USIZE_BYTES).cast::<usize>().read_unaligned();
        if let Some(index) = self.rindex_of_needle(chunk) {
            return Some(end.sub(USIZE_BYTES).add(index));
        }

        let mut cur = end.sub(end.as_usize() & USIZE_ALIGN);
        debug_assert!(start <= cur && cur <= end);
        while cur > start.add(USIZE_BYTES) {
            debug_assert_eq!(0, cur.as_usize() % USIZE_BYTES);

            let chunk = cur.sub(USIZE_BYTES).cast::<usize>().read();
            if let Some(index) = self.rindex_of_needle(chunk) {
                return Some(cur.sub(USIZE_BYTES).add(index));
            }
            cur = cur.sub(USIZE_BYTES);
        }
        debug_assert!(cur >= start && start.add(USIZE_BYTES) >= cur);
        cur = start;
        let chunk = cur.cast::<usize>().read_unaligned();
        if let Some(index) = self.rindex_of_needle(chunk) {
            return Some(cur.add(index));
        }
        None
    }

    /// Returns an iterator over all occurrences of one of the needle bytes in
    /// the given haystack.
    ///
    /// The iterator returned implements `DoubleEndedIterator`. This means it
    /// can also be used to find occurrences in reverse order.
    pub fn iter<'a, 'h>(&'a self, haystack: &'h [u8]) -> TwoIter<'a, 'h> {
        TwoIter { searcher: self, it: generic::Iter::new(haystack) }
    }

    #[inline(always)]
    fn index_of_needle(&self, chunk: usize) -> Option<usize> {
        match (
            find_zero_in_chunk(self.v1 ^ chunk),
            find_zero_in_chunk(self.v2 ^ chunk),
        ) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    #[inline(always)]
    fn rindex_of_needle(&self, chunk: usize) -> Option<usize> {
        match (
            rfind_zero_in_chunk(self.v1 ^ chunk),
            rfind_zero_in_chunk(self.v2 ^ chunk),
        ) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    #[inline(always)]
    fn confirm(&self, haystack_byte: u8) -> bool {
        self.s1 == haystack_byte || self.s2 == haystack_byte
    }
}

/// An iterator over all occurrences of two possible bytes in a haystack.
///
/// This iterator implements `DoubleEndedIterator`, which means it can also be
/// used to find occurrences in reverse order.
///
/// This iterator is created by the [`Two::iter`] method.
///
/// The lifetime parameters are as follows:
///
/// * `'a` refers to the lifetime of the underlying [`Two`] searcher.
/// * `'h` refers to the lifetime of the haystack being searched.
#[derive(Clone, Debug)]
pub struct TwoIter<'a, 'h> {
    /// The underlying memchr searcher.
    searcher: &'a Two,
    /// Generic iterator implementation.
    it: generic::Iter<'h>,
}

impl<'a, 'h> Iterator for TwoIter<'a, 'h> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        // SAFETY: We rely on the generic iterator to provide valid start
        // and end pointers, but we guarantee that any pointer returned by
        // 'find_raw' falls within the bounds of the start and end pointer.
        unsafe { self.it.next(|s, e| self.searcher.find_raw(s, e)) }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

impl<'a, 'h> DoubleEndedIterator for TwoIter<'a, 'h> {
    #[inline]
    fn next_back(&mut self) -> Option<usize> {
        // SAFETY: We rely on the generic iterator to provide valid start
        // and end pointers, but we guarantee that any pointer returned by
        // 'rfind_raw' falls within the bounds of the start and end pointer.
        unsafe { self.it.next_back(|s, e| self.searcher.rfind_raw(s, e)) }
    }
}

/// Finds all occurrences of three bytes in a haystack.
///
/// That is, this reports matches of one of three possible bytes. For example,
/// searching for `a`, `b` or `o` in `afoobar` would report matches at offsets
/// `0`, `2`, `3`, `4` and `5`.
#[derive(Clone, Copy, Debug)]
pub struct Three {
    s1: u8,
    s2: u8,
    s3: u8,
    v1: usize,
    v2: usize,
    v3: usize,
}

impl Three {
    /// Create a new searcher that finds occurrences of the three needle bytes
    /// given.
    #[inline]
    pub fn new(needle1: u8, needle2: u8, needle3: u8) -> Three {
        Three {
            s1: needle1,
            s2: needle2,
            s3: needle3,
            v1: splat(needle1),
            v2: splat(needle2),
            v3: splat(needle3),
        }
    }

    /// A test-only routine so that we can bundle a bunch of quickcheck
    /// properties into a single macro. Basically, this provides a constructor
    /// that makes it identical to most other memchr implementations, which
    /// have fallible constructors.
    #[cfg(test)]
    pub(crate) fn try_new(
        needle1: u8,
        needle2: u8,
        needle3: u8,
    ) -> Option<Three> {
        Some(Three::new(needle1, needle2, needle3))
    }

    /// Return the first occurrence of one of the needle bytes in the given
    /// haystack. If no such occurrence exists, then `None` is returned.
    ///
    /// The occurrence is reported as an offset into `haystack`. Its maximum
    /// value for a non-empty haystack is `haystack.len() - 1`.
    #[inline]
    pub fn find(&self, haystack: &[u8]) -> Option<usize> {
        // SAFETY: `find_raw` guarantees that if a pointer is returned, it
        // falls within the bounds of the start and end pointers.
        unsafe {
            generic::search_slice_with_raw(haystack, |s, e| {
                self.find_raw(s, e)
            })
        }
    }

    /// Return the last occurrence of one of the needle bytes in the given
    /// haystack. If no such occurrence exists, then `None` is returned.
    ///
    /// The occurrence is reported as an offset into `haystack`. Its maximum
    /// value for a non-empty haystack is `haystack.len() - 1`.
    #[inline]
    pub fn rfind(&self, haystack: &[u8]) -> Option<usize> {
        // SAFETY: `find_raw` guarantees that if a pointer is returned, it
        // falls within the bounds of the start and end pointers.
        unsafe {
            generic::search_slice_with_raw(haystack, |s, e| {
                self.rfind_raw(s, e)
            })
        }
    }

    /// Like `find`, but accepts and returns raw pointers.
    ///
    /// When a match is found, the pointer returned is guaranteed to be
    /// `>= start` and `< end`.
    ///
    /// This routine is useful if you're already using raw pointers and would
    /// like to avoid converting back to a slice before executing a search.
    ///
    /// # Safety
    ///
    /// * Both `start` and `end` must be valid for reads.
    /// * Both `start` and `end` must point to an initialized value.
    /// * Both `start` and `end` must point to the same allocated object and
    /// must either be in bounds or at most one byte past the end of the
    /// allocated object.
    /// * Both `start` and `end` must be _derived from_ a pointer to the same
    /// object.
    /// * The distance between `start` and `end` must not overflow `isize`.
    /// * The distance being in bounds must not rely on "wrapping around" the
    /// address space.
    ///
    /// Note that callers may pass a pair of pointers such that `start >= end`.
    /// In that case, `None` will always be returned.
    #[inline]
    pub unsafe fn find_raw(
        &self,
        start: *const u8,
        end: *const u8,
    ) -> Option<*const u8> {
        if start >= end {
            return None;
        }
        let confirm = |b| self.confirm(b);
        let len = end.distance(start);
        if len < USIZE_BYTES {
            return generic::fwd_byte_by_byte(start, end, confirm);
        }

        // The start of the search may not be aligned to `*const usize`,
        // so we do an unaligned load here.
        let chunk = start.cast::<usize>().read_unaligned();
        if let Some(index) = self.index_of_needle(chunk) {
            return Some(start.add(index));
        }

        // And now we start our search at a guaranteed aligned position.
        // The first iteration of the loop below will overlap with the
        // unaligned chunk above in cases where the search starts at an
        // unaligned offset, but that's okay as we're only here if that
        // above didn't find a match.
        let mut cur =
            start.add(USIZE_BYTES - (start.as_usize() & USIZE_ALIGN));
        debug_assert!(cur > start);
        debug_assert!(end.sub(USIZE_BYTES) >= start);
        while cur < end.sub(USIZE_BYTES) {
            debug_assert_eq!(0, cur.as_usize() % USIZE_BYTES);

            let chunk = cur.cast::<usize>().read();
            if let Some(index) = self.index_of_needle(chunk) {
                return Some(cur.add(index));
            }
            cur = cur.add(USIZE_BYTES);
        }
        debug_assert!(cur >= end.sub(USIZE_BYTES) && cur <= end);
        cur = end.sub(USIZE_BYTES);
        let chunk = cur.cast::<usize>().read_unaligned();
        if let Some(index) = self.index_of_needle(chunk) {
            return Some(cur.add(index));
        }
        None
    }

    /// Like `rfind`, but accepts and returns raw pointers.
    ///
    /// When a match is found, the pointer returned is guaranteed to be
    /// `>= start` and `< end`.
    ///
    /// This routine is useful if you're already using raw pointers and would
    /// like to avoid converting back to a slice before executing a search.
    ///
    /// # Safety
    ///
    /// * Both `start` and `end` must be valid for reads.
    /// * Both `start` and `end` must point to an initialized value.
    /// * Both `start` and `end` must point to the same allocated object and
    /// must either be in bounds or at most one byte past the end of the
    /// allocated object.
    /// * Both `start` and `end` must be _derived from_ a pointer to the same
    /// object.
    /// * The distance between `start` and `end` must not overflow `isize`.
    /// * The distance being in bounds must not rely on "wrapping around" the
    /// address space.
    ///
    /// Note that callers may pass a pair of pointers such that `start >= end`.
    /// In that case, `None` will always be returned.
    #[inline]
    pub unsafe fn rfind_raw(
        &self,
        start: *const u8,
        end: *const u8,
    ) -> Option<*const u8> {
        if start >= end {
            return None;
        }
        let confirm = |b| self.confirm(b);
        let len = end.distance(start);
        if len < USIZE_BYTES {
            return generic::rev_byte_by_byte(start, end, confirm);
        }

        let chunk = end.sub(USIZE_BYTES).cast::<usize>().read_unaligned();
        if let Some(index) = self.rindex_of_needle(chunk) {
            return Some(end.sub(USIZE_BYTES).add(index));
        }

        let mut cur = end.sub(end.as_usize() & USIZE_ALIGN);
        debug_assert!(start <= cur && cur <= end);
        while cur > start.add(USIZE_BYTES) {
            debug_assert_eq!(0, cur.as_usize() % USIZE_BYTES);

            let chunk = cur.sub(USIZE_BYTES).cast::<usize>().read();
            if let Some(index) = self.rindex_of_needle(chunk) {
                return Some(cur.sub(USIZE_BYTES).add(index));
            }
            cur = cur.sub(USIZE_BYTES);
        }
        debug_assert!(cur >= start && start.add(USIZE_BYTES) >= cur);
        cur = start;
        let chunk = cur.cast::<usize>().read_unaligned();
        if let Some(index) = self.rindex_of_needle(chunk) {
            return Some(cur.add(index));
        }
        None
    }

    /// Returns an iterator over all occurrences of one of the needle bytes in
    /// the given haystack.
    ///
    /// The iterator returned implements `DoubleEndedIterator`. This means it
    /// can also be used to find occurrences in reverse order.
    pub fn iter<'a, 'h>(&'a self, haystack: &'h [u8]) -> ThreeIter<'a, 'h> {
        ThreeIter { searcher: self, it: generic::Iter::new(haystack) }
    }

    #[inline(always)]
    fn index_of_needle(&self, chunk: usize) -> Option<usize> {
        #[inline(always)]
        fn min_index(a: Option<usize>, b: Option<usize>) -> Option<usize> {
            match (a, b) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            }
        }

        min_index(
            min_index(
                find_zero_in_chunk(self.v1 ^ chunk),
                find_zero_in_chunk(self.v2 ^ chunk),
            ),
            find_zero_in_chunk(self.v3 ^ chunk),
        )
    }

    #[inline(always)]
    fn rindex_of_needle(&self, chunk: usize) -> Option<usize> {
        #[inline(always)]
        fn max_index(a: Option<usize>, b: Option<usize>) -> Option<usize> {
            match (a, b) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            }
        }

        max_index(
            max_index(
                rfind_zero_in_chunk(self.v1 ^ chunk),
                rfind_zero_in_chunk(self.v2 ^ chunk),
            ),
            rfind_zero_in_chunk(self.v3 ^ chunk),
        )
    }

    #[inline(always)]
    fn confirm(&self, haystack_byte: u8) -> bool {
        self.s1 == haystack_byte
            || self.s2 == haystack_byte
            || self.s3 == haystack_byte
    }
}

/// An iterator over all occurrences of three possible bytes in a haystack.
///
/// This iterator implements `DoubleEndedIterator`, which means it can also be
/// used to find occurrences in reverse order.
///
/// This iterator is created by the [`Three::iter`] method.
///
/// The lifetime parameters are as follows:
///
/// * `'a` refers to the lifetime of the underlying [`Three`] searcher.
/// * `'h` refers to the lifetime of the haystack being searched.
#[derive(Clone, Debug)]
pub struct ThreeIter<'a, 'h> {
    /// The underlying memchr searcher.
    searcher: &'a Three,
    /// Generic iterator implementation.
    it: generic::Iter<'h>,
}

impl<'a, 'h> Iterator for ThreeIter<'a, 'h> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        // SAFETY: We rely on the generic iterator to provide valid start
        // and end pointers, but we guarantee that any pointer returned by
        // 'find_raw' falls within the bounds of the start and end pointer.
        unsafe { self.it.next(|s, e| self.searcher.find_raw(s, e)) }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

impl<'a, 'h> DoubleEndedIterator for ThreeIter<'a, 'h> {
    #[inline]
    fn next_back(&mut self) -> Option<usize> {
        // SAFETY: We rely on the generic iterator to provide valid start
        // and end pointers, but we guarantee that any pointer returned by
        // 'rfind_raw' falls within the bounds of the start and end pointer.
        unsafe { self.it.next_back(|s, e| self.searcher.rfind_raw(s, e)) }
    }
}

/// Return the index of the least significant zero byte in `x`.
///
/// That is, this routine treats `x` as a register of 8-bit lanes and returns
/// the index of the least significant lane that is `0`.
///
/// Based on "Matters Computational" by J. Arndt.
#[inline(always)]
fn lowest_zero_byte(x: usize) -> Option<usize> {
    // "The idea is to subtract one from each of the bytes and then look for
    // bytes where the borrow propagated all the way to the most significant
    // bit."
    const LO: usize = splat(0x01);
    const HI: usize = splat(0x80);

    let y = x.wrapping_sub(LO) & !x & HI;
    if y == 0 {
        None
    } else {
        Some(y.trailing_zeros() as usize / 8)
    }
}

/// Return the index of the most significant zero byte in `x`.
///
/// That is, this routine treats `x` as a register of 8-bit lanes and returns
/// the index of the most significant lane that is `0`.
///
/// Based on "Hacker's Delight" by Henry S. Warren.
#[inline(always)]
fn highest_zero_byte(x: usize) -> Option<usize> {
    const SEVEN_F: usize = splat(0x7F);

    let y = (x & SEVEN_F).wrapping_add(SEVEN_F);
    let y = !(y | x | SEVEN_F);
    (USIZE_BYTES - 1).checked_sub(y.leading_zeros() as usize / 8)
}

#[inline(always)]
fn find_zero_in_chunk(x: usize) -> Option<usize> {
    if cfg!(target_endian = "little") {
        lowest_zero_byte(x)
    } else {
        Some(USIZE_BYTES - 1 - highest_zero_byte(x)?)
    }
}

#[inline(always)]
fn rfind_zero_in_chunk(x: usize) -> Option<usize> {
    if cfg!(target_endian = "little") {
        highest_zero_byte(x)
    } else {
        Some(USIZE_BYTES - 1 - lowest_zero_byte(x)?)
    }
}

/// Repeat the given byte into a word size number. That is, every 8 bits
/// is equivalent to the given byte. For example, if `b` is `\x4E` or
/// `01001110` in binary, then the returned value on a 32-bit system would be:
/// `01001110_01001110_01001110_01001110`.
#[inline(always)]
const fn splat(b: u8) -> usize {
    // TODO: use `usize::from` once it can be used in const context.
    (b as usize) * (usize::MAX / 255)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cfg;

    define_memchr_quickcheck!(super, try_new);

    #[test]
    fn forward_one() {
        crate::tests::memchr::Runner::new(1).forward_iter(
            |haystack, needles| {
                Some(One::new(needles[0]).iter(haystack).collect())
            },
        )
    }

    #[test]
    fn reverse_one() {
        crate::tests::memchr::Runner::new(1).reverse_iter(
            |haystack, needles| {
                Some(One::new(needles[0]).iter(haystack).rev().collect())
            },
        )
    }

    #[test]
    fn count_one() {
        crate::tests::memchr::Runner::new(1).count_iter(|haystack, needles| {
            Some(One::new(needles[0]).iter(haystack).count())
        })
    }

    #[test]
    fn forward_two() {
        crate::tests::memchr::Runner::new(2).forward_iter(
            |haystack, needles| {
                let n1 = needles.get(0).copied()?;
                let n2 = needles.get(1).copied()?;
                Some(Two::new(n1, n2).iter(haystack).collect())
            },
        )
    }

    #[test]
    fn reverse_two() {
        crate::tests::memchr::Runner::new(2).reverse_iter(
            |haystack, needles| {
                let n1 = needles.get(0).copied()?;
                let n2 = needles.get(1).copied()?;
                Some(Two::new(n1, n2).iter(haystack).rev().collect())
            },
        )
    }

    #[test]
    fn forward_three() {
        crate::tests::memchr::Runner::new(3).forward_iter(
            |haystack, needles| {
                let n1 = needles.get(0).copied()?;
                let n2 = needles.get(1).copied()?;
                let n3 = needles.get(2).copied()?;
                Some(Three::new(n1, n2, n3).iter(haystack).collect())
            },
        )
    }

    #[test]
    fn reverse_three() {
        crate::tests::memchr::Runner::new(3).reverse_iter(
            |haystack, needles| {
                let n1 = needles.get(0).copied()?;
                let n2 = needles.get(1).copied()?;
                let n3 = needles.get(2).copied()?;
                Some(Three::new(n1, n2, n3).iter(haystack).rev().collect())
            },
        )
    }

    // This was found by quickcheck in the course of refactoring this crate
    // after memchr 2.5.0.
    #[test]
    fn regression_double_ended_iterator() {
        let finder = One::new(b'a');
        let haystack = "a";
        let mut it = finder.iter(haystack.as_bytes());
        assert_eq!(Some(0), it.next());
        assert_eq!(None, it.next_back());
    }

    // This regression test was caught by ripgrep's test suite on i686 when
    // upgrading to memchr 2.6. Namely, something about the \x0B bytes here
    // screws with the SWAR counting approach I was using. This regression test
    // prompted me to remove the SWAR counting approach and just replace it
    // with a byte-at-a-time loop.
    #[test]
    fn regression_count_new_lines() {
        let haystack = "01234567\x0b\n\x0b\n\x0b\n\x0b\nx";
        let count = One::new(b'\n').count(haystack.as_bytes());
        assert_eq!(4, count);
    }

    // A test[1] that failed on some big endian targets after a perf
    // improvement was merged[2].
    //
    // At first it seemed like the test suite somehow missed the regression,
    // but in actuality, CI was not running tests with `cross` but instead with
    // `cargo` specifically. This is because those steps were using `cargo`
    // instead of `${{ env.CARGO }}`. So adding this regression test doesn't
    // really help catch that class of failure, but we add it anyway for good
    // measure.
    //
    // [1]: https://github.com/BurntSushi/memchr/issues/152
    // [2]: https://github.com/BurntSushi/memchr/pull/151
    #[test]
    fn regression_big_endian1() {
        assert_eq!(One::new(b':').find(b"1:23"), Some(1));
    }

    // Interestingly, I couldn't get `regression_big_endian1` to fail for me
    // on the `powerpc64-unknown-linux-gnu` target. But I found another case
    // through quickcheck that does.
    #[test]
    fn regression_big_endian2() {
        let data = [0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(One::new(b'\x00').find(&data), Some(0));
    }

    /// Generate 500K values.
    fn special_values() -> impl Iterator<Item = usize> {
        fn all_bytes() -> impl Iterator<Item = u8> {
            0..=0xff
        }

        fn some_bytes() -> impl Iterator<Item = u8> {
            [0x00, 0x01, 0x02, 0x10, 0x11, 0x8f, 0xff].into_iter()
        }

        all_bytes().flat_map(move |first_byte| {
            some_bytes().flat_map(move |middle_byte| {
                all_bytes().map(move |last_byte| {
                    splat(middle_byte) & !0xff & !(0xff << (usize::BITS - 8))
                        | ((first_byte as usize) << (usize::BITS - 8))
                        | (last_byte as usize)
                })
            })
        })
    }

    fn lowest_zero_byte_simple(value: usize) -> Option<usize> {
        value.to_le_bytes().iter().position(|&b| b == 0)
    }

    fn highest_zero_byte_simple(value: usize) -> Option<usize> {
        value.to_le_bytes().iter().rposition(|&b| b == 0)
    }

    #[test]
    fn test_lowest_zero_byte() {
        assert_eq!(Some(0), lowest_zero_byte(0x00000000));
        assert_eq!(Some(0), lowest_zero_byte(0x01000000));
        assert_eq!(Some(1), lowest_zero_byte(0x00000001));
        assert_eq!(Some(1), lowest_zero_byte(0x00000010));
        assert_eq!(Some(1), lowest_zero_byte(0x00220010));
        assert_eq!(Some(1), lowest_zero_byte(0xff220010));
        assert_eq!(Some(USIZE_BYTES - 1), lowest_zero_byte(usize::MAX >> 8));
        assert_eq!(Some(USIZE_BYTES - 1), lowest_zero_byte(usize::MAX >> 9));
        assert_eq!(Some(USIZE_BYTES - 2), lowest_zero_byte(usize::MAX >> 16));
        assert_eq!(None, lowest_zero_byte(usize::MAX >> 7));
        assert_eq!(None, lowest_zero_byte(usize::MAX));
    }

    #[test]
    fn test_highest_zero_byte() {
        assert_eq!(Some(USIZE_BYTES - 1), highest_zero_byte(0x00000000));
        assert_eq!(Some(USIZE_BYTES - 1), highest_zero_byte(0x00345678));
        assert_eq!(Some(USIZE_BYTES - 1), highest_zero_byte(usize::MAX >> 8));
        assert_eq!(Some(USIZE_BYTES - 1), highest_zero_byte(usize::MAX >> 9));
        assert_eq!(Some(USIZE_BYTES - 1), highest_zero_byte(usize::MAX >> 9));
        assert_eq!(
            Some(USIZE_BYTES - 1),
            highest_zero_byte((usize::MAX >> 9) & !0xff)
        );
        assert_eq!(None, highest_zero_byte(usize::MAX >> 3));
    }

    #[test]
    fn test_lowest_zero_bytes_special_values() {
        if cfg!(miri) {
            return;
        }

        for value in special_values() {
            assert_eq!(
                lowest_zero_byte_simple(value),
                lowest_zero_byte(value)
            );
        }
    }

    #[test]
    fn test_highest_zero_bytes_special_values() {
        if cfg!(miri) {
            return;
        }

        for value in special_values() {
            assert_eq!(
                highest_zero_byte_simple(value),
                highest_zero_byte(value)
            );
        }
    }
}
