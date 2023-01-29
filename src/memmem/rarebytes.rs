/// A heuristic frequency based detection of rare bytes for substring search.
///
/// This detector attempts to pick out two bytes in a needle that are predicted
/// to occur least frequently. The purpose is to use these bytes to implement
/// fast candidate search using vectorized code.
///
/// A set of offsets is only computed for needles of length 2 or greater.
/// Smaller needles should be special cased by the substring search algorithm
/// in use. (e.g., Use memchr for single byte needles.)
///
/// Note that we use `u8` to represent the offsets of the rare bytes in a
/// needle to reduce space usage. This means that rare byte occurring after the
/// first 255 bytes in a needle will never be used.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RareNeedleBytes {
    /// The leftmost offset of the rarest byte in the needle, according to
    /// pre-computed frequency analysis. The "leftmost offset" means that
    /// rare1i <= i for all i where needle[i] == needle[rare1i].
    rare1i: u8,
    /// The leftmost offset of the second rarest byte in the needle, according
    /// to pre-computed frequency analysis. The "leftmost offset" means that
    /// rare2i <= i for all i where needle[i] == needle[rare2i].
    ///
    /// The second rarest byte is used as a type of guard for quickly detecting
    /// a mismatch if the first byte matches. This is a hedge against
    /// pathological cases where the pre-computed frequency analysis may be
    /// off. (But of course, does not prevent *all* pathological cases.)
    ///
    /// In general, rare1i != rare2i by construction, although there is no hard
    /// requirement that they be different. However, since the case of a single
    /// byte needle is handled specially by memchr itself, rare2i generally
    /// always should be different from rare1i since it would otherwise be
    /// ineffective as a guard.
    rare2i: u8,
}

impl RareNeedleBytes {
    /// Create a new pair of rare needle bytes with the given offsets. This is
    /// only used in tests for generating input data.
    #[cfg(all(test, feature = "std"))]
    pub(crate) fn new(rare1i: u8, rare2i: u8) -> RareNeedleBytes {
        RareNeedleBytes { rare1i, rare2i }
    }

    /// Detect the leftmost offsets of the two rarest bytes in the given
    /// needle.
    pub(crate) fn forward(needle: &[u8]) -> RareNeedleBytes {
        if needle.len() <= 1 || needle.len() > core::u8::MAX as usize {
            // For needles bigger than u8::MAX, our offsets aren't big enough.
            // (We make our offsets small to reduce stack copying.)
            // If you have a use case for it, please file an issue. In that
            // case, we should probably just adjust the routine below to pick
            // some rare bytes from the first 255 bytes of the needle.
            //
            // Also note that for needles of size 0 or 1, they are special
            // cased in Two-Way.
            //
            // TODO: Benchmar this.
            return RareNeedleBytes { rare1i: 0, rare2i: 0 };
        }

        // Find the rarest two bytes. We make them distinct by construction.
        let freq = byte_frequencies();
        let (mut rare1, mut rare1i) = (needle[0], 0);
        let (mut rare2, mut rare2i) = (needle[1], 1);
        if rank(freq, rare2) < rank(freq, rare1) {
            core::mem::swap(&mut rare1, &mut rare2);
            core::mem::swap(&mut rare1i, &mut rare2i);
        }
        for (i, &b) in needle.iter().enumerate().skip(2) {
            if rank(freq, b) < rank(freq, rare1) {
                rare2 = rare1;
                rare2i = rare1i;
                rare1 = b;
                rare1i = i as u8;
            } else if b != rare1 && rank(freq, b) < rank(freq, rare2) {
                rare2 = b;
                rare2i = i as u8;
            }
        }
        // While not strictly required, we really don't want these to be
        // equivalent. If they were, it would reduce the effectiveness of
        // candidate searching using these rare bytes by increasing the rate of
        // false positives.
        assert_ne!(rare1i, rare2i);
        RareNeedleBytes { rare1i, rare2i }
    }

    /// Return the rare bytes in the given needle in the forward direction.
    /// The needle given must be the same one given to the RareNeedleBytes
    /// constructor.
    pub(crate) fn as_rare_bytes(&self, needle: &[u8]) -> (u8, u8) {
        (needle[self.rare1i as usize], needle[self.rare2i as usize])
    }

    /// Return the rare offsets such that the first offset is always <= to the
    /// second offset. This is useful when the caller doesn't care whether
    /// rare1 is rarer than rare2, but just wants to ensure that they are
    /// ordered with respect to one another.
    #[cfg(memchr_runtime_simd)]
    pub(crate) fn as_rare_ordered_usize(&self) -> (usize, usize) {
        let (rare1i, rare2i) = self.as_rare_ordered_u8();
        (rare1i as usize, rare2i as usize)
    }

    /// Like as_rare_ordered_usize, but returns the offsets as their native
    /// u8 values.
    #[cfg(memchr_runtime_simd)]
    pub(crate) fn as_rare_ordered_u8(&self) -> (u8, u8) {
        if self.rare1i <= self.rare2i {
            (self.rare1i, self.rare2i)
        } else {
            (self.rare2i, self.rare1i)
        }
    }

    /// Return the rare offsets as usize values in the order in which they were
    /// constructed. rare1, for example, is constructed as the "rarer" byte,
    /// and thus, callers may want to treat it differently from rare2.
    pub(crate) fn as_rare_usize(&self) -> (usize, usize) {
        (self.rare1i as usize, self.rare2i as usize)
    }

