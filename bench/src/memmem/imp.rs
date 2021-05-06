/*
This module defines a common API (by convention) for all of the different
impls that we benchmark. The intent here is to 1) make it easy to write macros
for generating benchmark definitions generic over impls and 2) make it easier
to read the benchmarks themselves and grok how exactly each of the impls are
being invoked.

The naming scheme of each function follows the pertinent parts of our benchmark
naming scheme (see parent module docs). Namely, it is

  {impl}/{fwd|rev}/{config}

Where 'impl' is the underlying implementation and 'config' is the manner of
search. The slash indicates a module boundary. We use modules for this because
it makes writing macros to define benchmarks for all variants much easier.
*/

/// memchr's implementation of memmem. This is the implementation that we hope
/// does approximately as well as all other implementations, and a lot better
/// in at least some cases.
pub(crate) mod krate {
    pub(crate) fn available(_: &str) -> &'static [&'static str] {
        &["reverse", "oneshot", "prebuilt", "oneshotiter", "prebuiltiter"]
    }

    pub(crate) mod fwd {
        use memchr::memmem;

        pub(crate) fn oneshot(haystack: &str, needle: &str) -> bool {
            memmem::find(haystack.as_bytes(), needle.as_bytes()).is_some()
        }

        pub(crate) fn prebuilt(
            needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            let finder = memmem::Finder::new(needle).into_owned();
            move |h| finder.find(h.as_bytes()).is_some()
        }

        pub(crate) fn oneshotiter<'a>(
            haystack: &'a str,
            needle: &'a str,
        ) -> impl Iterator<Item = usize> + 'a {
            memmem::find_iter(haystack.as_bytes(), needle.as_bytes())
        }

        pub(crate) fn prebuiltiter(needle: &str) -> PrebuiltIter {
            PrebuiltIter(memmem::Finder::new(needle).into_owned())
        }

        #[derive(Debug)]
        pub(crate) struct PrebuiltIter(memmem::Finder<'static>);

        impl PrebuiltIter {
            pub(crate) fn iter<'a>(
                &'a self,
                haystack: &'a str,
            ) -> impl Iterator<Item = usize> + 'a {
                self.0.find_iter(haystack.as_bytes())
            }
        }
    }

    pub(crate) mod rev {
        use memchr::memmem;

        pub(crate) fn oneshot(haystack: &str, needle: &str) -> bool {
            memmem::rfind(haystack.as_bytes(), needle.as_bytes()).is_some()
        }

        pub(crate) fn prebuilt(
            needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            let finder = memmem::FinderRev::new(needle).into_owned();
            move |h| finder.rfind(h.as_bytes()).is_some()
        }

        pub(crate) fn oneshotiter<'a>(
            haystack: &'a str,
            needle: &'a str,
        ) -> impl Iterator<Item = usize> + 'a {
            memmem::rfind_iter(haystack.as_bytes(), needle.as_bytes())
        }

        pub(crate) fn prebuiltiter(needle: &str) -> PrebuiltIter {
            PrebuiltIter(memmem::FinderRev::new(needle).into_owned())
        }

        #[derive(Debug)]
        pub(crate) struct PrebuiltIter(memmem::FinderRev<'static>);

        impl PrebuiltIter {
            pub(crate) fn iter<'a>(
                &'a self,
                haystack: &'a str,
            ) -> impl Iterator<Item = usize> + 'a {
                self.0.rfind_iter(haystack.as_bytes())
            }
        }
    }
}

