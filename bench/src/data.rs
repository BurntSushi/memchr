#![allow(dead_code)]

pub const SHERLOCK_HUGE: &'static [u8] =
    include_bytes!("../data/sherlock/huge.txt");
pub const SHERLOCK_SMALL: &'static [u8] =
    include_bytes!("../data/sherlock/small.txt");
pub const SHERLOCK_TINY: &'static [u8] =
    include_bytes!("../data/sherlock/tiny.txt");

pub const SUBTITLE_EN_HUGE: &'static str =
    include_str!("../data/opensubtitles/en-huge.txt");
pub const SUBTITLE_EN_MEDIUM: &'static str =
    include_str!("../data/opensubtitles/en-medium.txt");
pub const SUBTITLE_EN_SMALL: &'static str =
    include_str!("../data/opensubtitles/en-small.txt");
pub const SUBTITLE_EN_TINY: &'static str =
    include_str!("../data/opensubtitles/en-tiny.txt");
pub const SUBTITLE_EN_TEENY: &'static str =
    include_str!("../data/opensubtitles/en-teeny.txt");

pub const SUBTITLE_RU_HUGE: &'static str =
    include_str!("../data/opensubtitles/ru-huge.txt");
pub const SUBTITLE_RU_MEDIUM: &'static str =
    include_str!("../data/opensubtitles/ru-medium.txt");
pub const SUBTITLE_RU_SMALL: &'static str =
    include_str!("../data/opensubtitles/ru-small.txt");
pub const SUBTITLE_RU_TINY: &'static str =
    include_str!("../data/opensubtitles/ru-tiny.txt");
pub const SUBTITLE_RU_TEENY: &'static str =
    include_str!("../data/opensubtitles/ru-teeny.txt");

pub const SUBTITLE_ZH_HUGE: &'static str =
    include_str!("../data/opensubtitles/zh-huge.txt");
pub const SUBTITLE_ZH_MEDIUM: &'static str =
    include_str!("../data/opensubtitles/zh-medium.txt");
pub const SUBTITLE_ZH_SMALL: &'static str =
    include_str!("../data/opensubtitles/zh-small.txt");
pub const SUBTITLE_ZH_TINY: &'static str =
    include_str!("../data/opensubtitles/zh-tiny.txt");
pub const SUBTITLE_ZH_TEENY: &'static str =
    include_str!("../data/opensubtitles/zh-teeny.txt");

pub const PATHOLOGICAL_MD5_HUGE: &'static str =
    include_str!("../data/pathological/md5-huge.txt");
pub const PATHOLOGICAL_RANDOM_HUGE: &'static str =
    include_str!("../data/pathological/random-huge.txt");
pub const PATHOLOGICAL_REPEATED_RARE_HUGE: &'static str =
    include_str!("../data/pathological/repeated-rare-huge.txt");
pub const PATHOLOGICAL_REPEATED_RARE_SMALL: &'static str =
    include_str!("../data/pathological/repeated-rare-small.txt");
pub const PATHOLOGICAL_DEFEAT_SIMPLE_VECTOR: &'static str =
    include_str!("../data/pathological/defeat-simple-vector.txt");
pub const PATHOLOGICAL_DEFEAT_SIMPLE_VECTOR_FREQ: &'static str =
    include_str!("../data/pathological/defeat-simple-vector-freq.txt");
pub const PATHOLOGICAL_DEFEAT_SIMPLE_VECTOR_REPEATED: &'static str =
    include_str!("../data/pathological/defeat-simple-vector-repeated.txt");

pub const SLICESLICE_I386: &'static str =
    include_str!("../data/sliceslice/i386.txt");
pub const SLICESLICE_WORDS: &'static str =
    include_str!("../data/sliceslice/words.txt");
pub const SLICESLICE_HAYSTACK: &'static str =
    include_str!("../data/sliceslice/haystack.txt");

pub const CODE_RUST_LIBRARY: &'static str =
    include_str!("../data/code/rust-library.rs");
