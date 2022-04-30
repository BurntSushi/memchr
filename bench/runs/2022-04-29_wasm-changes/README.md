# Benchmark environment

* Baseline: 8e1da98fee06d66c13e66c330e3a3dd6ccf0e3a0
* Test: cfa2fda7d2b6cbe6c8a161421d4665181dc5e3b5
* Intel i5-7600 @ 3.5 GHz
* Linux 5.16.15
* `criterion 0.3.4`
* `critcmp 0.1.7`

These benchmarks were run after the initial [WASM PR] was merged. The main idea
here was to compare the non-WASM benchmarks before and after the PR merged to
ensure there were no serious regressions. Why test non-WASM? Because there
was some refactoring involved to add WASM support to the various dispatch
mechanisms. There does appear to be *some* slowdown in some cases, but also
some speedup in other cases. (Try `critcmp baseline.json test.json -t10
-fkrate` in this directory.) But it's hard to tell whether this is noise or
not. Probably not, since the various routines for constructing and executing
the search do seem to be quite sensitive in terms of codegen. There is some
nasty black art involved here. But there wasn't enough of a change IMO to
bother with this too much.

[WASM PR]: https://github.com/BurntSushi/memchr/pull/84

# memchr benchmarks

```
group               2022-04-29_wasm-changes/memchr1/fallback/    2022-04-29_wasm-changes/memchr1/krate/    2022-04-29_wasm-changes/memchr1/libc/    2022-04-29_wasm-changes/memchr1/naive/
-----               -----------------------------------------    --------------------------------------    -------------------------------------    --------------------------------------
empty/never         74.60     2.3±0.04ns        0 B/sec          1.00      0.0±0.00ns        0 B/sec       80.25     2.5±0.05ns        0 B/sec      16.02     0.5±0.00ns        0 B/sec
huge/common         3.01    665.4±0.71µs   852.7 MB/sec          1.00    220.8±0.42µs     2.5 GB/sec       1.16    256.3±0.54µs     2.2 GB/sec      2.06    454.2±0.40µs  1249.2 MB/sec
huge/never          4.93     42.0±0.02µs    13.2 GB/sec          1.00      8.5±0.01µs    65.0 GB/sec       1.07      9.1±0.02µs    60.6 GB/sec      17.14   146.1±0.18µs     3.8 GB/sec
huge/rare           4.69     44.5±0.02µs    12.5 GB/sec          1.00      9.5±0.01µs    58.4 GB/sec       1.01      9.6±0.01µs    57.8 GB/sec      15.55   147.5±0.10µs     3.8 GB/sec
huge/uncommon       2.32    164.1±0.27µs     3.4 GB/sec          1.09     77.2±0.12µs     7.2 GB/sec       1.00     70.8±0.14µs     7.8 GB/sec      2.81    198.9±0.28µs     2.8 GB/sec
huge/verycommon     2.61   1199.1±4.70µs   473.2 MB/sec          1.00    459.2±0.59µs  1235.5 MB/sec       1.16    533.2±0.73µs  1064.1 MB/sec      2.00    919.5±1.76µs   617.0 MB/sec
small/common        1.34    274.5±0.53ns     2.3 GB/sec          1.00    204.7±0.23ns     3.0 GB/sec       1.09    222.1±0.18ns     2.8 GB/sec      1.19    242.9±2.14ns     2.5 GB/sec
small/never         8.34     56.9±0.10ns    10.9 GB/sec          1.00      6.8±0.01ns    90.7 GB/sec       1.02      6.9±0.01ns    89.2 GB/sec      25.35   172.8±0.16ns     3.6 GB/sec
small/rare          5.71     63.5±0.98ns     9.7 GB/sec          1.00     11.1±0.02ns    55.6 GB/sec       1.01     11.2±0.01ns    55.1 GB/sec      16.37   182.1±1.35ns     3.4 GB/sec
small/uncommon      2.07     94.4±0.07ns     6.6 GB/sec          1.00     45.7±0.07ns    13.5 GB/sec       1.04     47.6±0.03ns    13.0 GB/sec      4.58    209.2±3.05ns     3.0 GB/sec
small/verycommon    1.64    543.0±0.44ns  1166.2 MB/sec          1.48    491.3±0.98ns  1288.9 MB/sec       1.75    579.3±0.51ns  1093.1 MB/sec      1.00    330.9±6.34ns  1913.9 MB/sec
tiny/common         1.51     54.7±0.08ns  1202.5 MB/sec          1.39     50.5±0.05ns  1304.1 MB/sec       1.42     51.5±0.10ns  1276.8 MB/sec      1.00     36.3±0.02ns  1811.3 MB/sec
tiny/never          2.50      8.0±0.02ns     8.1 GB/sec          1.14      3.6±0.02ns    17.6 GB/sec       1.00      3.2±0.00ns    20.1 GB/sec      8.69     27.7±0.11ns     2.3 GB/sec
tiny/rare           2.18     11.3±0.01ns     5.7 GB/sec          1.14      5.9±0.03ns    10.9 GB/sec       1.00      5.2±0.01ns    12.4 GB/sec      5.44     28.2±0.07ns     2.3 GB/sec
tiny/uncommon       2.28     40.2±0.04ns  1635.5 MB/sec          1.00     17.6±0.01ns     3.6 GB/sec       1.10     19.3±0.05ns     3.3 GB/sec      1.44     25.4±0.25ns     2.5 GB/sec
```

