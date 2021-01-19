use criterion::{Bencher, Criterion};

use crate::{
    define,
    memchr::{
        imp::{
            fallback1_count, fallback2_count, fallback3_count, memchr1_count,
            memchr2_count, memchr3_count, memrchr1_count, memrchr2_count,
            memrchr3_count, naive1_count, naive2_count, naive3_count,
        },
        inputs::{Input, Search1, Search2, Search3, EMPTY, HUGE, SMALL, TINY},
    },
};

#[path = "../../../src/memchr/c.rs"]
mod c;
#[allow(dead_code)]
#[path = "../../../src/memchr/fallback.rs"]
mod fallback;
mod imp;
mod inputs;
#[path = "../../../src/memchr/naive.rs"]
mod naive;

pub fn all(c: &mut Criterion) {
    define_memchr_input1(c, "memchr1/krate/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memchr1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_memchr_input1(c, "memchr1/krate/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memchr1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_memchr_input1(c, "memchr1/krate/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memchr1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_memchr_input1(c, "memchr1/krate/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memchr1_count(search.byte1.byte, search.corpus),
            );
        });
    });

    define_memchr_input1(c, "memchr1/libc/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                imp::memchr1_libc_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_memchr_input1(c, "memchr1/libc/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                imp::memchr1_libc_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_memchr_input1(c, "memchr1/libc/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                imp::memchr1_libc_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_memchr_input1(c, "memchr1/libc/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                imp::memchr1_libc_count(search.byte1.byte, search.corpus),
            );
        });
    });

    define_memchr_input1(
        c,
        "memchr1/fallback/huge",
        HUGE,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count,
                    fallback1_count(search.byte1.byte, search.corpus),
                );
            });
        },
    );
    define_memchr_input1(
        c,
        "memchr1/fallback/small",
        SMALL,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count,
                    fallback1_count(search.byte1.byte, search.corpus),
                );
            });
        },
    );
    define_memchr_input1(
        c,
        "memchr1/fallback/tiny",
        TINY,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count,
                    fallback1_count(search.byte1.byte, search.corpus),
                );
            });
        },
    );
    define_memchr_input1(
        c,
        "memchr1/fallback/empty",
        EMPTY,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count,
                    fallback1_count(search.byte1.byte, search.corpus),
                );
            });
        },
    );

    define_memchr_input1(c, "memchr1/naive/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                naive1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_memchr_input1(c, "memchr1/naive/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                naive1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_memchr_input1(c, "memchr1/naive/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                naive1_count(search.byte1.byte, search.corpus),
            );
        });
    });
    define_memchr_input1(c, "memchr1/naive/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                naive1_count(search.byte1.byte, search.corpus),
            );
        });
    });

    define_memchr_input2(c, "memchr2/krate/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                memchr2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_memchr_input2(c, "memchr2/krate/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                memchr2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_memchr_input2(c, "memchr2/krate/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                memchr2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_memchr_input2(c, "memchr2/krate/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                memchr2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });

    define_memchr_input2(
        c,
        "memchr2/fallback/huge",
        HUGE,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count + search.byte2.count,
                    fallback2_count(
                        search.byte1.byte,
                        search.byte2.byte,
                        search.corpus,
                    )
                );
            });
        },
    );
    define_memchr_input2(
        c,
        "memchr2/fallback/small",
        SMALL,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count + search.byte2.count,
                    fallback2_count(
                        search.byte1.byte,
                        search.byte2.byte,
                        search.corpus,
                    )
                );
            });
        },
    );
    define_memchr_input2(
        c,
        "memchr2/fallback/tiny",
        TINY,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count + search.byte2.count,
                    fallback2_count(
                        search.byte1.byte,
                        search.byte2.byte,
                        search.corpus,
                    )
                );
            });
        },
    );
    define_memchr_input2(
        c,
        "memchr2/fallback/empty",
        EMPTY,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count + search.byte2.count,
                    fallback2_count(
                        search.byte1.byte,
                        search.byte2.byte,
                        search.corpus,
                    )
                );
            });
        },
    );

    define_memchr_input2(c, "memchr2/naive/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                naive2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_memchr_input2(c, "memchr2/naive/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                naive2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_memchr_input2(c, "memchr2/naive/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                naive2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_memchr_input2(c, "memchr2/naive/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                naive2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });

    define_memchr_input3(c, "memchr3/krate/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                memchr3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_memchr_input3(c, "memchr3/krate/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                memchr3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_memchr_input3(c, "memchr3/krate/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                memchr3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_memchr_input3(c, "memchr3/krate/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                memchr3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });

    define_memchr_input3(
        c,
        "memchr3/fallback/huge",
        HUGE,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count
                        + search.byte2.count
                        + search.byte3.count,
                    fallback3_count(
                        search.byte1.byte,
                        search.byte2.byte,
                        search.byte3.byte,
                        search.corpus,
                    )
                );
            });
        },
    );
    define_memchr_input3(
        c,
        "memchr3/fallback/small",
        SMALL,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count
                        + search.byte2.count
                        + search.byte3.count,
                    fallback3_count(
                        search.byte1.byte,
                        search.byte2.byte,
                        search.byte3.byte,
                        search.corpus,
                    )
                );
            });
        },
    );
    define_memchr_input3(
        c,
        "memchr3/fallback/tiny",
        TINY,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count
                        + search.byte2.count
                        + search.byte3.count,
                    fallback3_count(
                        search.byte1.byte,
                        search.byte2.byte,
                        search.byte3.byte,
                        search.corpus,
                    )
                );
            });
        },
    );
    define_memchr_input3(
        c,
        "memchr3/fallback/empty",
        EMPTY,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count
                        + search.byte2.count
                        + search.byte3.count,
                    fallback3_count(
                        search.byte1.byte,
                        search.byte2.byte,
                        search.byte3.byte,
                        search.corpus,
                    )
                );
            });
        },
    );

    define_memchr_input3(c, "memchr3/naive/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                naive3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_memchr_input3(c, "memchr3/naive/small", SMALL, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                naive3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_memchr_input3(c, "memchr3/naive/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                naive3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_memchr_input3(c, "memchr3/naive/empty", EMPTY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                naive3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });

    define_memchr_input1(c, "memrchr1/krate/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memrchr1_count(search.byte1.byte, search.corpus)
            );
        });
    });
    define_memchr_input1(
        c,
        "memrchr1/krate/small",
        SMALL,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count,
                    memrchr1_count(search.byte1.byte, search.corpus)
                );
            });
        },
    );
    define_memchr_input1(c, "memrchr1/krate/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count,
                memrchr1_count(search.byte1.byte, search.corpus)
            );
        });
    });
    define_memchr_input1(
        c,
        "memrchr1/krate/empty",
        EMPTY,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count,
                    memrchr1_count(search.byte1.byte, search.corpus)
                );
            });
        },
    );

    #[cfg(all(target_os = "linux"))]
    {
        define_memchr_input1(
            c,
            "memrchr1/libc/huge",
            HUGE,
            move |search, b| {
                b.iter(|| {
                    assert_eq!(
                        search.byte1.count,
                        imp::memrchr1_libc_count(
                            search.byte1.byte,
                            search.corpus
                        )
                    );
                });
            },
        );
        define_memchr_input1(
            c,
            "memrchr1/libc/small",
            SMALL,
            move |search, b| {
                b.iter(|| {
                    assert_eq!(
                        search.byte1.count,
                        imp::memrchr1_libc_count(
                            search.byte1.byte,
                            search.corpus
                        )
                    );
                });
            },
        );
        define_memchr_input1(
            c,
            "memrchr1/libc/tiny",
            TINY,
            move |search, b| {
                b.iter(|| {
                    assert_eq!(
                        search.byte1.count,
                        imp::memrchr1_libc_count(
                            search.byte1.byte,
                            search.corpus
                        )
                    );
                });
            },
        );
        define_memchr_input1(
            c,
            "memrchr1/libc/empty",
            EMPTY,
            move |search, b| {
                b.iter(|| {
                    assert_eq!(
                        search.byte1.count,
                        imp::memrchr1_libc_count(
                            search.byte1.byte,
                            search.corpus
                        )
                    );
                });
            },
        );
    }

    define_memchr_input2(c, "memrchr2/krate/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                memrchr2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_memchr_input2(
        c,
        "memrchr2/krate/small",
        SMALL,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count + search.byte2.count,
                    memrchr2_count(
                        search.byte1.byte,
                        search.byte2.byte,
                        search.corpus,
                    )
                );
            });
        },
    );
    define_memchr_input2(c, "memrchr2/krate/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count,
                memrchr2_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.corpus,
                )
            );
        });
    });
    define_memchr_input2(
        c,
        "memrchr2/krate/empty",
        EMPTY,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count + search.byte2.count,
                    memrchr2_count(
                        search.byte1.byte,
                        search.byte2.byte,
                        search.corpus,
                    )
                );
            });
        },
    );

    define_memchr_input3(c, "memrchr3/krate/huge", HUGE, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                memrchr3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_memchr_input3(
        c,
        "memrchr3/krate/small",
        SMALL,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count
                        + search.byte2.count
                        + search.byte3.count,
                    memrchr3_count(
                        search.byte1.byte,
                        search.byte2.byte,
                        search.byte3.byte,
                        search.corpus,
                    )
                );
            });
        },
    );
    define_memchr_input3(c, "memrchr3/krate/tiny", TINY, move |search, b| {
        b.iter(|| {
            assert_eq!(
                search.byte1.count + search.byte2.count + search.byte3.count,
                memrchr3_count(
                    search.byte1.byte,
                    search.byte2.byte,
                    search.byte3.byte,
                    search.corpus,
                )
            );
        });
    });
    define_memchr_input3(
        c,
        "memrchr3/krate/empty",
        EMPTY,
        move |search, b| {
            b.iter(|| {
                assert_eq!(
                    search.byte1.count
                        + search.byte2.count
                        + search.byte3.count,
                    memrchr3_count(
                        search.byte1.byte,
                        search.byte2.byte,
                        search.byte3.byte,
                        search.corpus,
                    )
                );
            });
        },
    );
}

