This directory contains some selected benchmark runs on this crate. Each
subdirectory corresponds to a single run. In general, each run should contain a
complete set of benchmarks defined at the time they were run.

Each sub-directory has two files:

* `raw.json` is the raw JSON export of the benchmark data. This is best read
  with `critcmp` (see below), which can slice and dice it any number of ways.
* `README.md` contains details about the benchmarking environment along with
  two tables: one table comparing memchr implementations and another comparing
  memmem implementations. The tables are generated via the `make-pretty-tables`
  script in this directory. (This is not all benchmarks. It leaves out reverse
  searching and the `memchr2` and `memchr3` benchmarks.)

The tables in the README are primarily meant to make the data _somewhat_ easy
to read in GitHub. But in reality, `critcmp` is the best way to read the
benchmarks. For example, let's say you wanted to compare only this crate's
memmem implementation and std's substring search routine, but only benchmarks
which differ by 5% or more. `critcmp` can do this for you:

```
critcmp runs/2021-04-30_initial/raw.json -g 'memmem/[^/]+/(.*)' -f '/(krate|stud)/' -t5
```

(NOTE: Consult the README in the corresponding run directory for the `critcmp`
version used to generate the `raw.json` file. You may need to use the same
version of `critcmp` to read it.)

This command does four things:

* Reads the benchmark data from `runs/2021-04-30_initial/raw.json`.
* Compares all benchmarks against each other where the values of the capturing
  groups in `memmem/[^/]+/(.*)` are equivalent in the benchmark names.
* Limits the benchmark comparisons to those corresponding to either this
  crate's implementation (named `krate`) or std's implementation (named
  `stud`).
* Only prints comparisons with a difference of 5% or more.

For more info on specific benchmarks, see the source code and `data` directory.
