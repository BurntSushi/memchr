These needles were taken from the `sliceslice` [project benchmarks][1].

These inputs drive two benchmarks, one on short haystacks and the other on long
haystacks. For the short haystack case, we actually search the needles themselves.
That is, for each needle we count the number of needles in which that needle is
contained.

For the long haystack case, we just take the haystacks used in the `sliceslice`
benchmarks.

[1]: https://github.com/cloudflare/sliceslice-rs