fn define_memchr_input1<'i>(
    c: &mut Criterion,
    group: &str,
    input: Input,
    bench: impl FnMut(Search1, &mut Bencher<'_>) + Clone + 'static,
) {
    macro_rules! def {
        ($name:expr, $kind:ident) => {
            if let Some(search) = input.$kind() {
                let corp = input.corpus;
                let name = format!("{}/{}", group, $name);
                let mut bench = bench.clone();
                define(c, &name, corp, Box::new(move |b| bench(search, b)));
            }
        };
    }
    def!("never", never1);
    def!("rare", rare1);
    def!("uncommon", uncommon1);
    def!("common", common1);
    def!("verycommon", verycommon1);
    def!("supercommon", supercommon1);
}

fn define_memchr_input2<'i>(
    c: &mut Criterion,
    group: &str,
    input: Input,
    bench: impl FnMut(Search2, &mut Bencher<'_>) + Clone + 'static,
) {
    macro_rules! def {
        ($name:expr, $kind:ident) => {
            if let Some(search) = input.$kind() {
                let corp = input.corpus;
                let name = format!("{}/{}", group, $name);
                let mut bench = bench.clone();
                define(c, &name, corp, Box::new(move |b| bench(search, b)));
            }
        };
    }
    def!("never", never2);
    def!("rare", rare2);
    def!("uncommon", uncommon2);
    def!("common", common2);
    def!("verycommon", verycommon2);
    def!("supercommon", supercommon2);
}

fn define_memchr_input3<'i>(
    c: &mut Criterion,
    group: &str,
    input: Input,
    bench: impl FnMut(Search3, &mut Bencher<'_>) + Clone + 'static,
) {
    macro_rules! def {
        ($name:expr, $kind:ident) => {
            if let Some(search) = input.$kind() {
                let corp = input.corpus;
                let name = format!("{}/{}", group, $name);
                let mut bench = bench.clone();
                define(c, &name, corp, Box::new(move |b| bench(search, b)));
            }
        };
    }
    def!("never", never3);
    def!("rare", rare3);
    def!("uncommon", uncommon3);
    def!("common", common3);
    def!("verycommon", verycommon3);
    def!("supercommon", supercommon3);
}