/// memchr's implementation of memmem, but without prefilters enabled. This
/// exists because sometimes prefilters aren't the right choice, and it's good
/// to be able to compare it against prefilter-accelerated searches to see
/// where this might be faster.
pub(crate) mod krate_nopre {
    pub(crate) fn available(_: &str) -> &'static [&'static str] {
        &["reverse", "oneshot", "prebuilt", "oneshotiter", "prebuiltiter"]
    }

    pub(crate) mod fwd {
        use memchr::memmem;

        fn finder(needle: &[u8]) -> memmem::Finder<'_> {
            memmem::FinderBuilder::new()
                .prefilter(memmem::Prefilter::None)
                .build_forward(needle)
        }

        pub(crate) fn oneshot(haystack: &str, needle: &str) -> bool {
            finder(needle.as_bytes()).find(haystack.as_bytes()).is_some()
        }

        pub(crate) fn prebuilt(
            needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            let finder = finder(needle.as_bytes()).into_owned();
            move |h| finder.find(h.as_bytes()).is_some()
        }

        pub(crate) fn oneshotiter<'a>(
            haystack: &'a str,
            needle: &'a str,
        ) -> impl Iterator<Item = usize> + 'a {
            super::super::iter_from_find(
                haystack.as_bytes(),
                needle.as_bytes(),
                |h, n| finder(n).find(h),
            )
        }

        pub(crate) fn prebuiltiter(needle: &str) -> PrebuiltIter {
            PrebuiltIter(finder(needle.as_bytes()).into_owned())
        }

        #[derive(Debug)]
        pub(crate) struct PrebuiltIter(memmem::Finder<'static>);

        impl PrebuiltIter {
            pub(crate) fn iter<'a>(
                &'a self,
                haystack: &'a str,
            ) -> impl Iterator<Item = usize> + 'a {
                self.0.find_iter(haystack.as_bytes())
            }
        }
    }

    // N.B. memrmem/krate_nopre and memrmem/krate should be equivalent for now
    // since reverse searching doesn't have any prefilter support.
    pub(crate) mod rev {
        use memchr::memmem;

        fn finder(needle: &[u8]) -> memmem::FinderRev<'_> {
            memmem::FinderBuilder::new()
                .prefilter(memmem::Prefilter::None)
                .build_reverse(needle)
        }

        pub(crate) fn oneshot(haystack: &str, needle: &str) -> bool {
            finder(needle.as_bytes()).rfind(haystack.as_bytes()).is_some()
        }

        pub(crate) fn prebuilt(
            needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            let finder = finder(needle.as_bytes()).into_owned();
            move |h| finder.rfind(h.as_bytes()).is_some()
        }

        pub(crate) fn oneshotiter<'a>(
            haystack: &'a str,
            needle: &'a str,
        ) -> impl Iterator<Item = usize> + 'a {
            super::super::iter_from_rfind(
                haystack.as_bytes(),
                needle.as_bytes(),
                |h, n| finder(n).rfind(h),
            )
        }

        pub(crate) fn prebuiltiter(needle: &str) -> PrebuiltIter {
            PrebuiltIter(finder(needle.as_bytes()).into_owned())
        }

        #[derive(Debug)]
        pub(crate) struct PrebuiltIter(memmem::FinderRev<'static>);

        impl PrebuiltIter {
            pub(crate) fn iter<'a>(
                &'a self,
                haystack: &'a str,
            ) -> impl Iterator<Item = usize> + 'a {
                self.0.rfind_iter(haystack.as_bytes())
            }
        }
    }
}