    /// Return the byte frequency rank of each byte. The higher the rank, the
    /// more frequency the byte is predicted to be. The needle given must be
    /// the same one given to the RareNeedleBytes constructor.
    pub(crate) fn as_ranks(&self, needle: &[u8]) -> (usize, usize) {
        let freq = byte_frequencies();
        let (b1, b2) = self.as_rare_bytes(needle);
        (rank(freq, b1), rank(freq, b2))
    }
}

/// Return the heuristical frequency rank of the given byte. A lower rank
/// means the byte is believed to occur less frequently.
fn rank(freq: ByteFrequencies, b: u8) -> usize {
    freq[b as usize] as usize
}


use std::sync::atomic::{AtomicUsize, Ordering};

type ByteTable = [u8; 256];
type ByteFrequencies = &'static ByteTable;

/// Set the byte frequency table used to construct `RareNeedleBytes`.
/// 
/// This setting can have a dramatic impact on performance depending on
/// the type of data being searched. The frequency table defined in 
/// `crate::memmem::byte_frequencies::BYTE_FREQUENCIES` is very good for most
/// common data types, but can be sub-optimal when scanning specific kinds of data,
/// such as binary executables. In a binary executable, the `\x00` byte is the most
/// common byte by an order of magnitude, whereas in the default table it is not very frequent.
/// 
/// This is a `global` setting in order to minimize the impact on the rest
/// of the API. Using a custom byte frequency table is a very nice use case
/// and should not impact the speed or usability of the rest of the API.
/// Also, the most common case for using a byte frequency table is where
/// ALL searches are performed with the custom table, so we can get away
/// with not implementing a per-search selection of frequency tables.
/// 
/// Example
/// ```
/// use memchr::memmem;
/// 
/// // The default table (good for most inputs)
/// memmem::set_byte_frequencies(None);
/// 
/// // A table that is good for searching binary executables
/// memmem::set_byte_frequencies(Some(&[
///     255, 128, 61, 43, 50, 41, 27, 28, 57, 15, 21, 13, 24, 17, 17, 89, 58, 16, 11, 7, 14, 23, 7, 6, 24, 9, 6, 5, 9, 4, 7, 16,
///     68, 11, 9, 6, 88, 7, 4, 4, 23, 9, 4, 8, 8, 5, 10, 4, 30, 11, 9, 24, 11, 5, 5, 5, 19, 11, 6, 17, 9, 9, 6, 8,
///     48, 58, 11, 14, 53, 40, 9, 9, 254, 35, 3, 6, 52, 23, 6, 6, 27, 4, 7, 11, 14, 13, 10, 11, 11, 5, 2, 10, 16, 12, 6, 19,
///     19, 20, 5, 14, 16, 31, 19, 7, 14, 20, 4, 4, 19, 8, 18, 20, 24, 1, 25, 19, 58, 29, 10, 5, 15, 20, 2, 2, 9, 4, 3, 5,
///     51, 11, 4, 53, 23, 39, 6, 4, 13, 81, 4, 186, 5, 67, 3, 2, 15, 0, 0, 1, 3, 2, 0, 0, 5, 0, 0, 0, 2, 0, 0, 0,
///     12, 2, 1, 1, 3, 1, 1, 1, 6, 1, 2, 1, 3, 1, 1, 2, 9, 1, 1, 0, 2, 2, 4, 4, 11, 6, 7, 3, 6, 9, 4, 5,
///     46, 18, 8, 18, 17, 3, 8, 20, 16, 10, 3, 7, 175, 4, 6, 7, 13, 3, 7, 3, 3, 1, 3, 3, 10, 3, 1, 5, 2, 0, 1, 2,
///     16, 3, 5, 1, 6, 1, 1, 2, 58, 20, 3, 14, 12, 2, 1, 3, 16, 3, 5, 8, 3, 1, 8, 6, 17, 6, 5, 3, 8, 6, 13, 175,
/// ]));
/// 
/// let finder = memmem::Finder::new("foo");
/// 
/// assert_eq!(Some(4), finder.find(b"baz foo quux"));
/// assert_eq!(None, finder.find(b"quux baz bar"));
/// ```
/// 
pub fn set_byte_frequencies(frequencies: Option<ByteFrequencies>) {
    let new_ptr = match frequencies {
        None => 0,
        Some(f) => f as *const ByteTable as usize
    };
    // SAFETY: `ByteFrequencies` is a static const reference, so the
    // pointer must be valid for the entire program lifetime and can
    // therefore be stored safely and dereferenced later at any time.
    // Also, we only ever read the memory pointed at by `PTR_BYTE_FREQ`,
    // so calling `byte_frequencies()/set_byte_frequencies()` is thread safe,
    // provided that we atomically read/write the value of the pointer.
    PTR_BYTE_FREQ.store(new_ptr, Ordering::SeqCst);
}


/// Return the global byte frequency table used to determine the 
/// heuristical frequency rank of the given byte.
fn byte_frequencies() -> ByteFrequencies {
    let ptr = PTR_BYTE_FREQ.load(Ordering::SeqCst);
    if ptr == 0 {
        &crate::memmem::byte_frequencies::BYTE_FREQUENCIES
    } else {
        // SAFETY: If `PTR_BYTE_FREQ` != 0 then it was explicity set via
        // `set_byte_frequencies`, so we can safely dereference the pointer and 
        // transmute to a static reference (see `set_byte_frequencies` for more details).
        unsafe { 
            let deref: &ByteTable = &*(ptr as *const ByteTable);
            std::mem::transmute(deref)
        }
    }
}

/// Global storage for the current byte frequency table in use
static PTR_BYTE_FREQ: AtomicUsize = AtomicUsize::new(0);