# memmem benchmarks

```
group                                                                2022-04-29_wasm-changes/memmem/krate/    2022-04-29_wasm-changes/memmem/sliceslice/    2022-04-29_wasm-changes/memmem/stud/    2022-04-29_wasm-changes/memmem/twoway/
-----                                                                -------------------------------------    ------------------------------------------    ------------------------------------    --------------------------------------
oneshot/code-rust-library/never-fn-quux                              1.00     51.4±0.05µs    29.9 GB/sec      1.02     52.4±0.08µs    29.3 GB/sec           23.26  1195.8±3.51µs  1314.4 MB/sec     3.12    160.3±0.31µs     9.6 GB/sec
oneshot/code-rust-library/never-fn-strength                          1.09     53.2±0.03µs    28.8 GB/sec      1.00     49.0±0.35µs    31.3 GB/sec           27.16  1330.4±1.96µs  1181.4 MB/sec     4.46    218.2±0.59µs     7.0 GB/sec
oneshot/code-rust-library/never-fn-strength-paren                    1.05     53.2±0.11µs    28.8 GB/sec      1.00     50.9±0.07µs    30.2 GB/sec           24.30  1236.0±3.33µs  1271.6 MB/sec     4.29    218.1±0.41µs     7.0 GB/sec
oneshot/code-rust-library/rare-fn-from-str                           1.09     15.4±0.02µs    99.5 GB/sec      1.00     14.2±0.02µs   108.4 GB/sec           23.03   326.0±0.87µs     4.7 GB/sec     9.06    128.2±0.32µs    12.0 GB/sec
oneshot/huge-en/never-all-common-bytes                               1.13     26.3±0.05µs    21.7 GB/sec      1.00     23.3±0.04µs    24.5 GB/sec           10.57   246.7±0.64µs     2.3 GB/sec     3.98     92.8±0.12µs     6.2 GB/sec
oneshot/huge-en/never-john-watson                                    1.00     16.7±0.03µs    34.3 GB/sec      1.07     17.9±0.03µs    32.0 GB/sec           25.80   429.8±0.94µs  1360.9 MB/sec     4.89     81.5±0.17µs     7.0 GB/sec
oneshot/huge-en/never-some-rare-bytes                                1.00     16.5±0.03µs    34.6 GB/sec      1.07     17.7±0.02µs    32.3 GB/sec           14.38   237.4±0.67µs     2.4 GB/sec     3.50     57.7±0.05µs     9.9 GB/sec
oneshot/huge-en/never-two-space                                      1.12     19.3±0.02µs    29.6 GB/sec      1.00     17.3±0.03µs    33.0 GB/sec           30.61   529.5±0.38µs  1104.7 MB/sec     7.17    124.1±0.11µs     4.6 GB/sec
oneshot/huge-en/rare-huge-needle                                     1.30     39.9±0.08µs    14.3 GB/sec      1.00     30.7±0.05µs    18.6 GB/sec           3.66    112.4±0.18µs     5.1 GB/sec     2.49     76.7±0.11µs     7.5 GB/sec
oneshot/huge-en/rare-long-needle                                     1.00     16.4±0.02µs    34.7 GB/sec      1.96     32.2±0.08µs    17.7 GB/sec           7.22    118.7±0.34µs     4.8 GB/sec     4.64     76.3±0.11µs     7.5 GB/sec
oneshot/huge-en/rare-medium-needle                                   1.00     20.0±0.04µs    28.5 GB/sec      2.66     53.3±0.10µs    10.7 GB/sec           9.68    193.9±0.37µs     2.9 GB/sec     4.03     80.7±0.12µs     7.1 GB/sec
oneshot/huge-en/rare-sherlock                                        1.00     16.6±0.02µs    34.3 GB/sec      1.07     17.8±0.03µs    32.1 GB/sec           16.33   271.8±0.16µs     2.1 GB/sec     4.33     72.1±0.05µs     7.9 GB/sec
oneshot/huge-en/rare-sherlock-holmes                                 1.07     19.2±0.04µs    29.8 GB/sec      1.00     18.0±0.02µs    31.8 GB/sec           16.08   289.3±0.59µs  2021.8 MB/sec     14.39   258.9±0.61µs     2.2 GB/sec
oneshot/huge-ru/never-john-watson                                    1.00     16.7±0.02µs    34.3 GB/sec      5.17     86.1±0.16µs     6.6 GB/sec           19.02   316.8±0.75µs  1846.8 MB/sec     8.77    146.0±0.40µs     3.9 GB/sec
oneshot/huge-ru/rare-sherlock                                        1.00     16.7±0.02µs    34.2 GB/sec      1.95     32.6±0.04µs    17.5 GB/sec           23.68   395.6±0.42µs  1478.8 MB/sec     4.30     71.9±0.12µs     7.9 GB/sec
oneshot/huge-ru/rare-sherlock-holmes                                 1.00     16.7±0.04µs    34.2 GB/sec      4.43     74.0±0.17µs     7.7 GB/sec           16.35   273.1±0.78µs     2.1 GB/sec     13.97   233.3±0.87µs     2.4 GB/sec
oneshot/huge-zh/never-john-watson                                    1.04     19.5±0.04µs    29.2 GB/sec      1.00     18.7±0.01µs    30.5 GB/sec           6.51    121.8±0.29µs     4.7 GB/sec     3.17     59.3±0.09µs     9.6 GB/sec
oneshot/huge-zh/rare-sherlock                                        1.00     16.9±0.04µs    33.8 GB/sec      1.33     22.4±0.03µs    25.5 GB/sec           12.07   203.8±0.50µs     2.8 GB/sec     3.86     65.2±0.11µs     8.8 GB/sec
oneshot/huge-zh/rare-sherlock-holmes                                 1.00     19.7±0.04µs    29.0 GB/sec      1.80     35.4±0.03µs    16.1 GB/sec           7.63    150.3±0.50µs     3.8 GB/sec     3.48     68.5±0.09µs     8.3 GB/sec
oneshot/pathological-defeat-simple-vector-freq/rare-alphabet         1.51     72.0±0.11µs     9.3 GB/sec      8.00    381.1±0.62µs  1802.1 MB/sec           1.00     47.6±0.04µs    14.1 GB/sec     1.95     92.8±0.18µs     7.2 GB/sec
oneshot/pathological-defeat-simple-vector-repeated/rare-alphabet     18.61  1235.3±3.27µs   555.9 MB/sec      34.86     2.3±0.00ms   296.8 MB/sec           16.83  1116.8±2.98µs   614.9 MB/sec     1.00     66.4±0.12µs    10.1 GB/sec
oneshot/pathological-defeat-simple-vector/rare-alphabet              2.45    228.5±0.75µs     2.2 GB/sec      1.29    120.3±0.13µs     4.3 GB/sec           1.34    125.1±0.16µs     4.1 GB/sec     1.00     93.4±0.05µs     5.5 GB/sec
oneshot/pathological-md5-huge/never-no-hash                          1.00      6.0±0.01µs    23.3 GB/sec      1.77     10.7±0.02µs    13.2 GB/sec           4.00     24.2±0.11µs     5.8 GB/sec     3.18     19.2±0.04µs     7.3 GB/sec
oneshot/pathological-md5-huge/rare-last-hash                         1.00      6.0±0.03µs    23.4 GB/sec      1.76     10.6±0.03µs    13.3 GB/sec           4.07     24.5±0.05µs     5.7 GB/sec     3.19     19.2±0.04µs     7.3 GB/sec
oneshot/pathological-repeated-rare-huge/never-tricky                 1.11     15.6±0.02µs    29.8 GB/sec      1.00     14.1±0.03µs    33.0 GB/sec           43.13   608.1±1.80µs   784.4 MB/sec     17.53   247.1±0.17µs  1930.4 MB/sec
oneshot/pathological-repeated-rare-small/never-tricky                1.60     58.1±0.11ns    16.0 GB/sec      1.00     36.3±0.41ns    25.7 GB/sec           36.80  1334.1±3.83ns   715.6 MB/sec     14.53   526.8±1.76ns  1812.2 MB/sec
oneshot/teeny-en/never-all-common-bytes                              1.75     26.6±0.53ns  1002.3 MB/sec      1.00     15.2±0.04ns  1756.3 MB/sec           1.93     29.3±0.02ns   911.3 MB/sec     2.94     44.6±0.15ns   598.2 MB/sec
oneshot/teeny-en/never-john-watson                                   1.49     24.8±0.02ns  1074.6 MB/sec      1.00     16.7±0.04ns  1600.1 MB/sec           1.94     32.4±0.01ns   823.7 MB/sec     2.86     47.7±0.03ns   559.8 MB/sec
oneshot/teeny-en/never-some-rare-bytes                               2.17     28.6±0.65ns   934.2 MB/sec      1.00     13.2±0.03ns  2029.3 MB/sec           1.75     23.0±0.06ns  1159.7 MB/sec     2.03     26.8±0.01ns   998.0 MB/sec
oneshot/teeny-en/never-two-space                                     2.45     29.3±0.42ns   912.7 MB/sec      1.00     11.9±0.03ns     2.2 GB/sec           2.30     27.5±0.02ns   971.1 MB/sec     2.24     26.8±0.01ns   997.9 MB/sec
oneshot/teeny-en/rare-sherlock                                       1.53     20.5±0.01ns  1303.0 MB/sec      1.00     13.4±0.03ns  2000.1 MB/sec           2.69     35.9±0.05ns   743.6 MB/sec     2.85     38.0±0.11ns   702.4 MB/sec
oneshot/teeny-en/rare-sherlock-holmes                                1.04     27.0±0.03ns   988.0 MB/sec      1.00     26.1±0.02ns  1022.9 MB/sec           2.50     65.3±0.03ns   408.7 MB/sec     2.04     53.3±0.17ns   501.3 MB/sec
oneshot/teeny-ru/never-john-watson                                   2.73     40.8±0.39ns   981.3 MB/sec      1.00     15.0±0.04ns     2.6 GB/sec           3.65     54.6±0.03ns   733.0 MB/sec     4.61     69.0±0.20ns   580.5 MB/sec
oneshot/teeny-ru/rare-sherlock                                       1.54     28.1±0.02ns  1424.8 MB/sec      1.00     18.2±0.01ns     2.1 GB/sec           2.75     50.1±0.03ns   799.8 MB/sec     2.53     46.0±0.05ns   870.3 MB/sec
oneshot/teeny-ru/rare-sherlock-holmes                                1.99     37.4±0.02ns  1071.5 MB/sec      1.00     18.7±0.01ns     2.1 GB/sec           5.08     95.2±0.10ns   420.8 MB/sec     3.44     64.5±0.08ns   620.9 MB/sec
oneshot/teeny-zh/never-john-watson                                   1.69     29.5±0.48ns  1003.7 MB/sec      1.00     17.4±0.02ns  1697.4 MB/sec           2.24     39.0±0.08ns   758.6 MB/sec     3.08     53.6±0.10ns   551.5 MB/sec
oneshot/teeny-zh/rare-sherlock                                       1.37     19.5±0.02ns  1518.4 MB/sec      1.00     14.2±0.01ns     2.0 GB/sec           3.07     43.8±0.14ns   675.0 MB/sec     2.79     39.8±0.12ns   743.5 MB/sec
oneshot/teeny-zh/rare-sherlock-holmes                                1.39     31.7±0.03ns   933.8 MB/sec      1.00     22.7±0.05ns  1301.2 MB/sec           3.50     79.5±0.14ns   371.8 MB/sec     2.76     62.6±0.18ns   472.2 MB/sec
oneshotiter/code-rust-library/common-fn                              1.00    103.6±0.17µs    14.8 GB/sec                                                    10.41  1078.8±2.97µs  1457.0 MB/sec     2.24    232.0±0.27µs     6.6 GB/sec
oneshotiter/code-rust-library/common-fn-is-empty                     1.00     56.6±0.09µs    27.1 GB/sec                                                    20.40  1155.3±3.01µs  1360.5 MB/sec     3.00    169.9±0.35µs     9.0 GB/sec
oneshotiter/code-rust-library/common-let                             1.00    147.4±0.35µs    10.4 GB/sec                                                    8.92   1314.7±1.95µs  1195.6 MB/sec     2.06    303.2±0.17µs     5.1 GB/sec
oneshotiter/code-rust-library/common-paren                           1.09    390.8±0.58µs     3.9 GB/sec                                                    3.57   1285.6±0.67µs  1222.5 MB/sec     1.00    359.8±0.28µs     4.3 GB/sec
oneshotiter/huge-en/common-one-space                                 1.16    580.5±1.26µs  1007.6 MB/sec                                                    2.19   1097.4±0.65µs   533.0 MB/sec     1.00    501.1±0.72µs  1167.4 MB/sec
oneshotiter/huge-en/common-that                                      1.00     49.5±0.17µs    11.5 GB/sec                                                    6.80    336.7±1.19µs  1737.4 MB/sec     2.10    104.0±0.06µs     5.5 GB/sec
oneshotiter/huge-en/common-you                                       1.00    100.4±0.19µs     5.7 GB/sec                                                    4.16    417.6±1.01µs  1400.8 MB/sec     1.60    160.2±0.09µs     3.6 GB/sec
oneshotiter/huge-ru/common-not                                       1.00     71.0±0.12µs     8.0 GB/sec                                                    9.05    642.3±1.60µs   910.7 MB/sec     4.19    297.2±0.17µs  1968.6 MB/sec
oneshotiter/huge-ru/common-one-space                                 1.14    308.9±0.46µs  1893.9 MB/sec                                                    2.54    691.3±0.30µs   846.2 MB/sec     1.00    272.1±0.56µs     2.1 GB/sec
oneshotiter/huge-ru/common-that                                      1.00     36.5±0.04µs    15.6 GB/sec                                                    16.17   591.0±1.33µs   989.9 MB/sec     3.89    142.1±0.06µs     4.0 GB/sec
oneshotiter/huge-zh/common-do-not                                    1.00     67.2±0.19µs     8.5 GB/sec                                                    4.34    291.9±0.27µs  2004.4 MB/sec     2.27    152.8±0.09µs     3.7 GB/sec
oneshotiter/huge-zh/common-one-space                                 1.10    166.7±0.27µs     3.4 GB/sec                                                    3.48    527.9±0.48µs  1108.2 MB/sec     1.00    151.7±0.22µs     3.8 GB/sec
oneshotiter/huge-zh/common-that                                      1.00     36.0±0.06µs    15.8 GB/sec                                                    5.28    190.2±0.15µs     3.0 GB/sec     2.38     85.6±0.07µs     6.7 GB/sec
oneshotiter/pathological-md5-huge/common-two-bytes                   1.00     12.0±0.04µs    11.7 GB/sec                                                    12.41   148.9±0.07µs   969.2 MB/sec     2.40     28.8±0.03µs     4.9 GB/sec
oneshotiter/pathological-repeated-rare-huge/common-match             1.29    506.4±0.94µs   941.9 MB/sec                                                    1.00    392.2±1.08µs  1216.2 MB/sec     5.21      2.0±0.00ms   233.4 MB/sec
oneshotiter/pathological-repeated-rare-small/common-match            1.22   1013.2±1.06ns   942.2 MB/sec                                                    1.00    829.5±0.59ns  1150.8 MB/sec     4.93      4.1±0.00µs   233.3 MB/sec
prebuilt/code-rust-library/never-fn-quux                             1.00     51.4±0.07µs    29.9 GB/sec      1.03     52.7±0.37µs    29.1 GB/sec
prebuilt/code-rust-library/never-fn-strength                         1.09     52.7±0.05µs    29.1 GB/sec      1.00     48.5±0.10µs    31.7 GB/sec
prebuilt/code-rust-library/never-fn-strength-paren                   1.04     52.8±0.12µs    29.1 GB/sec      1.00     50.6±0.37µs    30.3 GB/sec
prebuilt/code-rust-library/rare-fn-from-str                          1.08     15.2±0.04µs   100.7 GB/sec      1.00     14.1±0.02µs   108.7 GB/sec
prebuilt/huge-en/never-all-common-bytes                              1.13     26.3±0.05µs    21.7 GB/sec      1.00     23.2±0.04µs    24.7 GB/sec
prebuilt/huge-en/never-john-watson                                   1.00     16.5±0.02µs    34.6 GB/sec      1.07     17.7±0.03µs    32.3 GB/sec
prebuilt/huge-en/never-some-rare-bytes                               1.00     16.6±0.02µs    34.3 GB/sec      1.06     17.6±0.03µs    32.4 GB/sec
prebuilt/huge-en/never-two-space                                     1.11     19.3±0.03µs    29.6 GB/sec      1.00     17.4±0.02µs    32.8 GB/sec
prebuilt/huge-en/rare-huge-needle                                    1.25     38.2±0.06µs    14.9 GB/sec      1.00     30.6±0.06µs    18.6 GB/sec
prebuilt/huge-en/rare-long-needle                                    1.00     16.0±0.02µs    35.8 GB/sec      2.03     32.4±0.08µs    17.7 GB/sec
prebuilt/huge-en/rare-medium-needle                                  1.00     19.9±0.02µs    28.7 GB/sec      2.60     51.8±0.06µs    11.0 GB/sec
prebuilt/huge-en/rare-sherlock                                       1.00     16.6±0.03µs    34.4 GB/sec      1.07     17.8±0.03µs    32.2 GB/sec
prebuilt/huge-en/rare-sherlock-holmes                                1.05     19.1±0.03µs    29.9 GB/sec      1.00     18.2±0.02µs    31.5 GB/sec
prebuilt/huge-ru/never-john-watson                                   1.00     16.5±0.04µs    34.6 GB/sec      5.08     83.8±0.13µs     6.8 GB/sec
prebuilt/huge-ru/rare-sherlock                                       1.00     16.7±0.02µs    34.3 GB/sec      1.97     32.8±0.07µs    17.4 GB/sec
prebuilt/huge-ru/rare-sherlock-holmes                                1.00     16.5±0.03µs    34.6 GB/sec      4.36     72.0±0.10µs     7.9 GB/sec
prebuilt/huge-zh/never-john-watson                                   1.05     19.5±0.02µs    29.3 GB/sec      1.00     18.6±0.08µs    30.8 GB/sec
prebuilt/huge-zh/rare-sherlock                                       1.00     16.9±0.05µs    33.9 GB/sec      1.33     22.4±0.02µs    25.5 GB/sec
prebuilt/huge-zh/rare-sherlock-holmes                                1.00     19.6±0.04µs    29.1 GB/sec      1.82     35.7±0.04µs    16.0 GB/sec
prebuilt/pathological-defeat-simple-vector-freq/rare-alphabet        1.00     71.6±0.05µs     9.4 GB/sec      5.92    423.8±0.51µs  1620.4 MB/sec
prebuilt/pathological-defeat-simple-vector-repeated/rare-alphabet    1.00   1235.2±2.04µs   556.0 MB/sec      1.99      2.5±0.00ms   279.4 MB/sec
prebuilt/pathological-defeat-simple-vector/rare-alphabet             1.68    201.6±0.44µs     2.5 GB/sec      1.00    120.3±0.18µs     4.3 GB/sec
prebuilt/pathological-md5-huge/never-no-hash                         1.00      6.0±0.01µs    23.6 GB/sec      1.82     10.9±0.05µs    13.0 GB/sec
prebuilt/pathological-md5-huge/rare-last-hash                        1.00      6.0±0.02µs    23.4 GB/sec      1.66     10.0±0.04µs    14.1 GB/sec
prebuilt/pathological-repeated-rare-huge/never-tricky                1.10     15.6±0.04µs    29.8 GB/sec      1.00     14.2±0.02µs    32.7 GB/sec
prebuilt/pathological-repeated-rare-small/never-tricky               1.20     35.0±0.13ns    26.6 GB/sec      1.00     29.3±0.02ns    31.8 GB/sec
prebuilt/sliceslice-haystack/words                                   1.06    180.7±0.20µs        0 B/sec      1.00    171.2±0.21µs        0 B/sec           9.44   1615.2±5.96µs        0 B/sec
prebuilt/sliceslice-i386/words                                       1.00     25.0±0.02ms        0 B/sec      1.05     26.2±0.02ms        0 B/sec           13.17   329.8±0.55ms        0 B/sec
prebuilt/sliceslice-words/words                                      1.00     80.3±0.02ms        0 B/sec      1.04     83.6±0.09ms        0 B/sec           3.09    248.5±0.64ms        0 B/sec
prebuilt/teeny-en/never-all-common-bytes                             1.56      8.8±0.01ns     2.9 GB/sec      1.00      5.7±0.00ns     4.6 GB/sec
prebuilt/teeny-en/never-john-watson                                  1.44      8.8±0.01ns     2.9 GB/sec      1.00      6.1±0.00ns     4.2 GB/sec
prebuilt/teeny-en/never-some-rare-bytes                              1.44      8.8±0.01ns     2.9 GB/sec      1.00      6.1±0.00ns     4.2 GB/sec
prebuilt/teeny-en/never-two-space                                    1.44      8.8±0.00ns     2.9 GB/sec      1.00      6.1±0.00ns     4.2 GB/sec
prebuilt/teeny-en/rare-sherlock                                      1.89      9.8±0.00ns     2.7 GB/sec      1.00      5.2±0.00ns     5.1 GB/sec
prebuilt/teeny-en/rare-sherlock-holmes                               1.00     10.1±0.00ns     2.6 GB/sec      1.45     14.6±0.03ns  1829.7 MB/sec
prebuilt/teeny-ru/never-john-watson                                  1.14      7.8±0.02ns     5.0 GB/sec      1.00      6.9±0.00ns     5.7 GB/sec
prebuilt/teeny-ru/rare-sherlock                                      1.48      9.8±0.03ns     4.0 GB/sec      1.00      6.6±0.00ns     5.9 GB/sec
prebuilt/teeny-ru/rare-sherlock-holmes                               1.36     12.4±0.05ns     3.2 GB/sec      1.00      9.1±0.01ns     4.3 GB/sec
prebuilt/teeny-zh/never-john-watson                                  1.44      8.9±0.00ns     3.3 GB/sec      1.00      6.1±0.00ns     4.7 GB/sec
prebuilt/teeny-zh/rare-sherlock                                      2.12      9.9±0.01ns     2.9 GB/sec      1.00      4.7±0.00ns     6.2 GB/sec
prebuilt/teeny-zh/rare-sherlock-holmes                               1.17     19.2±0.03ns  1539.6 MB/sec      1.00     16.4±0.01ns  1797.2 MB/sec
prebuiltiter/code-rust-library/common-fn                             1.00    104.7±0.15µs    14.7 GB/sec
prebuiltiter/code-rust-library/common-fn-is-empty                    1.00     56.6±0.11µs    27.1 GB/sec
prebuiltiter/code-rust-library/common-let                            1.00    148.6±0.14µs    10.3 GB/sec
prebuiltiter/code-rust-library/common-paren                          1.00    396.7±0.61µs     3.9 GB/sec
prebuiltiter/huge-en/common-one-space                                1.00    579.0±1.29µs  1010.3 MB/sec
prebuiltiter/huge-en/common-that                                     1.00     49.8±0.10µs    11.5 GB/sec
prebuiltiter/huge-en/common-you                                      1.00    100.1±0.16µs     5.7 GB/sec
prebuiltiter/huge-ru/common-not                                      1.00     70.7±0.10µs     8.1 GB/sec
prebuiltiter/huge-ru/common-one-space                                1.00    308.9±0.37µs  1893.9 MB/sec
prebuiltiter/huge-ru/common-that                                     1.00     36.2±0.05µs    15.8 GB/sec
prebuiltiter/huge-zh/common-do-not                                   1.00     67.3±0.10µs     8.5 GB/sec
prebuiltiter/huge-zh/common-one-space                                1.00    169.8±0.20µs     3.4 GB/sec
prebuiltiter/huge-zh/common-that                                     1.00     36.1±0.07µs    15.8 GB/sec
prebuiltiter/pathological-md5-huge/common-two-bytes                  1.00     11.9±0.04µs    11.9 GB/sec
prebuiltiter/pathological-repeated-rare-huge/common-match            1.00    507.6±0.75µs   939.5 MB/sec
prebuiltiter/pathological-repeated-rare-small/common-match           1.00    999.1±2.86ns   955.5 MB/sec
```
