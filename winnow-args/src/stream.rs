//! The input: a whole command line as one byte buffer.
//!
//! Every word of `argv` is stored back to back, each one followed by [`SEP`]
//! (NUL). A command line can never contain NUL — neither Unix `argv` nor the
//! Windows command line can carry one — so the byte is free to mark where one
//! word ends and the next begins.
//!
//! Flattening `argv` this way is what lets the rest of the crate be ordinary
//! winnow parsers over a [`BStr`]: a position is one pointer, a checkpoint is a
//! copy of it, and `--path=x`, `-vpx` and `-p x` are all just byte patterns.
//! The only thing a flat buffer cannot say by itself is *where in the grammar*
//! the position is — at the start of a word, part-way through a bundle of short
//! flags, or past a `--` — so [`Argv`] carries that as a [`Mode`] next to the
//! bytes, and its checkpoint saves both. Backtracking in `alt` therefore
//! restores the mode too, which a `Stateful` wrapper would not.

use std::ffi::OsStr;

use winnow::stream::{
    AsBStr, BStr, Compare, CompareResult, FindSlice, Location, Needed, Offset, SliceLen, Stream,
    StreamIsPartial,
};

/// The byte that terminates every word in the buffer.
pub const SEP: u8 = 0;

/// An owned command line, flattened into NUL-terminated words.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct ArgvBuf {
    bytes: Vec<u8>,
}

impl ArgvBuf {
    /// Flatten `args`, which should not include the program name.
    ///
    /// A word containing NUL cannot come from a real command line; one passed in
    /// by hand would be read as two words.
    pub fn new<I, S>(args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut bytes = Vec::new();
        for arg in args {
            let arg = arg.as_ref().as_encoded_bytes();
            debug_assert!(!arg.contains(&SEP), "argv words cannot contain NUL");
            bytes.extend_from_slice(arg);
            bytes.push(SEP);
        }
        Self { bytes }
    }

    /// The current process's arguments, without the program name.
    pub fn from_env() -> Self {
        Self::new(std::env::args_os().skip(1))
    }

    /// A stream positioned at the first word.
    pub fn argv(&self) -> Argv<'_> {
        Argv::new(&self.bytes)
    }

    /// The flattened bytes, NUL-terminated words.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl std::fmt::Debug for ArgvBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.argv().fmt(f)
    }
}

/// Where in the grammar a position is.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    /// At the start of a word, and flags are still recognized.
    Word,
    /// Inside a bundle of short flags: the next byte is another letter, or the
    /// attached value of the letter just read.
    Bundle,
    /// Past a `--`: every word is a value.
    Stopped,
}

/// A command line being parsed: the [`Stream`] every parser in this crate reads.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Argv<'i> {
    initial: &'i [u8],
    input: &'i BStr,
    mode: Mode,
}

impl<'i> Argv<'i> {
    /// Parse a buffer that is already flattened: NUL-terminated words.
    pub fn new(bytes: &'i [u8]) -> Self {
        debug_assert!(
            bytes.last().is_none_or(|&b| b == SEP),
            "the last word must be NUL-terminated"
        );
        Self {
            initial: bytes,
            input: BStr::new(bytes),
            mode: Mode::Word,
        }
    }

    /// Whether every word has been consumed.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    /// Where in the grammar the stream is.
    #[inline(always)]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    #[inline(always)]
    pub(crate) fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    /// The unconsumed bytes.
    #[inline(always)]
    pub fn as_bytes(&self) -> &'i [u8] {
        self.input
    }

    /// Byte offset from the start of the command line.
    #[inline(always)]
    pub fn offset(&self) -> usize {
        self.initial.len() - self.input.len()
    }

    /// The whole command line this stream started from.
    pub fn initial(&self) -> &'i [u8] {
        self.initial
    }
}

impl std::fmt::Debug for Argv<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut words = self
            .input
            .split(|&b| b == SEP)
            .map(BStr::new)
            .collect::<Vec<_>>();
        // Every word is terminated, so splitting leaves an empty tail.
        words.pop();
        f.debug_struct("Argv")
            .field("mode", &self.mode)
            .field("words", &words)
            .finish()
    }
}

/// A saved position: the bytes and the [`Mode`] together.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ArgvCheckpoint<'i> {
    input: <&'i BStr as Stream>::Checkpoint,
    mode: Mode,
}

impl Offset for ArgvCheckpoint<'_> {
    #[inline(always)]
    fn offset_from(&self, start: &Self) -> usize {
        self.input.offset_from(&start.input)
    }
}

impl Offset for Argv<'_> {
    #[inline(always)]
    fn offset_from(&self, start: &Self) -> usize {
        self.input.offset_from(&start.input)
    }
}

impl<'i> Offset<ArgvCheckpoint<'i>> for Argv<'i> {
    #[inline(always)]
    fn offset_from(&self, start: &ArgvCheckpoint<'i>) -> usize {
        self.input.offset_from(&start.input)
    }
}

impl SliceLen for Argv<'_> {
    #[inline(always)]
    fn slice_len(&self) -> usize {
        self.input.len()
    }
}

impl<'i> Stream for Argv<'i> {
    type Token = u8;
    type Slice = &'i [u8];
    type IterOffsets = <&'i BStr as Stream>::IterOffsets;
    type Checkpoint = ArgvCheckpoint<'i>;

    #[inline(always)]
    fn iter_offsets(&self) -> Self::IterOffsets {
        self.input.iter_offsets()
    }
    #[inline(always)]
    fn eof_offset(&self) -> usize {
        self.input.eof_offset()
    }
    #[inline(always)]
    fn next_token(&mut self) -> Option<Self::Token> {
        self.input.next_token()
    }
    #[inline(always)]
    fn peek_token(&self) -> Option<Self::Token> {
        self.input.peek_token()
    }
    #[inline(always)]
    fn offset_for<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Token) -> bool,
    {
        self.input.offset_for(predicate)
    }
    #[inline(always)]
    fn offset_at(&self, tokens: usize) -> Result<usize, Needed> {
        self.input.offset_at(tokens)
    }
    #[inline(always)]
    fn next_slice(&mut self, offset: usize) -> Self::Slice {
        self.input.next_slice(offset)
    }
    #[inline(always)]
    fn peek_slice(&self, offset: usize) -> Self::Slice {
        self.input.peek_slice(offset)
    }
    #[inline(always)]
    fn checkpoint(&self) -> Self::Checkpoint {
        ArgvCheckpoint {
            input: self.input.checkpoint(),
            mode: self.mode,
        }
    }
    #[inline(always)]
    fn reset(&mut self, checkpoint: &Self::Checkpoint) {
        self.input.reset(&checkpoint.input);
        self.mode = checkpoint.mode;
    }
    fn trace(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?}", self.mode, self.input)
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

impl<'i, T> Compare<T> for Argv<'i>
where
    &'i BStr: Compare<T>,
{
    #[inline(always)]
    fn compare(&self, t: T) -> CompareResult {
        self.input.compare(t)
    }
}

impl<'i, S> FindSlice<S> for Argv<'i>
where
    &'i BStr: FindSlice<S>,
{
    #[inline(always)]
    fn find_slice(&self, substr: S) -> Option<std::ops::Range<usize>> {
        self.input.find_slice(substr)
    }
}

impl AsBStr for Argv<'_> {
    #[inline(always)]
    fn as_bstr(&self) -> &[u8] {
        self.input
    }
}
