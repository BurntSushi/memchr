use std::{
    io::Read,
    time::{Duration, Instant},
};

use {
    anyhow::Context,
    bstr::{ByteSlice, ByteVec},
};

/// A single benchmark configuration read from a sequence of KLV items on
/// stdin.
#[derive(Clone, Debug, Default)]
pub struct Benchmark {
    pub name: String,
    pub model: String,
    pub needles: Vec<Vec<u8>>,
    pub haystack: Vec<u8>,
    pub max_iters: u64,
    pub max_warmup_iters: u64,
    pub max_time: Duration,
    pub max_warmup_time: Duration,
}

impl Benchmark {
    /// Read the KLV benchmark configuration from stdin.
    pub fn from_stdin() -> anyhow::Result<Benchmark> {
        let mut raw = vec![];
        std::io::stdin().read_to_end(&mut raw)?;
        Benchmark::read(&raw)
    }

    /// Return a single needle from this benchmark. If the benchmark has no
    /// needles or more than one, then an error is returned.
    pub fn one_needle(&self) -> anyhow::Result<&[u8]> {
        anyhow::ensure!(
            self.needles.len() == 1,
            "benchmark only supports one needle, but {} were found",
            self.needles.len(),
        );
        Ok(&self.needles[0])
    }

    /// Return a single needle byte from this benchmark. If the benchmark has
    /// no needles or more than one, then an error is returned. An error is
    /// also returned if the single needle a length not equal to 1.
    pub fn one_needle_byte(&self) -> anyhow::Result<u8> {
        anyhow::ensure!(
            self.needles.len() == 1,
            "benchmark only supports one needle, but {} were found",
            self.needles.len(),
        );
        anyhow::ensure!(
            self.needles[0].len() == 1,
            "needle must have length 1 (in bytes) but it has length {}",
            self.needles[0].len(),
        );
        Ok(self.needles[0][0])
    }

    /// Return a single needle byte from this benchmark. If the benchmark has
    /// a number of needles not equal to 2, or if any of the needles have a
    /// length not equal to 1, then an error is returned.
    pub fn two_needle_bytes(&self) -> anyhow::Result<(u8, u8)> {
        anyhow::ensure!(
            self.needles.len() == 2,
            "benchmark supports two needles, but {} were found",
            self.needles.len(),
        );
        anyhow::ensure!(
            self.needles[0].len() == 1,
            "first needle has length {} but expected 1",
            self.needles[0].len(),
        );
        anyhow::ensure!(
            self.needles[1].len() == 1,
            "second needle has length {} but expected 1",
            self.needles[1].len(),
        );
        Ok((self.needles[0][0], self.needles[1][0]))
    }

    /// Return a single needle byte from this benchmark. If the benchmark has
    /// a number of needles not equal to 2, or if any of the needles have a
    /// length not equal to 1, then an error is returned.
    pub fn three_needle_bytes(&self) -> anyhow::Result<(u8, u8, u8)> {
        anyhow::ensure!(
            self.needles.len() == 3,
            "benchmark supports three needles, but {} were found",
            self.needles.len(),
        );
        anyhow::ensure!(
            self.needles[0].len() == 1,
            "first needle has length {} but expected 1",
            self.needles[0].len(),
        );
        anyhow::ensure!(
            self.needles[1].len() == 1,
            "second needle has length {} but expected 1",
            self.needles[1].len(),
        );
        anyhow::ensure!(
            self.needles[2].len() == 1,
            "second needle has length {} but expected 1",
            self.needles[2].len(),
        );
        Ok((self.needles[0][0], self.needles[1][0], self.needles[2][0]))
    }

    fn read(mut raw: &[u8]) -> anyhow::Result<Benchmark> {
        let mut config = Benchmark::default();
        while !raw.is_empty() {
            let (klv, nread) = OneKLV::read(raw)?;
            raw = &raw[nread..];
            config.set(klv)?;
        }
        Ok(config)
    }

