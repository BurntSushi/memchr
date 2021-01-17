/// A description of an input to benchmark on.
#[derive(Clone, Copy, Debug)]
pub struct Input {
    /// The bytes to search.
    pub corpus: &'static [u8],
    /// Distinct bytes that never occur in the input.
    pub never: &'static [SearchByte],
    /// Distinct bytes that occur very rarely (<0.1%).
    pub rare: &'static [SearchByte],
    /// Distinct bytes that are uncommon (~1%).
    pub uncommon: &'static [SearchByte],
    /// Distinct bytes that are common (~5%).
    pub common: &'static [SearchByte],
    /// Distinct bytes that are very common (~10%).
    pub verycommon: &'static [SearchByte],
    /// Distinct bytes that are super common (>90%).
    pub supercommon: &'static [SearchByte],
}

pub const HUGE: Input = Input {
    corpus: crate::data::SHERLOCK_HUGE,
    never: &[
        SearchByte { byte: b'<', count: 0 },
        SearchByte { byte: b'>', count: 0 },
        SearchByte { byte: b'=', count: 0 },
    ],
    rare: &[
        SearchByte { byte: b'z', count: 151 },
        SearchByte { byte: b'R', count: 275 },
        SearchByte { byte: b'J', count: 120 },
    ],
    uncommon: &[
        SearchByte { byte: b'b', count: 6124 },
        SearchByte { byte: b'p', count: 6989 },
        SearchByte { byte: b'.', count: 6425 },
    ],
    common: &[
        SearchByte { byte: b'a', count: 35301 },
        SearchByte { byte: b't', count: 39268 },
        SearchByte { byte: b'o', count: 34495 },
    ],
    verycommon: &[SearchByte { byte: b' ', count: 97626 }],
    supercommon: &[],
};

pub const SMALL: Input = Input {
    corpus: crate::data::SHERLOCK_SMALL,
    never: &[
        SearchByte { byte: b'<', count: 0 },
        SearchByte { byte: b'>', count: 0 },
        SearchByte { byte: b'=', count: 0 },
    ],
    rare: &[
        SearchByte { byte: b'R', count: 1 },
        SearchByte { byte: b'P', count: 1 },
        SearchByte { byte: b'T', count: 1 },
    ],
    uncommon: &[
        SearchByte { byte: b'b', count: 8 },
        SearchByte { byte: b'g', count: 8 },
        SearchByte { byte: b'p', count: 8 },
    ],
    common: &[
        SearchByte { byte: b'a', count: 44 },
        SearchByte { byte: b'h', count: 34 },
        SearchByte { byte: b'i', count: 35 },
    ],
    verycommon: &[SearchByte { byte: b' ', count: 106 }],
    supercommon: &[],
};

pub const TINY: Input = Input {
    corpus: crate::data::SHERLOCK_TINY,
    never: &[
        SearchByte { byte: b'<', count: 0 },
        SearchByte { byte: b'>', count: 0 },
        SearchByte { byte: b'=', count: 0 },
    ],
    rare: &[
        SearchByte { byte: b'.', count: 1 },
        SearchByte { byte: b'H', count: 1 },
        SearchByte { byte: b'M', count: 1 },
    ],
    uncommon: &[
        SearchByte { byte: b'l', count: 5 },
        SearchByte { byte: b's', count: 5 },
        SearchByte { byte: b'e', count: 6 },
    ],
    common: &[SearchByte { byte: b' ', count: 11 }],
    verycommon: &[],
    supercommon: &[],
};

pub const EMPTY: Input = Input {
    corpus: &[],
    never: &[
        SearchByte { byte: b'a', count: 0 },
        SearchByte { byte: b'b', count: 0 },
        SearchByte { byte: b'c', count: 0 },
    ],
    rare: &[],
    uncommon: &[],
    common: &[],
    verycommon: &[],
    supercommon: &[],
};

