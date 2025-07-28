use std::io::Write;

use shared::{Benchmark, Sample};

fn main() -> anyhow::Result<()> {
    let mut args = vec![];
    for osarg in std::env::args_os().skip(1) {
        let Ok(arg) = osarg.into_string() else {
            anyhow::bail!("all arguments must be valid UTF-8")
        };
        args.push(arg);
    }
    anyhow::ensure!(
        !args.is_empty(),
        "Usage: runner [--quiet] (<engine-name> | --version)"
    );
    if args.iter().any(|a| a == "--version") {
        writeln!(std::io::stdout(), env!("CARGO_PKG_VERSION"))?;
        return Ok(());
    }
    let quiet = args.iter().any(|a| a == "--quiet");
    let engine = &**args.last().unwrap();
    let b = Benchmark::from_stdin()?;
    let samples = match (&*engine, &*b.model) {
        ("memchr-prebuilt", "count-bytes") => bufchr_prebuilt_count(&b)?,
        (engine, model) => {
            anyhow::bail!("unrecognized engine '{engine}' and model '{model}'")
        }
    };
    if !quiet {
        let mut stdout = std::io::stdout().lock();
        for s in samples.iter() {
            writeln!(stdout, "{},{}", s.duration.as_nanos(), s.count)?;
        }
    }
    Ok(())
}

fn bufchr_prebuilt_count(b: &Benchmark) -> anyhow::Result<Vec<Sample>> {
    let haystack = &b.haystack;
    let needle = b.one_needle_byte()?;
    shared::run(b, || Ok(bufchr_sse2_iter(needle, haystack)))
}

/// A trait for adding some helper routines to pointers.
pub(crate) trait Pointer {
    /// Returns the distance, in units of `T`, between `self` and `origin`.
    ///
    /// # Safety
    ///
    /// Same as `ptr::offset_from` in addition to `self >= origin`.
    unsafe fn distance(self, origin: Self) -> usize;

    /// Casts this pointer to `usize`.
    ///
    /// Callers should not convert the `usize` back to a pointer if at all
    /// possible. (And if you believe it's necessary, open an issue to discuss
    /// why. Otherwise, it has the potential to violate pointer provenance.)
    /// The purpose of this function is just to be able to do arithmetic, i.e.,
    /// computing offsets or alignments.
    fn as_usize(self) -> usize;
}

impl<T> Pointer for *const T {
    unsafe fn distance(self, origin: *const T) -> usize {
        // TODO: Replace with `ptr::sub_ptr` once stabilized.
        usize::try_from(self.offset_from(origin)).unwrap_unchecked()
    }

    fn as_usize(self) -> usize {
        self as usize
    }
}

use core::arch::x86_64::{
    __m128i, _mm_cmpeq_epi8, _mm_load_si128, _mm_loadu_si128,
    _mm_movemask_epi8, _mm_set1_epi8,
};

#[inline(always)]
fn get_for_offset(mask: u32) -> u32 {
    #[cfg(target_endian = "big")]
    {
        mask.swap_bytes()
    }
    #[cfg(target_endian = "little")]
    {
        mask
    }
}

#[inline(always)]
fn first_offset(mask: u32) -> usize {
    get_for_offset(mask).trailing_zeros() as usize
}

#[inline(always)]
fn clear_least_significant_bit(mask: u32) -> u32 {
    mask & (mask - 1)
}

struct OneMatches<'h> {
    start: *const u8,
    end: *const u8,
    current: *const u8,
    mask: Option<(*const u8, u32)>,
    needle: u8,
    splat: __m128i,
    haystack: core::marker::PhantomData<&'h [u8]>,
}

const BYTES: usize = 16;
const ALIGN: usize = 15;

impl<'h> OneMatches<'h> {
    unsafe fn new(needle: u8, haystack: &[u8]) -> Self {
        let ptr = haystack.as_ptr();

        Self {
            start: ptr,
            end: ptr.wrapping_add(haystack.len()),
            current: ptr,
            mask: None,
            needle,
            splat: _mm_set1_epi8(needle as i8),
            haystack: core::marker::PhantomData,
        }
    }

    unsafe fn next(&mut self) -> Option<usize> {
        if self.start >= self.end {
            return None;
        }

        'main: loop {
            // Processing current move mask
            if let Some((from, mask)) = &mut self.mask {
                debug_assert!(*mask != 0);

                let offset = from.add(first_offset(*mask));
                let next_mask = clear_least_significant_bit(*mask);

                if next_mask != 0 {
                    *mask = next_mask;
                } else {
                    self.mask = None;
                }

                return Some(offset.distance(self.start));
            }

            // NOTE: bytes from the start must be consumed linearly until
            // alignement is reached!
            // NOTE: must not happen if empty

            // Initial unaligned load
            // if self.current == self.start {
            //     let chunk = _mm_loadu_si128(self.current as *const __m128i);
            //     let cmp = _mm_cmpeq_epi8(chunk, self.splat);
            //     let mask = _mm_movemask_epi8(cmp) as u32;

            //     let next = self.start.add(BYTES - (self.start.as_usize() & ALIGN));

            //     if mask != 0 {
            //         self.mask = Some((self.start, mask));
            //         self.current = next;
            //         continue 'main;
            //     } else {
            //         self.current = next;
            //     }
            // }

            // Main loop of aligned loads
            while self.current <= self.end.sub(BYTES) {
                // debug_assert_eq!(0, self.current.as_usize() % BYTES);

                let chunk = _mm_loadu_si128(self.current as *const __m128i);
                let cmp = _mm_cmpeq_epi8(chunk, self.splat);
                let mask = _mm_movemask_epi8(cmp) as u32;

                let next = self.current.add(BYTES);

                if mask != 0 {
                    self.mask = Some((self.current, mask));
                    self.current = next;
                    continue 'main;
                } else {
                    self.current = next;
                }
            }

            // debug_assert!(self.end.distance(self.current) < BYTES);

            // Processing remaining bytes linearly
            while self.current < self.end {
                if *self.current == self.needle {
                    let offset = self.current.distance(self.start);
                    self.current = self.current.add(1);
                    return Some(offset);
                } else {
                    self.current = self.current.add(1);
                }
            }

            return None;
        }
    }
}

struct OneMatchesIter<'h>(OneMatches<'h>);

impl<'h> OneMatchesIter<'h> {
    fn new(needle: u8, haystack: &[u8]) -> Self {
        unsafe { OneMatchesIter(OneMatches::new(needle, haystack)) }
    }
}

impl<'h> Iterator for OneMatchesIter<'h> {
    type Item = usize;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.0.next() }
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
pub fn bufchr_sse2_iter(needle: u8, haystack: &[u8]) -> usize {
    OneMatchesIter::new(needle, haystack).count()
}
