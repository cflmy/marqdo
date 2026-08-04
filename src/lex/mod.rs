//! Line classification and tokenization (no Flex).
//! Implementation arrives in M1.

#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Comment,
    Code,
    Blank,
}
