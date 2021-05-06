# Benchmark environment

* Commit 3d405e85a277198d99149a48ee50467da3f71e9e
* Intel i5-7600 @ 3.5 GHz
* Linux 5.11.8
* `criterion 0.3.3`
* `critcmp 0.1.7`

These benchmarks were run after the `sliceslice 0.3.0` release. The main change
in this release was to make constructing the searcher generic over the needle,
which means it no longer requires a `Box<[u8]>`. This improves the `oneshot`
benchmarks for `sliceslice` quite a bit.

# memchr benchmarks

```
group               2021-05-06/memchr1/fallback/           2021-05-06/memchr1/krate/              2021-05-06/memchr1/libc/               2021-05-06/memchr1/naive/
-----               ----------------------------           -------------------------              ------------------------               -------------------------
empty/never         9.92      2.4±0.02ns        0 B/sec    1.00      0.2±0.00ns        0 B/sec    7.97      2.0±0.00ns        0 B/sec    2.09      0.5±0.02ns        0 B/sec
huge/common         2.92    642.1±1.15µs   883.6 MB/sec    1.00    219.6±0.25µs     2.5 GB/sec    1.28    280.7±0.23µs  2021.4 MB/sec    2.11    462.5±2.66µs  1226.7 MB/sec
huge/never          3.80     32.0±0.04µs    17.3 GB/sec    1.07      9.0±0.01µs    61.4 GB/sec    1.00      8.4±0.01µs    65.7 GB/sec    17.27   145.5±0.20µs     3.8 GB/sec
huge/rare           3.46     32.6±0.04µs    17.0 GB/sec    1.00      9.4±0.01µs    58.8 GB/sec    1.01      9.5±0.01µs    58.4 GB/sec    15.62   147.1±0.17µs     3.8 GB/sec
huge/uncommon       2.09    151.9±0.26µs     3.6 GB/sec    1.05     76.5±0.22µs     7.2 GB/sec    1.00     72.8±0.18µs     7.6 GB/sec    2.76    200.9±0.39µs     2.8 GB/sec
huge/verycommon     2.46   1130.4±5.16µs   501.9 MB/sec    1.00    459.2±0.72µs  1235.6 MB/sec    1.28    589.1±0.69µs   963.1 MB/sec    2.08    953.0±5.80µs   595.4 MB/sec
small/common        1.41    289.4±0.31ns     2.1 GB/sec    1.00    204.9±0.38ns     3.0 GB/sec    1.07    219.5±0.22ns     2.8 GB/sec    1.18    240.9±2.64ns     2.6 GB/sec
small/never         5.30     39.3±0.04ns    15.7 GB/sec    1.01      7.5±0.01ns    82.5 GB/sec    1.00      7.4±0.03ns    83.4 GB/sec    23.23   172.3±0.38ns     3.6 GB/sec
small/rare          4.40     49.5±0.29ns    12.5 GB/sec    1.08     12.1±0.01ns    51.0 GB/sec    1.00     11.2±0.02ns    55.0 GB/sec    16.02   180.1±0.26ns     3.4 GB/sec
small/uncommon      1.81     83.4±0.18ns     7.4 GB/sec    1.00     46.1±0.05ns    13.4 GB/sec    1.03     47.3±0.05ns    13.1 GB/sec    4.15    191.0±0.42ns     3.2 GB/sec
small/verycommon    1.86    555.4±0.89ns  1140.1 MB/sec    1.66    495.8±0.77ns  1277.2 MB/sec    1.80    536.5±0.69ns  1180.2 MB/sec    1.00    297.8±3.05ns     2.1 GB/sec
tiny/common         1.82     55.3±0.11ns  1190.0 MB/sec    1.71     51.9±0.07ns  1268.3 MB/sec    1.63     49.4±0.07ns  1332.2 MB/sec    1.00     30.3±0.28ns     2.1 GB/sec
tiny/never          1.79      6.1±0.01ns    10.5 GB/sec    1.08      3.7±0.01ns    17.4 GB/sec    1.00      3.4±0.00ns    18.8 GB/sec    7.78     26.6±0.03ns     2.4 GB/sec
tiny/rare           1.52      9.3±0.02ns     6.9 GB/sec    1.04      6.3±0.02ns    10.1 GB/sec    1.00      6.1±0.02ns    10.5 GB/sec    4.37     26.8±0.03ns     2.4 GB/sec
tiny/uncommon       2.10     37.8±0.08ns  1741.3 MB/sec    1.00     18.0±0.02ns     3.6 GB/sec    1.04     18.7±0.03ns     3.4 GB/sec    1.24     22.3±0.19ns     2.9 GB/sec
```

