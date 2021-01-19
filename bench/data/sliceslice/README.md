These benchmark inputs were taken from the sliceslice [project benchmarks][1].

These inputs drive two benchmarks, one on short haystacks and the other on long
haystacks, with a slightly unusual but interesting configuration. Neither of
these benchmarks include the time it takes to build a searcher. They only
measure actual search time.

The short haystack benchmark starts by loading all of the words in `words.txt`
into memory and sorting them in ascending order by their length. Then, a
substring searcher is created for each of these words in the same order. The
actual benchmark consists of executing each searcher once on every needle that
appears after it in the list. In essence, this benchmark tests how quickly the
implementation can deal with tiny haystacks. The results of this benchmark tend
to come down to how much overhead the implementation has. In other words, this
benchmark tests latency.

The long haystack benchmark has a setup similar to the short haystack
benchmark, except it also loads the contents of `i386.txt` into memory. The
actual benchmark itself executes each of the searchers built (from `words.txt`)
on the `i386.txt` haystack. This benchmark, executing on a much longer
haystack, tests throughput as opposed to latency across a wide variety of
needles.

[1]: https://github.com/cloudflare/sliceslice-rs