/// bstr's implementation of memmem.
///
/// The implementation in this crate was originally copied from bstr.
/// Eventually, bstr will just use the implementation in this crate, but at time
/// of writing, it was useful to benchmark against the "original" version.
pub(crate) mod bstr {
    pub(crate) fn available(_: &str) -> &'static [&'static str] {
        &["reverse", "oneshot", "prebuilt", "oneshotiter", "prebuiltiter"]
    }

    pub(crate) mod fwd {
        pub(crate) fn oneshot(haystack: &str, needle: &str) -> bool {
            bstr::ByteSlice::find(haystack.as_bytes(), needle.as_bytes())
                .is_some()
        }

        pub(crate) fn prebuilt(
            needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            let finder = bstr::Finder::new(needle).into_owned();
            move |h| finder.find(h.as_bytes()).is_some()
        }

        pub(crate) fn oneshotiter<'a>(
            haystack: &'a str,
            needle: &'a str,
        ) -> impl Iterator<Item = usize> + 'a {
            bstr::ByteSlice::find_iter(haystack.as_bytes(), needle.as_bytes())
        }

        pub(crate) fn prebuiltiter(needle: &str) -> PrebuiltIter {
            PrebuiltIter(bstr::Finder::new(needle).into_owned())
        }

        #[derive(Debug)]
        pub(crate) struct PrebuiltIter(bstr::Finder<'static>);

        impl PrebuiltIter {
            pub(crate) fn iter<'a>(
                &'a self,
                haystack: &'a str,
            ) -> impl Iterator<Item = usize> + 'a {
                super::super::iter_from_find(
                    haystack.as_bytes(),
                    self.0.needle(),
                    move |h, _| self.0.find(h),
                )
            }
        }
    }

    pub(crate) mod rev {
        pub(crate) fn oneshot(haystack: &str, needle: &str) -> bool {
            bstr::ByteSlice::rfind(haystack.as_bytes(), needle.as_bytes())
                .is_some()
        }

        pub(crate) fn prebuilt(
            needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            let finder = bstr::FinderReverse::new(needle).into_owned();
            move |h| finder.rfind(h.as_bytes()).is_some()
        }

        pub(crate) fn oneshotiter<'a>(
            haystack: &'a str,
            needle: &'a str,
        ) -> impl Iterator<Item = usize> + 'a {
            bstr::ByteSlice::rfind_iter(haystack.as_bytes(), needle.as_bytes())
        }

        pub(crate) fn prebuiltiter(needle: &str) -> PrebuiltIter {
            PrebuiltIter(bstr::FinderReverse::new(needle).into_owned())
        }

        #[derive(Debug)]
        pub(crate) struct PrebuiltIter(bstr::FinderReverse<'static>);

        impl PrebuiltIter {
            pub(crate) fn iter<'a>(
                &'a self,
                haystack: &'a str,
            ) -> impl Iterator<Item = usize> + 'a {
                super::super::iter_from_rfind(
                    haystack.as_bytes(),
                    self.0.needle(),
                    move |h, _| self.0.rfind(h),
                )
            }
        }
    }
}

/// regex's implementation of substring search.
///
/// regex is where the concept of using heuristics based on an a priori
/// assumption of byte frequency originated. Eventually, regex will just use the
/// implementation in this crate, but it will still be useful to benchmark since
/// regex tends to have higher latency. It would be good to measure that.
///
/// For regex, we don't provide oneshots, since that requires compiling the
/// regex which we know is going to be ridiculously slow. No real need to
/// measure it I think.
pub(crate) mod regex {
    pub(crate) fn available(_: &str) -> &'static [&'static str] {
        &["prebuilt", "prebuiltiter"]
    }

    pub(crate) mod fwd {
        pub(crate) fn oneshot(_haystack: &str, _needle: &str) -> bool {
            unimplemented!("regex does not support oneshot searches")
        }

        pub(crate) fn prebuilt(
            needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            let finder = regex::Regex::new(&regex::escape(needle)).unwrap();
            move |h| finder.is_match(h)
        }

        pub(crate) fn oneshotiter(
            _haystack: &str,
            _needle: &str,
        ) -> impl Iterator<Item = usize> + 'static {
            std::iter::from_fn(move || {
                unimplemented!("regex does not support oneshot searches")
            })
        }

        pub(crate) fn prebuiltiter(needle: &str) -> PrebuiltIter {
            PrebuiltIter(regex::Regex::new(&regex::escape(needle)).unwrap())
        }

        #[derive(Debug)]
        pub(crate) struct PrebuiltIter(regex::Regex);

        impl PrebuiltIter {
            pub(crate) fn iter<'a>(
                &'a self,
                haystack: &'a str,
            ) -> impl Iterator<Item = usize> + 'a {
                self.0.find_iter(haystack).map(|m| m.start())
            }
        }
    }

    pub(crate) mod rev {
        pub(crate) fn oneshot(_haystack: &str, _needle: &str) -> bool {
            unimplemented!("regex does not support reverse searches")
        }

        pub(crate) fn prebuilt(
            _needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            |_| unimplemented!("regex does not support reverse searches")
        }

        pub(crate) fn oneshotiter(
            _haystack: &str,
            _needle: &str,
        ) -> impl Iterator<Item = usize> + 'static {
            std::iter::from_fn(move || {
                unimplemented!("regex does not support reverse searches")
            })
        }

        pub(crate) fn prebuiltiter(_needle: &str) -> super::super::NoIter {
            unimplemented!("regex does not support reverse searches")
        }
    }
}