impl Input {
    /// Return all of this input's "never" bytes only if there are at least
    /// `min` of them.
    fn never(&self, min: usize) -> Option<&'static [SearchByte]> {
        if self.never.len() < min {
            None
        } else {
            Some(self.never)
        }
    }

    pub fn never1(&self) -> Option<Search1> {
        self.never(1).and_then(|bytes| Search1::new(self.corpus, bytes))
    }

    pub fn never2(&self) -> Option<Search2> {
        self.never(2).and_then(|bytes| Search2::new(self.corpus, bytes))
    }

    pub fn never3(&self) -> Option<Search3> {
        self.never(3).and_then(|bytes| Search3::new(self.corpus, bytes))
    }

    /// Return all of this input's "rare" bytes only if there are at least
    /// `min` of them.
    fn rare(&self, min: usize) -> Option<&'static [SearchByte]> {
        if self.rare.len() < min {
            None
        } else {
            Some(self.rare)
        }
    }

    pub fn rare1(&self) -> Option<Search1> {
        self.rare(1).and_then(|bytes| Search1::new(self.corpus, bytes))
    }

    pub fn rare2(&self) -> Option<Search2> {
        self.rare(2).and_then(|bytes| Search2::new(self.corpus, bytes))
    }

    pub fn rare3(&self) -> Option<Search3> {
        self.rare(3).and_then(|bytes| Search3::new(self.corpus, bytes))
    }

    /// Return all of this input's "uncommon" bytes only if there are at least
    /// `min` of them.
    fn uncommon(&self, min: usize) -> Option<&'static [SearchByte]> {
        if self.uncommon.len() < min {
            None
        } else {
            Some(self.uncommon)
        }
    }

    pub fn uncommon1(&self) -> Option<Search1> {
        self.uncommon(1).and_then(|bytes| Search1::new(self.corpus, bytes))
    }

    pub fn uncommon2(&self) -> Option<Search2> {
        self.uncommon(2).and_then(|bytes| Search2::new(self.corpus, bytes))
    }

    pub fn uncommon3(&self) -> Option<Search3> {
        self.uncommon(3).and_then(|bytes| Search3::new(self.corpus, bytes))
    }

    /// Return all of this input's "common" bytes only if there are at least
    /// `min` of them.
    fn common(&self, min: usize) -> Option<&'static [SearchByte]> {
        if self.common.len() < min {
            None
        } else {
            Some(self.common)
        }
    }

    pub fn common1(&self) -> Option<Search1> {
        self.common(1).and_then(|bytes| Search1::new(self.corpus, bytes))
    }

    pub fn common2(&self) -> Option<Search2> {
        self.common(2).and_then(|bytes| Search2::new(self.corpus, bytes))
    }

    pub fn common3(&self) -> Option<Search3> {
        self.common(3).and_then(|bytes| Search3::new(self.corpus, bytes))
    }

    /// Return all of this input's "verycommon" bytes only if there are at
    /// least `min` of them.
    fn verycommon(&self, min: usize) -> Option<&'static [SearchByte]> {
        if self.verycommon.len() < min {
            None
        } else {
            Some(self.verycommon)
        }
    }

    pub fn verycommon1(&self) -> Option<Search1> {
        self.verycommon(1).and_then(|bytes| Search1::new(self.corpus, bytes))
    }

    pub fn verycommon2(&self) -> Option<Search2> {
        self.verycommon(2).and_then(|bytes| Search2::new(self.corpus, bytes))
    }

    pub fn verycommon3(&self) -> Option<Search3> {
        self.verycommon(3).and_then(|bytes| Search3::new(self.corpus, bytes))
    }

    /// Return all of this input's "supercommon" bytes only if there are at
    /// least `min` of them.
    fn supercommon(&self, min: usize) -> Option<&'static [SearchByte]> {
        if self.supercommon.len() < min {
            None
        } else {
            Some(self.supercommon)
        }
    }

    pub fn supercommon1(&self) -> Option<Search1> {
        self.supercommon(1).and_then(|bytes| Search1::new(self.corpus, bytes))
    }

    pub fn supercommon2(&self) -> Option<Search2> {
        self.supercommon(2).and_then(|bytes| Search2::new(self.corpus, bytes))
    }

    pub fn supercommon3(&self) -> Option<Search3> {
        self.supercommon(3).and_then(|bytes| Search3::new(self.corpus, bytes))
    }
}

/// A description of a single byte, along with the number of times it is
/// expected to occur for a particular data source.
#[derive(Clone, Copy, Debug)]
pub struct SearchByte {
    /// A byte. Any byte.
    pub byte: u8,
    /// The number of times it is expected to occur.
    pub count: usize,
}

/// A description of a search for one particular byte.
#[derive(Clone, Copy, Debug)]
pub struct Search1 {
    /// The thing to search.
    pub corpus: &'static [u8],
    /// The thing to search for. One byte.
    pub byte1: SearchByte,
}

impl Search1 {
    pub fn new(
        corpus: &'static [u8],
        bytes: &[SearchByte],
    ) -> Option<Search1> {
        if bytes.len() < 1 {
            None
        } else {
            Some(Search1 { corpus, byte1: bytes[0] })
        }
    }
}

/// A description of a search for one of two particular bytes.
#[derive(Clone, Copy, Debug)]
pub struct Search2 {
    /// The thing to search.
    pub corpus: &'static [u8],
    /// One of the things to search for.
    pub byte1: SearchByte,
    /// The other thing to search for.
    pub byte2: SearchByte,
}

impl Search2 {
    pub fn new(
        corpus: &'static [u8],
        bytes: &[SearchByte],
    ) -> Option<Search2> {
        if bytes.len() < 2 {
            None
        } else {
            Some(Search2 { corpus, byte1: bytes[0], byte2: bytes[1] })
        }
    }
}

/// A description of a search for one of three particular bytes.
#[derive(Clone, Copy, Debug)]
pub struct Search3 {
    /// The thing to search.
    pub corpus: &'static [u8],
    /// One of the things to search for.
    pub byte1: SearchByte,
    /// The other thing to search for.
    pub byte2: SearchByte,
    /// Another thing to search for.
    pub byte3: SearchByte,
}

impl Search3 {
    pub fn new(
        corpus: &'static [u8],
        bytes: &[SearchByte],
    ) -> Option<Search3> {
        if bytes.len() < 3 {
            None
        } else {
            Some(Search3 {
                corpus,
                byte1: bytes[0],
                byte2: bytes[1],
                byte3: bytes[2],
            })
        }
    }
}
