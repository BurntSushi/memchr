This crate provides a single function, `memchr`, which exposes a safe interface
to the corresponding function in `libc`.

[![Build status](https://api.travis-ci.org/BurntSushi/rust-memchr.png)](https://travis-ci.org/BurntSushi/rust-memchr)
[![](http://meritbadge.herokuapp.com/memchr)](https://crates.io/crates/memchr)

Dual-licensed under MIT or the [UNLICENSE](http://unlicense.org).


### Documentation

[http://burntsushi.net/rustdoc/memchr/](http://burntsushi.net/rustdoc/memchr/).


### Performance

On my system (Linux/amd64), `memchr` is about an order of magnitude faster than
the more idiomatic `haystack.iter().position(|&b| b == needle)`:

```
test iterator    ... bench:       5,301 ns/iter (+/- 2,745)
test libc_memchr ... bench:         203 ns/iter (+/- 17)
```