/// std's substring search implementation.
///
/// std uses Two-Way like this crate, but doesn't have any prefilter
/// heuristics.
///
/// std doesn't have any way to amortize the construction of the searcher, so
/// we can't implement any of the prebuilt routines.
pub(crate) mod stud {
    pub(crate) fn available(_: &str) -> &'static [&'static str] {
        &["reverse", "oneshot", "oneshotiter"]
    }

    pub(crate) mod fwd {
        pub(crate) fn oneshot(haystack: &str, needle: &str) -> bool {
            haystack.contains(needle)
        }

        pub(crate) fn prebuilt(
            _needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            |_| unimplemented!("std does not support prebuilt searches")
        }

        pub(crate) fn oneshotiter<'a>(
            haystack: &'a str,
            needle: &'a str,
        ) -> impl Iterator<Item = usize> + 'a {
            haystack.match_indices(needle).map(|(i, _)| i)
        }

        pub(crate) fn prebuiltiter(_needle: &str) -> super::super::NoIter {
            super::super::NoIter { imp: "std" }
        }
    }

    pub(crate) mod rev {
        pub(crate) fn oneshot(haystack: &str, needle: &str) -> bool {
            haystack.contains(needle)
        }

        pub(crate) fn prebuilt(
            _needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            |_| unimplemented!("std does not support prebuilt searches")
        }

        pub(crate) fn oneshotiter<'a>(
            haystack: &'a str,
            needle: &'a str,
        ) -> impl Iterator<Item = usize> + 'a {
            haystack.rmatch_indices(needle).map(|(i, _)| i)
        }

        pub(crate) fn prebuiltiter(_needle: &str) -> super::super::NoIter {
            super::super::NoIter { imp: "std" }
        }
    }
}

/// Substring search from the twoway crate.
///
/// twoway uses, obviously, Two-Way as an implementation. AIUI, it was taken
/// from std at some point but heavily modified to support a prefilter via
/// PCMPESTRI from the SSE 4.2 ISA extension. (And also uses memchr for
/// single-byte needles.)
///
/// Like std, there is no way to amortize the construction of the searcher, so
/// we can't implement any of the prebuilt routines.
pub(crate) mod twoway {
    pub(crate) fn available(_: &str) -> &'static [&'static str] {
        &["reverse", "oneshot", "oneshotiter"]
    }

    pub(crate) mod fwd {
        pub(crate) fn oneshot(haystack: &str, needle: &str) -> bool {
            twoway::find_bytes(haystack.as_bytes(), needle.as_bytes())
                .is_some()
        }

        pub(crate) fn prebuilt(
            _needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            |_| unimplemented!("twoway does not support prebuilt searches")
        }

        pub(crate) fn oneshotiter<'a>(
            haystack: &'a str,
            needle: &'a str,
        ) -> impl Iterator<Item = usize> + 'a {
            super::super::iter_from_find(
                haystack.as_bytes(),
                needle.as_bytes(),
                twoway::find_bytes,
            )
        }

        pub(crate) fn prebuiltiter(_needle: &str) -> super::super::NoIter {
            super::super::NoIter { imp: "twoway" }
        }
    }

    pub(crate) mod rev {
        pub(crate) fn oneshot(haystack: &str, needle: &str) -> bool {
            twoway::rfind_bytes(haystack.as_bytes(), needle.as_bytes())
                .is_some()
        }

        pub(crate) fn prebuilt(
            _needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            |_| unimplemented!("twoway does not support prebuilt searches")
        }

        pub(crate) fn oneshotiter<'a>(
            haystack: &'a str,
            needle: &'a str,
        ) -> impl Iterator<Item = usize> + 'a {
            super::super::iter_from_rfind(
                haystack.as_bytes(),
                needle.as_bytes(),
                twoway::rfind_bytes,
            )
        }

        pub(crate) fn prebuiltiter(_needle: &str) -> super::super::NoIter {
            super::super::NoIter { imp: "twoway" }
        }
    }
}