    fn set(&mut self, klv: OneKLV) -> anyhow::Result<()> {
        let parse_duration = |v: &str| -> anyhow::Result<Duration> {
            Ok(Duration::from_nanos(v.parse()?))
        };
        let OneKLV { key, value } = klv;
        match &*key {
            "name" => self.name = value.to_str()?.to_string(),
            "model" => self.model = value.to_str()?.to_string(),
            "pattern" => {
                self.needles.push(Vec::unescape_bytes(value.to_str()?))
            }
            "haystack" => self.haystack = value.to_vec(),
            "max-iters" => self.max_iters = value.to_str()?.parse()?,
            "max-warmup-iters" => {
                self.max_warmup_iters = value.to_str()?.parse()?
            }
            "max-time" => self.max_time = parse_duration(value.to_str()?)?,
            "max-warmup-time" => {
                self.max_warmup_time = parse_duration(value.to_str()?)?
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct OneKLV {
    key: String,
    value: Vec<u8>,
}

impl OneKLV {
    fn read(bytes: &[u8]) -> anyhow::Result<(OneKLV, usize)> {
        let mut nread = 0;
        let (key, bytes) = match bytes.split_once_str(":") {
            Some(x) => x,
            None => anyhow::bail!(
                "failed to find first ':' in key-length-value item \
                 where the next (at most) 80 bytes are: {:?}",
                bytes[..std::cmp::min(80, bytes.len())].as_bstr(),
            ),
        };
        nread += key.len() + 1; // +1 for ':'
        let key = key
            .to_str()
            .with_context(|| {
                format!("key {:?} is not valid UTF-8", key.as_bstr())
            })?
            .to_string();

        let (len, bytes) = match bytes.split_once_str(":") {
            Some(x) => x,
            None => anyhow::bail!(
                "failed to find second ':' in key-length-value item \
                 for key '{}'",
                key,
            ),
        };
        nread += len.len() + 1; // +1 for ':'
        let len = len.to_str().with_context(|| {
            format!("length for key '{}' is not valid UTF-8", key)
        })?;
        let len = len.parse::<usize>().with_context(|| {
            format!(
                "length '{}' for key '{}' is not a valid integer",
                len, key,
            )
        })?;

        anyhow::ensure!(
            bytes.len() >= len,
            "got length of {} for key '{}', but only {} bytes remain",
            len,
            key,
            bytes.len(),
        );
        let value = bytes[..len].into();
        let bytes = &bytes[len..];
        nread += len;

        anyhow::ensure!(
            bytes.len() >= 1,
            "expected trailing '\\n' after value, but got EOF",
        );
        anyhow::ensure!(
            bytes[0] == b'\n',
            "expected '\\n' after value, but got {:?}",
            bytes[0..1].as_bstr(),
        );
        nread += 1;

        let klv = OneKLV { key, value };
        Ok((klv, nread))
    }
}

/// A sample computed from a single benchmark iteration.
#[derive(Clone, Debug)]
pub struct Sample {
    /// The duration of the iteration.
    pub duration: Duration,
    /// The count reported by the benchmark. This is used by the harness to
    /// verify that the result is correct.
    ///
    /// All benchmark models except for regex-redux use this. For regex-redux,
    /// it is always zero.
    pub count: u64,
}

/// Run the given `bench` function repeatedly until either the maximum
/// time or number of iterations has been reached and return the set of
/// samples.
pub fn run(
    b: &Benchmark,
    bench: impl FnMut() -> anyhow::Result<usize>,
) -> anyhow::Result<Vec<Sample>> {
    run_and_count(b, |count| Ok(count), bench)
}

/// Run the given `bench` function repeatedly until either the maximum
/// time or number of iterations has been reached and return the set of
/// samples. The count for each sample is determined by running `count` on
/// the result of `bench`. The execution time of `count` is specifically
/// not included in the sample's duration.
///
/// N.B. This variant only exists for the 'compile' model. We want to only
/// measure compile time, but still do extra work that we specifically
/// don't measure to produce a count to ensure the compile regex behaves as
/// expected.
pub fn run_and_count<T>(
    b: &Benchmark,
    mut count: impl FnMut(T) -> anyhow::Result<usize>,
    mut bench: impl FnMut() -> anyhow::Result<T>,
) -> anyhow::Result<Vec<Sample>> {
    let warmup_start = Instant::now();
    for _ in 0..b.max_warmup_iters {
        let result = bench();
        // We still compute the count in case there was a problem doing so,
        // even though we don't do anything with the count.
        let _count = count(result?)?;
        if warmup_start.elapsed() >= b.max_warmup_time {
            break;
        }
    }

    let mut samples = vec![];
    let run_start = Instant::now();
    for _ in 0..b.max_iters {
        let bench_start = Instant::now();
        let result = bench();
        let duration = bench_start.elapsed();
        // Should be fine since it's unreasonable for a match count to
        // exceed u64::MAX.
        let count = u64::try_from(count(result?)?).unwrap();
        samples.push(Sample { duration, count });
        if run_start.elapsed() >= b.max_time {
            break;
        }
    }
    Ok(samples)
}

pub fn count_memchr(
    mut haystack: &[u8],
    needle: u8,
    memchr: impl Fn(&[u8], u8) -> Option<usize>,
) -> usize {
    let mut count = 0;
    while let Some(i) = memchr(haystack, needle) {
        count += 1;
        haystack = &haystack[i + 1..];
    }
    count
}

pub fn count_memchr2(
    mut haystack: &[u8],
    needle1: u8,
    needle2: u8,
    memchr2: impl Fn(&[u8], u8, u8) -> Option<usize>,
) -> usize {
    let mut count = 0;
    while let Some(i) = memchr2(haystack, needle1, needle2) {
        count += 1;
        haystack = &haystack[i + 1..];
    }
    count
}

pub fn count_memchr3(
    mut haystack: &[u8],
    needle1: u8,
    needle2: u8,
    needle3: u8,
    memchr3: impl Fn(&[u8], u8, u8, u8) -> Option<usize>,
) -> usize {
    let mut count = 0;
    while let Some(i) = memchr3(haystack, needle1, needle2, needle3) {
        count += 1;
        haystack = &haystack[i + 1..];
    }
    count
}

pub fn count_memmem(
    mut haystack: &[u8],
    needle: &[u8],
    memmem: impl Fn(&[u8], &[u8]) -> Option<usize>,
) -> usize {
    let mut count = 0;
    while let Some(i) = memmem(haystack, needle) {
        count += 1;
        haystack = match haystack.get(i + needle.len()..) {
            Some(haystack) => haystack,
            None => break,
        };
    }
    count
}

pub fn count_memmem_str(
    mut haystack: &str,
    needle: &str,
    memmem: impl Fn(&str, &str) -> Option<usize>,
) -> usize {
    let mut count = 0;
    while let Some(i) = memmem(haystack, needle) {
        count += 1;
        haystack = match haystack.get(i + needle.len()..) {
            Some(haystack) => haystack,
            None => break,
        };
    }
    count
}
