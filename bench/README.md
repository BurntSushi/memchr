This directory defines a large suite of benchmarks for both the memchr and
memmem APIs in this crate. A selection of "competitor" implementations are
chosen. In general, benchmarks are meant to be a tool for optimization. That's
why there is so many: we want to be sure we get enough coverage such that our
benchmarks approximate real world usage. When some benchmarks look a bit slower
than we expect (for one reason another), we can use profiling tools to look at
codegen and attempt to improve that case.

Because there are so many benchmarks, if you run all of them, you might want to
step away for a cup of coffee (or two). Therefore, the typical way to run them
is to select a subset. For example,

```
$ cargo bench -- 'memmem/krate/.*never.*'
```

runs all benchmarks for the memmem implementation in this crate with searches
that never produce any matches. This will still take a bit, but perhaps only a
few minutes.

Running a specific benchmark can be useful for profiling. For example, if you
want to see where `memmem/krate/prebuiltiter/huge-en/common-one-space` is
spending all of its time, you would first want to run it (to make sure the code
is compiled):

```
$ cargo bench -- memmem/krate/prebuiltiter/huge-en/common-one-space
```

And then run it under your profiling tool (I use `perf` on Linux):

```
$ perfr --callgraph cargo bench -- memmem/krate/prebuiltiter/huge-en/common-one-space --profile-time 3
```

Where
[`perfr` is my own wrapper around `perf`](https://github.com/BurntSushi/dotfiles/blob/master/bin/perfr),
and the `--profile-time 3` flag means, "just run the code for 3 seconds, but
don't do anything else." This makes the benchmark harness get out of the way,
which lets the profile focus as much as possible on the code being measured.

See the README in the `runs` directory for a bit more info on how to use
`critcmp` to look at benchmark data in a way that makes it easy to do
comparisons.