# memmem benchmarks

```
group                                                                2021-05-06/memmem/krate/               2021-05-06/memmem/sliceslice/          2021-05-06/memmem/stud/                2021-05-06/memmem/twoway/
-----                                                                ------------------------               -----------------------------          -----------------------                -------------------------
oneshot/code-rust-library/never-fn-quux                              1.00     46.7±0.09µs    32.8 GB/sec    1.11     52.1±0.05µs    29.5 GB/sec    26.10  1219.6±3.42µs  1288.8 MB/sec    3.41    159.5±0.20µs     9.6 GB/sec
oneshot/code-rust-library/never-fn-strength                          1.01     48.8±0.05µs    31.4 GB/sec    1.00     48.5±0.04µs    31.6 GB/sec    28.21  1368.5±3.03µs  1148.6 MB/sec    4.64    225.1±0.36µs     6.8 GB/sec
oneshot/code-rust-library/never-fn-strength-paren                    1.00     48.3±0.17µs    31.8 GB/sec    1.04     50.2±0.06µs    30.6 GB/sec    26.30  1269.8±3.36µs  1237.8 MB/sec    4.66    225.1±0.35µs     6.8 GB/sec
oneshot/code-rust-library/rare-fn-from-str                           1.00     14.0±0.01µs   110.0 GB/sec    1.00     14.0±0.02µs   109.7 GB/sec    24.28   338.9±0.55µs     4.5 GB/sec    9.77    136.4±0.29µs    11.3 GB/sec
oneshot/huge-en/never-all-common-bytes                               1.13     25.9±0.06µs    22.1 GB/sec    1.00     23.0±0.02µs    24.8 GB/sec    11.61   267.1±0.72µs     2.1 GB/sec    4.20     96.7±0.16µs     5.9 GB/sec
oneshot/huge-en/never-john-watson                                    1.00     17.5±0.02µs    32.7 GB/sec    1.02     17.8±0.01µs    32.0 GB/sec    25.94   453.2±1.00µs  1290.6 MB/sec    4.86     84.9±0.08µs     6.7 GB/sec
oneshot/huge-en/never-some-rare-bytes                                1.00     17.5±0.02µs    32.6 GB/sec    1.00     17.5±0.02µs    32.6 GB/sec    13.61   238.2±0.38µs     2.4 GB/sec    3.78     66.2±0.07µs     8.6 GB/sec
oneshot/huge-en/never-two-space                                      1.02     17.5±0.03µs    32.5 GB/sec    1.00     17.3±0.02µs    33.0 GB/sec    36.59   632.6±0.63µs   924.7 MB/sec    7.45    128.9±0.34µs     4.4 GB/sec
oneshot/huge-en/rare-huge-needle                                     1.30     39.8±0.09µs    14.3 GB/sec    1.00     30.7±0.06µs    18.6 GB/sec    3.84    117.9±0.21µs     4.8 GB/sec    2.55     78.2±0.10µs     7.3 GB/sec
oneshot/huge-en/rare-long-needle                                     1.00     16.6±0.02µs    34.5 GB/sec    1.94     32.1±0.04µs    17.8 GB/sec    7.59    125.7±0.24µs     4.5 GB/sec    4.73     78.4±0.10µs     7.3 GB/sec
oneshot/huge-en/rare-medium-needle                                   1.00     18.6±0.02µs    30.7 GB/sec    2.75     51.1±0.08µs    11.2 GB/sec    11.07   206.0±0.24µs     2.8 GB/sec    4.54     84.5±0.13µs     6.8 GB/sec
oneshot/huge-en/rare-sherlock                                        1.00     18.0±0.03µs    31.7 GB/sec    1.00     18.0±0.04µs    31.7 GB/sec    16.16   291.5±0.51µs  2006.8 MB/sec    4.23     76.2±0.18µs     7.5 GB/sec
oneshot/huge-en/rare-sherlock-holmes                                 1.00     17.4±0.02µs    32.9 GB/sec    1.06     18.4±0.03µs    31.0 GB/sec    17.83   309.5±0.48µs  1889.7 MB/sec    13.47   233.8±0.26µs     2.4 GB/sec
oneshot/huge-ru/never-john-watson                                    1.00     17.5±0.02µs    32.7 GB/sec    4.67     81.5±0.11µs     7.0 GB/sec    18.86   329.3±0.58µs  1776.5 MB/sec    8.77    153.2±0.14µs     3.7 GB/sec
oneshot/huge-ru/rare-sherlock                                        1.00     17.5±0.02µs    32.7 GB/sec    1.86     32.6±0.03µs    17.5 GB/sec    23.21   406.0±0.85µs  1440.9 MB/sec    4.22     73.8±0.10µs     7.7 GB/sec
oneshot/huge-ru/rare-sherlock-holmes                                 1.00     17.5±0.02µs    32.7 GB/sec    4.11     71.9±0.09µs     7.9 GB/sec    16.36   286.1±0.59µs  2044.6 MB/sec    13.05   228.2±0.31µs     2.5 GB/sec
oneshot/huge-zh/never-john-watson                                    1.00     17.8±0.02µs    32.1 GB/sec    1.05     18.6±0.02µs    30.7 GB/sec    6.94    123.6±0.18µs     4.6 GB/sec    3.35     59.6±0.05µs     9.6 GB/sec
oneshot/huge-zh/rare-sherlock                                        1.00     17.9±0.02µs    31.9 GB/sec    1.24     22.2±0.02µs    25.8 GB/sec    11.65   208.4±0.42µs     2.7 GB/sec    3.68     65.9±0.09µs     8.7 GB/sec
oneshot/huge-zh/rare-sherlock-holmes                                 1.00     18.1±0.06µs    31.6 GB/sec    1.95     35.2±0.05µs    16.2 GB/sec    8.56    154.5±0.13µs     3.7 GB/sec    3.87     69.9±0.06µs     8.2 GB/sec
oneshot/pathological-defeat-simple-vector-freq/rare-alphabet         1.55     68.7±0.10µs     9.8 GB/sec    9.22    409.3±0.44µs  1677.9 MB/sec    1.00     44.4±0.06µs    15.1 GB/sec    2.14     95.1±0.07µs     7.1 GB/sec
oneshot/pathological-defeat-simple-vector-repeated/rare-alphabet     18.47  1228.0±1.91µs   559.2 MB/sec    41.25     2.7±0.00ms   250.4 MB/sec    18.47  1227.6±2.49µs   559.4 MB/sec    1.00     66.5±0.08µs    10.1 GB/sec
oneshot/pathological-defeat-simple-vector/rare-alphabet              2.26    208.8±0.60µs     2.5 GB/sec    1.79    164.7±0.17µs     3.1 GB/sec    1.13    104.5±0.13µs     4.9 GB/sec    1.00     92.2±0.08µs     5.6 GB/sec
oneshot/pathological-md5-huge/never-no-hash                          1.00      5.6±0.02µs    25.2 GB/sec    1.97     11.0±0.01µs    12.8 GB/sec    5.24     29.3±0.07µs     4.8 GB/sec    3.60     20.1±0.05µs     7.0 GB/sec
oneshot/pathological-md5-huge/rare-last-hash                         1.00      6.0±0.01µs    23.4 GB/sec    1.82     11.0±0.02µs    12.9 GB/sec    4.82     28.9±0.04µs     4.9 GB/sec    3.34     20.1±0.02µs     7.0 GB/sec
oneshot/pathological-repeated-rare-huge/never-tricky                 1.01     14.2±0.03µs    32.7 GB/sec    1.00     14.0±0.02µs    33.2 GB/sec    39.24   551.2±0.81µs   865.2 MB/sec    19.07   267.9±0.28µs  1780.5 MB/sec
oneshot/pathological-repeated-rare-small/never-tricky                2.03     65.4±0.14ns    14.3 GB/sec    1.00     32.2±0.49ns    28.9 GB/sec    35.10  1130.8±1.70ns   844.2 MB/sec    17.67   569.2±0.68ns  1677.0 MB/sec
oneshot/teeny-en/never-all-common-bytes                              1.97     24.1±0.03ns  1110.0 MB/sec    1.00     12.2±0.01ns     2.1 GB/sec    2.82     34.5±0.03ns   774.0 MB/sec    3.61     44.2±0.04ns   604.8 MB/sec
oneshot/teeny-en/never-john-watson                                   1.45     24.5±0.05ns  1090.5 MB/sec    1.00     16.9±0.03ns  1581.8 MB/sec    2.25     37.9±0.03ns   704.3 MB/sec    2.81     47.4±0.05ns   563.6 MB/sec
oneshot/teeny-en/never-some-rare-bytes                               2.02     24.3±0.02ns  1101.0 MB/sec    1.00     12.0±0.01ns     2.2 GB/sec    2.22     26.6±0.03ns  1003.8 MB/sec    2.28     27.4±0.03ns   975.5 MB/sec
oneshot/teeny-en/never-two-space                                     2.21     23.8±0.02ns  1122.8 MB/sec    1.00     10.8±0.01ns     2.4 GB/sec    3.10     33.4±0.02ns   799.5 MB/sec    2.54     27.4±0.03ns   975.1 MB/sec
oneshot/teeny-en/rare-sherlock                                       1.77     19.9±0.02ns  1340.2 MB/sec    1.00     11.3±0.01ns     2.3 GB/sec    3.60     40.6±0.03ns   657.7 MB/sec    3.27     36.8±0.04ns   724.7 MB/sec
oneshot/teeny-en/rare-sherlock-holmes                                1.00     26.0±0.06ns  1027.1 MB/sec    1.14     29.5±0.10ns   904.7 MB/sec    2.82     73.4±0.04ns   364.0 MB/sec    2.01     52.3±0.05ns   510.6 MB/sec
oneshot/teeny-ru/never-john-watson                                   2.32     33.1±0.05ns  1211.9 MB/sec    1.00     14.2±0.01ns     2.7 GB/sec    4.65     66.1±0.09ns   606.1 MB/sec    4.82     68.5±0.08ns   584.5 MB/sec
oneshot/teeny-ru/rare-sherlock                                       1.94     27.4±0.07ns  1464.3 MB/sec    1.00     14.1±0.02ns     2.8 GB/sec    4.03     56.8±0.11ns   705.6 MB/sec    3.21     45.1±0.04ns   887.2 MB/sec
oneshot/teeny-ru/rare-sherlock-holmes                                2.05     35.0±0.08ns  1144.1 MB/sec    1.00     17.0±0.02ns     2.3 GB/sec    6.39    108.8±0.07ns   368.0 MB/sec    3.73     63.5±0.09ns   630.3 MB/sec
oneshot/teeny-zh/never-john-watson                                   1.52     26.8±0.07ns  1104.1 MB/sec    1.00     17.6±0.02ns  1678.9 MB/sec    2.60     45.8±0.05ns   645.1 MB/sec    3.02     53.2±0.09ns   555.8 MB/sec
oneshot/teeny-zh/rare-sherlock                                       1.66     18.7±0.04ns  1577.9 MB/sec    1.00     11.3±0.01ns     2.6 GB/sec    4.45     50.2±0.05ns   589.4 MB/sec    3.49     39.3±0.07ns   751.9 MB/sec
oneshot/teeny-zh/rare-sherlock-holmes                                1.26     28.9±0.04ns  1023.9 MB/sec    1.00     22.9±0.03ns  1290.8 MB/sec    4.00     91.6±0.09ns   322.8 MB/sec    2.80     64.2±0.13ns   460.5 MB/sec
oneshotiter/code-rust-library/common-fn                              1.00     99.2±0.13µs    15.5 GB/sec                                           11.24  1114.9±1.09µs  1409.8 MB/sec    2.53    251.2±0.40µs     6.1 GB/sec
oneshotiter/code-rust-library/common-fn-is-empty                     1.00     52.5±0.11µs    29.2 GB/sec                                           25.83  1356.2±1.86µs  1158.9 MB/sec    3.25    170.6±0.22µs     9.0 GB/sec
oneshotiter/code-rust-library/common-let                             1.00    146.3±0.20µs    10.5 GB/sec                                           9.94   1454.3±1.91µs  1080.8 MB/sec    2.19    319.9±0.41µs     4.8 GB/sec
oneshotiter/code-rust-library/common-paren                           1.13    401.9±0.40µs     3.8 GB/sec                                           4.17   1487.5±2.70µs  1056.6 MB/sec    1.00    356.9±0.51µs     4.3 GB/sec
oneshotiter/huge-en/common-one-space                                 1.14    571.1±0.71µs  1024.2 MB/sec                                           2.74  1371.5±48.14µs   426.5 MB/sec    1.00    500.0±0.47µs  1169.9 MB/sec
oneshotiter/huge-en/common-that                                      1.00     40.9±0.05µs    14.0 GB/sec                                           9.74    398.2±0.59µs  1469.1 MB/sec    2.61    106.6±0.08µs     5.4 GB/sec
oneshotiter/huge-en/common-you                                       1.00    100.2±0.16µs     5.7 GB/sec                                           4.63    464.1±0.88µs  1260.4 MB/sec    1.61    161.3±0.18µs     3.5 GB/sec
oneshotiter/huge-ru/common-not                                       1.00     69.6±0.19µs     8.2 GB/sec                                           10.83   754.5±0.83µs   775.4 MB/sec    4.41    306.9±0.32µs  1906.0 MB/sec
oneshotiter/huge-ru/common-one-space                                 1.13    308.0±0.38µs  1899.0 MB/sec                                           3.08    837.6±5.44µs   698.4 MB/sec    1.00    271.8±0.34µs     2.1 GB/sec
oneshotiter/huge-ru/common-that                                      1.00     35.4±0.04µs    16.1 GB/sec                                           20.74   734.6±0.58µs   796.3 MB/sec    4.11    145.5±0.12µs     3.9 GB/sec
oneshotiter/huge-zh/common-do-not                                    1.00     66.7±0.08µs     8.6 GB/sec                                           4.75    316.5±0.28µs  1848.2 MB/sec    2.34    156.1±0.34µs     3.7 GB/sec
oneshotiter/huge-zh/common-one-space                                 1.13    171.5±0.19µs     3.3 GB/sec                                           4.10    623.3±0.56µs   938.5 MB/sec    1.00    152.2±0.24µs     3.8 GB/sec
oneshotiter/huge-zh/common-that                                      1.00     34.7±0.07µs    16.5 GB/sec                                           5.40    187.2±0.25µs     3.1 GB/sec    2.62     90.8±0.14µs     6.3 GB/sec
oneshotiter/pathological-md5-huge/common-two-bytes                   1.00     11.4±0.05µs    12.3 GB/sec                                           13.85   158.4±0.28µs   911.2 MB/sec    2.48     28.4±0.03µs     5.0 GB/sec
oneshotiter/pathological-repeated-rare-huge/common-match             1.16    497.0±0.53µs   959.7 MB/sec                                           1.00    428.6±1.32µs  1112.7 MB/sec    4.64   1989.2±1.79µs   239.8 MB/sec
oneshotiter/pathological-repeated-rare-small/common-match            1.10    992.6±1.20ns   961.7 MB/sec                                           1.00    905.7±0.81ns  1054.0 MB/sec    4.40      4.0±0.01µs   239.5 MB/sec
prebuilt/code-rust-library/never-fn-quux                             1.00     47.3±0.08µs    32.5 GB/sec    1.11     52.4±0.08µs    29.3 GB/sec                                         
prebuilt/code-rust-library/never-fn-strength                         1.00     48.8±0.06µs    31.5 GB/sec    1.00     48.9±0.07µs    31.4 GB/sec                                         
prebuilt/code-rust-library/never-fn-strength-paren                   1.00     48.2±0.04µs    31.8 GB/sec    1.05     50.6±0.07µs    30.3 GB/sec                                         
prebuilt/code-rust-library/rare-fn-from-str                          1.00     13.8±0.02µs   111.4 GB/sec    1.02     14.1±0.02µs   109.1 GB/sec                                         
prebuilt/huge-en/never-all-common-bytes                              1.11     25.6±0.06µs    22.3 GB/sec    1.00     23.1±0.03µs    24.7 GB/sec                                         
prebuilt/huge-en/never-john-watson                                   1.00     17.4±0.01µs    32.8 GB/sec    1.02     17.8±0.02µs    32.0 GB/sec                                         
prebuilt/huge-en/never-some-rare-bytes                               1.00     17.4±0.02µs    32.7 GB/sec    1.01     17.7±0.03µs    32.3 GB/sec                                         
prebuilt/huge-en/never-two-space                                     1.00     17.3±0.01µs    33.0 GB/sec    1.01     17.5±0.06µs    32.7 GB/sec                                         
prebuilt/huge-en/rare-huge-needle                                    1.23     37.8±0.10µs    15.1 GB/sec    1.00     30.8±0.03µs    18.6 GB/sec                                         
prebuilt/huge-en/rare-long-needle                                    1.00     15.8±0.01µs    36.2 GB/sec    2.04     32.2±0.05µs    17.7 GB/sec                                         
prebuilt/huge-en/rare-medium-needle                                  1.00     18.3±0.04µs    31.1 GB/sec    2.73     50.0±0.06µs    11.4 GB/sec                                         
prebuilt/huge-en/rare-sherlock                                       1.02     17.9±0.05µs    32.0 GB/sec    1.00     17.6±0.02µs    32.5 GB/sec                                         
prebuilt/huge-en/rare-sherlock-holmes                                1.00     17.3±0.02µs    33.0 GB/sec    1.05     18.2±0.02µs    31.5 GB/sec                                         
prebuilt/huge-ru/never-john-watson                                   1.00     17.6±0.02µs    32.4 GB/sec    4.69     82.8±0.13µs     6.9 GB/sec                                         
prebuilt/huge-ru/rare-sherlock                                       1.00     17.7±0.02µs    32.3 GB/sec    1.84     32.5±0.03µs    17.6 GB/sec                                         
prebuilt/huge-ru/rare-sherlock-holmes                                1.00     17.4±0.02µs    32.8 GB/sec    4.18     72.8±0.13µs     7.9 GB/sec                                         
prebuilt/huge-zh/never-john-watson                                   1.00     18.0±0.02µs    31.7 GB/sec    1.02     18.5±0.03µs    31.0 GB/sec                                         
prebuilt/huge-zh/rare-sherlock                                       1.00     17.8±0.02µs    32.0 GB/sec    1.25     22.3±0.03µs    25.6 GB/sec                                         
prebuilt/huge-zh/rare-sherlock-holmes                                1.00     18.2±0.02µs    31.4 GB/sec    1.94     35.3±0.06µs    16.2 GB/sec                                         
prebuilt/pathological-defeat-simple-vector-freq/rare-alphabet        1.00     68.3±0.11µs     9.8 GB/sec    5.52    376.8±0.34µs  1822.3 MB/sec                                         
prebuilt/pathological-defeat-simple-vector-repeated/rare-alphabet    1.00   1228.8±1.86µs   558.8 MB/sec    2.23      2.7±0.00ms   250.8 MB/sec                                         
prebuilt/pathological-defeat-simple-vector/rare-alphabet             1.26    207.9±0.18µs     2.5 GB/sec    1.00    164.8±0.29µs     3.1 GB/sec                                         
prebuilt/pathological-md5-huge/never-no-hash                         1.00      5.5±0.01µs    25.5 GB/sec    1.93     10.7±0.02µs    13.2 GB/sec                                         
prebuilt/pathological-md5-huge/rare-last-hash                        1.00      5.9±0.01µs    23.8 GB/sec    1.77     10.5±0.01µs    13.4 GB/sec                                         
prebuilt/pathological-repeated-rare-huge/never-tricky                1.00     14.2±0.01µs    32.8 GB/sec    1.00     14.2±0.02µs    32.9 GB/sec                                         
prebuilt/pathological-repeated-rare-small/never-tricky               1.25     35.9±0.11ns    25.9 GB/sec    1.00     28.6±0.05ns    32.6 GB/sec                                         
prebuilt/sliceslice-haystack/words                                   1.10    192.7±0.43µs        0 B/sec    1.00    175.1±0.45µs        0 B/sec    8.77   1536.7±3.64µs        0 B/sec  
prebuilt/sliceslice-i386/words                                       1.00     23.5±0.02ms        0 B/sec    1.14     26.8±0.03ms        0 B/sec    12.98   305.5±0.30ms        0 B/sec  
prebuilt/sliceslice-words/words                                      1.00     81.1±0.07ms        0 B/sec    1.07     87.1±0.08ms        0 B/sec    3.38    274.3±0.29ms        0 B/sec  
prebuilt/teeny-en/never-all-common-bytes                             1.51      9.0±0.01ns     2.9 GB/sec    1.00      6.0±0.01ns     4.3 GB/sec                                         
prebuilt/teeny-en/never-john-watson                                  1.42      9.0±0.01ns     2.9 GB/sec    1.00      6.4±0.01ns     4.1 GB/sec                                         
prebuilt/teeny-en/never-some-rare-bytes                              1.42      9.1±0.01ns     2.9 GB/sec    1.00      6.4±0.02ns     4.1 GB/sec                                         
prebuilt/teeny-en/never-two-space                                    1.38      8.8±0.01ns     3.0 GB/sec    1.00      6.4±0.01ns     4.1 GB/sec                                         
prebuilt/teeny-en/rare-sherlock                                      1.81      9.5±0.01ns     2.7 GB/sec    1.00      5.3±0.00ns     5.0 GB/sec                                         
prebuilt/teeny-en/rare-sherlock-holmes                               1.00     10.3±0.02ns     2.5 GB/sec    1.42     14.7±0.02ns  1819.3 MB/sec                                         
prebuilt/teeny-ru/never-john-watson                                  1.14      8.1±0.01ns     4.8 GB/sec    1.00      7.1±0.01ns     5.5 GB/sec                                         
prebuilt/teeny-ru/rare-sherlock                                      1.39      9.5±0.01ns     4.1 GB/sec    1.00      6.9±0.01ns     5.7 GB/sec                                         
prebuilt/teeny-ru/rare-sherlock-holmes                               1.34     12.5±0.01ns     3.1 GB/sec    1.00      9.3±0.02ns     4.2 GB/sec                                         
prebuilt/teeny-zh/never-john-watson                                  1.42      9.0±0.01ns     3.2 GB/sec    1.00      6.4±0.01ns     4.5 GB/sec                                         
prebuilt/teeny-zh/rare-sherlock                                      2.08     10.2±0.01ns     2.8 GB/sec    1.00      4.9±0.00ns     5.9 GB/sec                                         
prebuilt/teeny-zh/rare-sherlock-holmes                               1.25     19.5±0.03ns  1512.9 MB/sec    1.00     15.7±0.20ns  1886.9 MB/sec                                         
prebuiltiter/code-rust-library/common-fn                             1.00     98.5±0.11µs    15.6 GB/sec                                                                                
prebuiltiter/code-rust-library/common-fn-is-empty                    1.00     53.0±0.04µs    29.0 GB/sec                                                                                
prebuiltiter/code-rust-library/common-let                            1.00    145.0±0.24µs    10.6 GB/sec                                                                                
prebuiltiter/code-rust-library/common-paren                          1.00    397.5±0.61µs     3.9 GB/sec                                                                                
prebuiltiter/huge-en/common-one-space                                1.00    573.3±1.37µs  1020.2 MB/sec                                                                                
prebuiltiter/huge-en/common-that                                     1.00     40.7±0.03µs    14.0 GB/sec                                                                                
prebuiltiter/huge-en/common-you                                      1.00     97.7±0.12µs     5.8 GB/sec                                                                                
prebuiltiter/huge-ru/common-not                                      1.00     68.9±0.11µs     8.3 GB/sec                                                                                
prebuiltiter/huge-ru/common-one-space                                1.00    307.9±0.62µs  1899.8 MB/sec                                                                                
prebuiltiter/huge-ru/common-that                                     1.00     35.2±0.04µs    16.2 GB/sec                                                                                
prebuiltiter/huge-zh/common-do-not                                   1.00     65.1±0.09µs     8.8 GB/sec                                                                                
prebuiltiter/huge-zh/common-one-space                                1.00    169.6±0.28µs     3.4 GB/sec                                                                                
prebuiltiter/huge-zh/common-that                                     1.00     34.5±0.03µs    16.6 GB/sec                                                                                
prebuiltiter/pathological-md5-huge/common-two-bytes                  1.00     11.3±0.03µs    12.5 GB/sec                                                                                
prebuiltiter/pathological-repeated-rare-huge/common-match            1.00    499.0±0.48µs   955.8 MB/sec                                                                                
prebuiltiter/pathological-repeated-rare-small/common-match           1.00    972.4±1.77ns   981.7 MB/sec                                                                                
```
