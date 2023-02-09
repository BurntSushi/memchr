mod default;

/// This trait allows the user to customize the heuristic used to determine the
/// relative frequency of a given byte in the dataset being searched.
///
/// The use of this trait can have a dramatic impact on performance depending
/// on the type of data being searched. The details of why are explained in the
/// docs of [`prefilter::Prefilter`]. To summarize, the core algorithm uses a
/// prefilter to quickly identify candidate matches that are later verified
/// more slowly. This prefilter is implemented in terms of trying to find
/// `rare` bytes at specific offsets that will occur less frequently in the
/// dataset. While the concept of a `rare` byte is similar for most datasets,
/// there are some specific datasets (like binary executables) that have
/// dramatically different byte distributions. For these datasets customizing
/// the byte frequency heuristic can have a massive impact on performance, and
/// might even need to be done at runtime.
///
/// The default implementation of `HeuristicFrequencyRank` reads from the
/// static frequency table defined in `src/memmem/byte_frequencies.rs`. This
/// is optimal for most inputs, so if you are unsure of the impact of using a
/// custom `HeuristicFrequencyRank` you should probably just use the default.
///
/// # Example
///
/// ```
/// use memchr::memmem::{FinderBuilder, HeuristicFrequencyRank};
///
/// /// A byte-frequency table that is good for scanning binary executables.
/// struct Binary;
///
/// impl HeuristicFrequencyRank for Binary {
///     fn rank(&self, byte: u8) -> u8 {
///         const TABLE: [u8; 256] = [
///             255, 128, 61, 43, 50, 41, 27, 28, 57, 15, 21, 13, 24, 17, 17,
///             89, 58, 16, 11, 7, 14, 23, 7, 6, 24, 9, 6, 5, 9, 4, 7, 16,
///             68, 11, 9, 6, 88, 7, 4, 4, 23, 9, 4, 8, 8, 5, 10, 4, 30, 11,
///             9, 24, 11, 5, 5, 5, 19, 11, 6, 17, 9, 9, 6, 8,
///             48, 58, 11, 14, 53, 40, 9, 9, 254, 35, 3, 6, 52, 23, 6, 6, 27,
///             4, 7, 11, 14, 13, 10, 11, 11, 5, 2, 10, 16, 12, 6, 19,
///             19, 20, 5, 14, 16, 31, 19, 7, 14, 20, 4, 4, 19, 8, 18, 20, 24,
///             1, 25, 19, 58, 29, 10, 5, 15, 20, 2, 2, 9, 4, 3, 5,
///             51, 11, 4, 53, 23, 39, 6, 4, 13, 81, 4, 186, 5, 67, 3, 2, 15,
///             0, 0, 1, 3, 2, 0, 0, 5, 0, 0, 0, 2, 0, 0, 0,
///             12, 2, 1, 1, 3, 1, 1, 1, 6, 1, 2, 1, 3, 1, 1, 2, 9, 1, 1, 0,
///             2, 2, 4, 4, 11, 6, 7, 3, 6, 9, 4, 5,
///             46, 18, 8, 18, 17, 3, 8, 20, 16, 10, 3, 7, 175, 4, 6, 7, 13,
///             3, 7, 3, 3, 1, 3, 3, 10, 3, 1, 5, 2, 0, 1, 2,
///             16, 3, 5, 1, 6, 1, 1, 2, 58, 20, 3, 14, 12, 2, 1, 3, 16, 3, 5,
///             8, 3, 1, 8, 6, 17, 6, 5, 3, 8, 6, 13, 175,
///         ];
///         TABLE[byte as usize]
///     }
/// }
/// // Create a new finder with the custom heuristic.
/// let finder = FinderBuilder::new()
///     .build_forward_with_ranker(Binary, b"\x00\x00\xdd\xdd");
/// // Find needle with custom heuristic.
/// assert!(finder.find(b"\x00\x00\x00\xdd\xdd").is_some());
/// ```
pub trait HeuristicFrequencyRank {
    /// Return the heuristic frequency rank of the given byte. A lower rank
    /// means the byte is believed to occur less frequently in the haystack.
    ///
    /// Some uses of this heuristic may treat arbitrary absolute rank values as
    /// significant. For example, an implementation detail in this crate may
    /// determine that heuristic prefilters are inappropriate if every byte in
    /// the needle has a "high" rank.
    fn rank(&self, byte: u8) -> u8;
}

/// The default byte frequency heuristic that is good for most haystacks.
pub(crate) struct DefaultFrequencyRank;

impl HeuristicFrequencyRank for DefaultFrequencyRank {
    fn rank(&self, byte: u8) -> u8 {
        self::default::RANK[usize::from(byte)]
    }
}

/// This permits passing any implementation of `HeuristicFrequencyRank` as a
/// borrowed version of itself.
impl<'a, R> HeuristicFrequencyRank for &'a R
where
    R: HeuristicFrequencyRank,
{
    fn rank(&self, byte: u8) -> u8 {
        (**self).rank(byte)
    }
}
