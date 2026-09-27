//! The input: `argv` as a slice of words.
//!
//! [`Argv`] is a winnow [`Stream`] whose tokens are the words of the command
//! line, borrowed as [`BStr`], so nothing is copied or joined and any byte,
//! NUL included, can appear in a word. A word is never re-split: `"a b"` is
//! one token, the same as the shell handed it over.
//!
//! Offsets are counted in bytes as if every word were followed by one
//! separator, so a position inside a bundle of short flags (`-vq`) is still
//! a single number that shrinks as the lexer reads a letter. That keeps
//! `repeat`'s "parser must consume" check and error offsets meaningful.

use std::ffi::OsStr;

use winnow::stream::{BStr, Location, Needed, Offset, SliceLen, Stream, StreamIsPartial};

/// Borrow each argument as a [`BStr`] word.
pub fn words<S: AsRef<OsStr>>(args: &[S]) -> Vec<&BStr> {
    args.iter()
        .map(|arg| BStr::new(arg.as_ref().as_encoded_bytes()))
        .collect()
}

/// Where in the grammar a position is.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    /// At the start of a word, and flags are still recognized.
    Word,
    /// Inside a bundle of short flags: the rest of the current word is more
    /// letters, or the attached value of the letter just read.
    Bundle,
    /// Past a `--`: every word is a value.
    Stopped,
}

/// A command line being parsed: the [`Stream`] every parser in this crate reads.
///
/// Its checkpoint is the whole state, so `alt` backtracking restores the
/// [`Mode`] and the position inside a bundle along with the word.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Argv<'i> {
    /// Unfinished words; `words[0]` is the current one.
    words: &'i [&'i BStr],
    /// Bytes of `words[0]` already read, when inside a bundle.
    skip: u32,
    /// Offset to the end: unread bytes plus one separator per unfinished word.
    remaining: u32,
    total: u32,
    mode: Mode,
}

impl<'i> Argv<'i> {
    /// Parse `words`, which should not include the program name.
    pub fn new(words: &'i [&'i BStr]) -> Self {
        let total: usize = words.iter().map(|w| w.len() + 1).sum();
        // Kept narrow so a checkpoint is two 16-byte moves; argv is capped far below this.
        let total = u32::try_from(total).expect("a command line under 4 GiB");
        Self {
            words,
            skip: 0,
            remaining: total,
            total,
            mode: Mode::Word,
        }
    }

    /// Whether every word has been consumed.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Where in the grammar the stream is.
    #[inline(always)]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Read every remaining word as a value, as if `--` had been typed. An
    /// argument declared `double_dash = "automatic"` does this once it has a value.
    #[inline]
    pub fn stop_flags(&mut self) {
        if self.mode != Mode::Bundle {
            self.mode = Mode::Stopped;
        }
    }

    /// Recognize flags again after `--` or [`Argv::stop_flags`]: a command's
    /// `restart_token` starts a fresh invocation.
    #[inline]
    pub fn resume_flags(&mut self) {
        if self.mode == Mode::Stopped {
            self.mode = Mode::Word;
        }
    }

    #[inline(always)]
    pub(crate) fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    /// Byte offset from the start of the command line, one separator counted per word.
    #[inline(always)]
    pub fn offset(&self) -> usize {
        (self.total - self.remaining) as usize
    }

    /// The unread part of the current word; empty at the end.
    #[inline(always)]
    pub fn front(&self) -> &'i [u8] {
        match self.words.first() {
            Some(word) => &word[self.skip as usize..],
            None => &[],
        }
    }

    /// Read `n` bytes of the current word, staying inside it.
    #[inline(always)]
    pub(crate) fn take_bytes(&mut self, n: usize) {
        debug_assert!(n < self.front().len() + 1);
        self.skip += n as u32;
        self.remaining -= n as u32;
    }

    /// Read the rest of the current word and move to the next one.
    #[inline(always)]
    pub(crate) fn take_word(&mut self) -> &'i BStr {
        let rest = self.front();
        self.remaining -= rest.len() as u32 + 1;
        self.words = &self.words[1..];
        self.skip = 0;
        if self.mode == Mode::Bundle {
            self.mode = Mode::Word;
        }
        BStr::new(rest)
    }

    /// Number of whole words spanning `offset`, which must fall on a word boundary.
    fn words_in(&self, offset: usize) -> usize {
        let mut end = 0;
        for (n, (at, _)) in self.iter_offsets().enumerate() {
            if at == offset {
                return n;
            }
            end = at;
        }
        assert_eq!(
            offset, self.remaining as usize,
            "offset {offset} is inside a word (last start {end})"
        );
        self.words.len()
    }
}

impl std::fmt::Debug for Argv<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut words: Vec<&BStr> = self.words.to_vec();
        if let Some(first) = words.first_mut() {
            *first = BStr::new(&first[self.skip as usize..]);
        }
        f.debug_struct("Argv")
            .field("mode", &self.mode)
            .field("words", &words)
            .finish()
    }
}

/// Iterator over the remaining words with their offsets.
#[derive(Clone, Debug)]
pub struct IterOffsets<'i> {
    words: std::slice::Iter<'i, &'i BStr>,
    skip: usize,
    offset: usize,
}