/// Substring search from the sliceslice crate.
///
/// This crate is what inspired me to write a vectorized memmem implementation
/// in the memchr crate in the first place. In particular, it exposed some
/// serious weaknesses in my implementation in the bstr crate.
///
/// sliceslice doesn't actually do anything "new" other
/// than bringing a long known SIMD algorithm to Rust:
/// http://0x80.pl/articles/simd-strfind.html#algorithm-1-generic-simd
///
/// The main thrust of the algorithm is that it picks a couple of bytes in the
/// needle and uses SIMD to check whether those two bytes occur in the haystack
/// in a way that could lead to a match. If so, then you do a simple memcmp
/// confirmation step. The main problem with this algorithm is that its worst
/// case is multiplicative: that confirmatory step can become quite costly if
/// the SIMD prefilter isn't effective. The elegance of this method, however,
/// is that the prefilter is routinely effective.
///
/// The essence of memchr's implementation of memmem comes from sliceslice,
/// but also from regex's original idea to use heuristics based on an a priori
/// assumption of relative byte frequency AND from bstr's desire to have a
/// constant space and worst case O(m+n) substring search. My claim is that
/// it is the best of all words, and that's why this benchmark suite is so
/// comprehensive. There are a lot of cases and implementations to test.
///
/// NOTE: The API of sliceslice is quite constrained. My guess is that it was
/// designed for a very specific use case, and the API is heavily constrained
/// to that use case (whatever it is). While its API doesn't provide any
/// oneshot routines, we emulate them. (Its main problem is that every such
/// search requires copying the needle into a fresh allocation. The memchr
/// crate avoids that problem by being generic over the needle: it can be owned
/// or borrowed.) Also, since the API only enables testing whether a substring
/// exists or not, we can't benchmark iteration.
///
/// NOTE: sliceslice only works on x86_64 CPUs with AVX enabled. So not only
/// do we conditionally compile the routines below, but we only run these
/// benchmarks when AVX2 is available.
#[cfg(target_arch = "x86_64")]
pub(crate) mod sliceslice {
    pub(crate) fn available(needle: &str) -> &'static [&'static str] {
        // Apparently sliceslice doesn't support searching with an empty
        // needle. Sheesh.
        if !needle.is_empty() && is_x86_feature_detected!("avx2") {
            &["oneshot", "prebuilt"]
        } else {
            &[]
        }
    }

    pub(crate) mod fwd {
        pub(crate) fn oneshot(haystack: &str, needle: &str) -> bool {
            if !is_x86_feature_detected!("avx2") {
                unreachable!("sliceslice cannot be called without avx2");
            }
            let needle = needle.as_bytes();
            // SAFETY: This code path is only entered when AVX2 is enabled,
            // which is the only requirement for using DynamicAvx2Searcher.
            unsafe {
                let finder = sliceslice::x86::DynamicAvx2Searcher::new(needle);
                finder.search_in(haystack.as_bytes())
            }
        }

        pub(crate) fn prebuilt(
            needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            if !is_x86_feature_detected!("avx2") {
                unreachable!("sliceslice cannot be called without avx2");
            }
            let needle = needle.as_bytes().to_vec();
            // SAFETY: This code path is only entered when AVX2 is enabled,
            // which is the only requirement for using DynamicAvx2Searcher.
            unsafe {
                let finder = sliceslice::x86::DynamicAvx2Searcher::new(needle);
                move |h| finder.search_in(h.as_bytes())
            }
        }

        pub(crate) fn oneshotiter(
            _haystack: &str,
            _needle: &str,
        ) -> impl Iterator<Item = usize> + 'static {
            std::iter::from_fn(move || {
                unimplemented!("sliceslice doesn't not support iteration")
            })
        }

        pub(crate) fn prebuiltiter(_needle: &str) -> super::super::NoIter {
            unimplemented!("sliceslice doesn't support prebuilt iteration")
        }
    }

    pub(crate) mod rev {
        pub(crate) fn oneshot(_haystack: &str, _needle: &str) -> bool {
            unimplemented!("sliceslice does not support reverse searches")
        }

        pub(crate) fn prebuilt(
            _needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            |_| unimplemented!("sliceslice does not support reverse searches")
        }

        pub(crate) fn oneshotiter(
            _haystack: &str,
            _needle: &str,
        ) -> impl Iterator<Item = usize> + 'static {
            std::iter::from_fn(move || {
                unimplemented!("sliceslice does not support reverse searches")
            })
        }

        pub(crate) fn prebuiltiter(_needle: &str) -> super::super::NoIter {
            unimplemented!("sliceslice does not support reverse searches")
        }
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub(crate) mod sliceslice {
    pub(crate) fn available(_: &str) -> &'static [&'static str] {
        &[]
    }

    pub(crate) mod fwd {
        pub(crate) fn oneshot(_: &str, _: &str) -> bool {
            unimplemented!("sliceslice only runs on x86")
        }

        pub(crate) fn prebuilt(_: &str) -> impl Fn(&str) -> bool + 'static {
            unimplemented!("sliceslice only runs on x86")
        }

        pub(crate) fn oneshotiter<'a>(
            haystack: &'a str,
            needle: &'a str,
        ) -> impl Iterator<Item = usize> + 'static {
            std::iter::from_fn(move || {
                unimplemented!("sliceslice only runs on x86")
            })
        }

        pub(crate) fn prebuiltiter(needle: &str) -> super::super::NoIter {
            unimplemented!("sliceslice only runs on x86")
        }
    }

    pub(crate) mod rev {
        pub(crate) fn oneshot(haystack: &str, needle: &str) -> bool {
            unimplemented!("sliceslice does not support reverse searches")
        }

        pub(crate) fn prebuilt(
            needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            |_| unimplemented!("sliceslice does not support reverse searches")
        }

        pub(crate) fn oneshotiter(
            haystack: &str,
            needle: &str,
        ) -> impl Iterator<Item = usize> + 'static {
            std::iter::from_fn(move || {
                unimplemented!("sliceslice does not support reverse searches")
            })
        }

        pub(crate) fn prebuiltiter(needle: &str) -> super::super::NoIter {
            unimplemented!("sliceslice does not support reverse searches")
        }
    }
}

