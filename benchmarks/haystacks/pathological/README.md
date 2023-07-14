These data sets are specifically crafted to try and defeat heuristic
optimizations in various substring search implementations. The point of these
is to make the costs of those heuristics clearer. In particular, the main idea
behind heuristics is to sell out some rare or edge cases in favor of making
some common cases *a lot* faster (potentially by orders of magnitude). The key
to this is to make sure that those edge cases are impacted at tolerable levels.

Below is a description of each.

* `repeated-rare-*`: This is meant to be used with the needle `abczdef`. This
  input defeats a heuristic in the old bstr and regex substring implementations
  that looked for a rare byte (in this case, `z`) to run memchr on before
  looking for an actual match. This particular input causes that heuristic to
  stop on every byte in the input. In regex's case in particular, this causes
  `O(mn)` time complexity. (In the case of `bstr`, it does a little better by
  stopping this heuristic after a number of tries once it becomes clear that it
  is ineffective.)
* `defeat-simple-vector`: The corpus consists of `qaz` repeated over and over
  again. The intended needle is `qbz`. This is meant to be difficult for the
  "generic SIMD" algorithm[1] to handle. Namely, it will repeatedly find a
  candidate match via the `q` and `z` bytes in the needle, but the overall
  match will fail at the `memcmp` phase. Nevertheless, optimized versions of
  [1] still do reasonably well on this benchmark because the `memcmp` can be
  specialized to a single `u32` unaligned load and compare.
* `defeat-simple-vector-freq`: This is similarish to `defeat-simple-vector`,
  except it also attempts to defeat heuristic frequency analysis. The corpus
  consists of `qjaz` repeated over and over again, with the intended needle
  being `qja{49}z`. Heuristic frequency analysis might try either the `q` or
  the `j`, in addition to `z`. Given the nature of the corpus, this will result
  in a lot of false positive candidates, thus leading to an ineffective
  prefilter.
* `defeat-simple-vector-repeated`: This combines the "repeated-rare" and
  "defeat-simple-vector" inputs. The corpus consists of `z` entirely, with only
  the second to last byte being changed to `a`. The intended needle is
  `z{135}az`. The key here is that in [1], a candidate match will be found at
  every position in the haystack. And since the needle is very large, this will
  result in a full `memcmp` call out. [1] effectively drowns in `memcmp` being
  called at every position in the haystack. The algorithm in this crate does
  a bit better by noticing that the prefilter is ineffective and falling back
  to standard Two-Way.
* `md5-huge`: This file contains one md5 hash per line for each word in the
  `../sliceslice/words.txt` corpus. The intent of this benchmark is to defeat
  frequency heuristics by using a corpus comprised of random data. That is,
  no one bytes should be significantly more frequent than any other.
* `random-huge`: Similar to `md5-huge`, but with longer lines and more
  princpally random data. Generated via
  `dd if=/dev/urandom bs=32 count=10000 | xxd -ps -c32`.
  This was derived from a real world benchmark reported to ripgrep[2].
  In particular, it originally motivated the addition of Boyer-Moore to
  the regex crate, but now this case is handled just fine by the memmem
  implementation in this crate.

[1]: http://0x80.pl/articles/simd-strfind.html#algorithm-1-generic-simd
[2]: https://github.com/BurntSushi/ripgrep/issues/617