impl<'i> Iterator for IterOffsets<'i> {
    type Item = (usize, &'i BStr);

    fn next(&mut self) -> Option<Self::Item> {
        let word = BStr::new(&self.words.next()?[self.skip..]);
        self.skip = 0;
        let at = self.offset;
        self.offset += word.len() + 1;
        Some((at, word))
    }
}

impl Offset for Argv<'_> {
    #[inline(always)]
    fn offset_from(&self, start: &Self) -> usize {
        (start.remaining - self.remaining) as usize
    }
}

impl SliceLen for Argv<'_> {
    #[inline(always)]
    fn slice_len(&self) -> usize {
        self.remaining as usize
    }
}

/// Tokens are words (the unread part of the current one first); slices are
/// runs of whole words, so `next_slice` must not be asked to split a word.
impl<'i> Stream for Argv<'i> {
    type Token = &'i BStr;
    type Slice = &'i [&'i BStr];
    type IterOffsets = IterOffsets<'i>;
    type Checkpoint = Self;

    #[inline(always)]
    fn iter_offsets(&self) -> Self::IterOffsets {
        IterOffsets {
            words: self.words.iter(),
            skip: self.skip as usize,
            offset: 0,
        }
    }
    #[inline(always)]
    fn eof_offset(&self) -> usize {
        self.remaining as usize
    }
    #[inline(always)]
    fn next_token(&mut self) -> Option<Self::Token> {
        if self.is_empty() {
            None
        } else {
            Some(self.take_word())
        }
    }
    #[inline(always)]
    fn peek_token(&self) -> Option<Self::Token> {
        (!self.is_empty()).then(|| BStr::new(self.front()))
    }
    fn offset_for<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Token) -> bool,
    {
        self.iter_offsets()
            .find(|&(_, word)| predicate(word))
            .map(|(at, _)| at)
    }
    fn offset_at(&self, tokens: usize) -> Result<usize, Needed> {
        let mut iter = self.iter_offsets();
        match iter.nth(tokens) {
            Some((at, _)) => Ok(at),
            None if tokens == self.words.len() => Ok(self.remaining as usize),
            None => Err(Needed::new(tokens - self.words.len())),
        }
    }
    fn next_slice(&mut self, offset: usize) -> Self::Slice {
        let slice = self.peek_slice(offset);
        for _ in 0..slice.len() {
            self.take_word();
        }
        slice
    }
    fn peek_slice(&self, offset: usize) -> Self::Slice {
        assert!(
            self.skip == 0 || offset == 0,
            "cannot slice whole words inside a bundle"
        );
        &self.words[..self.words_in(offset)]
    }
    #[inline(always)]
    fn checkpoint(&self) -> Self::Checkpoint {
        *self
    }
    #[inline(always)]
    fn reset(&mut self, checkpoint: &Self::Checkpoint) {
        *self = *checkpoint;
    }
    fn trace(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl StreamIsPartial for Argv<'_> {
    type PartialState = ();

    #[inline(always)]
    fn complete(&mut self) -> Self::PartialState {}
    #[inline(always)]
    fn restore_partial(&mut self, _state: Self::PartialState) {}
    #[inline(always)]
    fn is_partial_supported() -> bool {
        false
    }
}

impl Location for Argv<'_> {
    #[inline(always)]
    fn previous_token_end(&self) -> usize {
        self.offset()
    }
    #[inline(always)]
    fn current_token_start(&self) -> usize {
        self.offset()
    }
}

#[cfg(test)]
mod tests {
    use winnow::combinator::repeat;
    use winnow::prelude::*;
    use winnow::token::{any, take};

    use super::*;

    fn line<'a>(words: &'a [&'a str]) -> Vec<&'a BStr> {
        words.iter().map(BStr::new).collect()
    }

    #[test]
    fn generic_token_parsers_see_whole_words() {
        let words = line(&["a b", "", "c"]);
        let mut input = Argv::new(&words);
        assert_eq!(
            any::<_, crate::Error>.parse_next(&mut input).unwrap(),
            "a b"
        );
        assert_eq!(input.offset(), 4);
        let rest: &[&BStr] = take::<_, _, crate::Error>(2usize)
            .parse_next(&mut input)
            .unwrap();
        assert_eq!(rest, &words[1..]);
        assert!(input.is_empty());
    }

    #[test]
    fn reading_a_letter_counts_as_progress() {
        let words = line(&["-abc"]);
        let mut input = Argv::new(&words);
        let letters: Vec<_> = repeat(0.., crate::token::arg)
            .parse_next(&mut input)
            .unwrap();
        assert_eq!(letters.len(), 3);
        assert!(input.is_empty());
    }

    #[test]
    fn checkpoint_restores_the_position_inside_a_bundle() {
        let words = line(&["-ab", "x"]);
        let mut input = Argv::new(&words);
        crate::token::arg(&mut input).unwrap();
        let saved = input.checkpoint();
        crate::token::arg(&mut input).unwrap();
        crate::token::arg(&mut input).unwrap();
        input.reset(&saved);
        assert_eq!((input.mode(), input.front()), (Mode::Bundle, &b"b"[..]));
    }
}