/// libc's substring search implementation.
///
/// libc doesn't have any way to amortize the construction of the searcher, so
/// we can't implement any of the prebuilt routines.
pub(crate) mod libc {
    pub(crate) fn available(_: &str) -> &'static [&'static str] {
        &["oneshot", "oneshotiter"]
    }

    pub(crate) mod fwd {
        fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
            let p = unsafe {
                libc::memmem(
                    haystack.as_ptr() as *const libc::c_void,
                    haystack.len(),
                    needle.as_ptr() as *const libc::c_void,
                    needle.len(),
                )
            };
            if p.is_null() {
                None
            } else {
                Some(p as usize - (haystack.as_ptr() as usize))
            }
        }

        pub(crate) fn oneshot(haystack: &str, needle: &str) -> bool {
            find(haystack.as_bytes(), needle.as_bytes()).is_some()
        }

        pub(crate) fn prebuilt(
            _needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            |_| unimplemented!("std does not support prebuilt searches")
        }

        pub(crate) fn oneshotiter<'a>(
            haystack: &'a str,
            needle: &'a str,
        ) -> impl Iterator<Item = usize> + 'a {
            super::super::iter_from_find(
                haystack.as_bytes(),
                needle.as_bytes(),
                find,
            )
        }

        pub(crate) fn prebuiltiter(_needle: &str) -> super::super::NoIter {
            super::super::NoIter { imp: "libc" }
        }
    }

    pub(crate) mod rev {
        pub(crate) fn oneshot(_haystack: &str, _needle: &str) -> bool {
            unimplemented!("libc does not support reverse searches")
        }

        pub(crate) fn prebuilt(
            _needle: &str,
        ) -> impl Fn(&str) -> bool + 'static {
            |_| unimplemented!("libc does not support reverse searches")
        }

        pub(crate) fn oneshotiter<'a>(
            _haystack: &'a str,
            _needle: &'a str,
        ) -> impl Iterator<Item = usize> + 'a {
            std::iter::from_fn(move || {
                unimplemented!("libc does not support reverse searches")
            })
        }

        pub(crate) fn prebuiltiter(_needle: &str) -> super::super::NoIter {
            unimplemented!("libc does not support reverse searches")
        }
    }
}

/// An iterator that looks like a PrebuilIter API-wise, but panics if it's
/// called. This should be used for implementations that don't support
/// prebuilt iteration.
#[derive(Debug)]
pub(crate) struct NoIter {
    /// The name of the impl to use in the panic message in case it is invoked
    /// by mistake. (But the benchmark harness should not invoke it, assuming
    /// each impl's 'available' function is correct.
    imp: &'static str,
}

impl NoIter {
    pub(crate) fn iter(
        &self,
        _: &str,
    ) -> impl Iterator<Item = usize> + 'static {
        let imp = self.imp;
        std::iter::from_fn(move || {
            unimplemented!("{} does not support prebuilt iteration", imp)
        })
    }
}

/// Accepts a corpus and a needle and a routine that implements substring
/// search, and returns an iterator over all matches. This is useful for
/// benchmarking "find all matches" for substring search implementations that
/// don't expose a native way to do this.
///
/// The closure given takes two parameters: the corpus and needle, in that
/// order.
fn iter_from_find<'a>(
    haystack: &'a [u8],
    needle: &'a [u8],
    mut find: impl FnMut(&[u8], &[u8]) -> Option<usize> + 'a,
) -> impl Iterator<Item = usize> + 'a {
    let mut pos = 0;
    std::iter::from_fn(move || {
        if pos > haystack.len() {
            return None;
        }
        match find(&haystack[pos..], needle) {
            None => None,
            Some(i) => {
                let found = pos + i;
                // We always need to add at least 1, in case of an empty needle.
                pos += i + std::cmp::max(1, needle.len());
                Some(found)
            }
        }
    })
}

/// Like iter_from_find, but for reverse searching.
fn iter_from_rfind<'a>(
    haystack: &'a [u8],
    needle: &'a [u8],
    mut rfind: impl FnMut(&[u8], &[u8]) -> Option<usize> + 'a,
) -> impl Iterator<Item = usize> + 'a {
    let mut pos = Some(haystack.len());
    std::iter::from_fn(move || {
        let end = match pos {
            None => return None,
            Some(end) => end,
        };
        match rfind(&haystack[..end], needle) {
            None => None,
            Some(i) => {
                if end == i {
                    // We always need to subtract at least 1, in case of an
                    // empty needle.
                    pos = end.checked_sub(1);
                } else {
                    pos = Some(i);
                }
                Some(i)
            }
        }
    })
}
