use crate::Error;

/// Converts a byte position into a `u32` source offset.
///
/// `Lexer::new` asserts the source fits in `u32`, so positions are always in
/// range.
#[expect(clippy::cast_possible_truncation)]
#[inline]
pub(super) fn source_offset(position: usize) -> u32 {
    debug_assert!(u32::try_from(position).is_ok());
    position as u32
}

/// Byte cursor over GraphQL source text.
///
/// Positions are `u32`: `Lexer::new` asserts the source is at most 4 GiB.
#[derive(Debug, Clone)]
pub(crate) struct Cursor<'a> {
    pub(super) index: u32,
    pub(super) offset: u32,
    pub(super) source: &'a str,
    pub(super) bytes: &'a [u8],
    pub(super) next: u32,
    pub(crate) err: Option<Error>,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(input: &'a str) -> Cursor<'a> {
        Cursor { index: 0, offset: 0, source: input, bytes: input.as_bytes(), next: 0, err: None }
    }
}

impl<'a> Cursor<'a> {
    /// Current place (index) in the cursor.
    pub(crate) fn index(&self) -> u32 {
        self.index
    }

    /// Length of the source text.
    #[inline]
    pub(super) fn len(&self) -> u32 {
        source_offset(self.bytes.len())
    }

    /// Consumes the remaining bytes of a name token and returns its full text.
    ///
    /// The first name byte is already consumed by `bump` in `State::Start`; this
    /// scans the rest of the name in a tight loop over the raw bytes, avoiding
    /// the per-byte state-machine dispatch of the main lexer loop. It leaves the
    /// cursor in the exact position the per-byte path would: stopped before the
    /// terminator (mirroring `prev_str`), or at end of input with the
    /// EOF-adjacent index preserved for token-limit diagnostics (mirroring
    /// `current_str`).
    pub(super) fn consume_name(&mut self) -> &'a str {
        let mut end = self.next;
        // `get`-based access keeps a single bounds check per byte: with `u32`
        // positions the optimizer cannot elide an indexed access from an
        // `end < len` guard.
        while self.bytes.get(end as usize).is_some_and(|&byte| super::is_name_continue(byte)) {
            end += 1;
        }

        let slice = &self.source[self.index as usize..end as usize];
        self.index = if end == self.len() && end > 0 { end - 1 } else { end };
        self.offset = end;
        self.next = end;
        slice
    }

    /// Returns the token text before the last consumed byte and rewinds to it.
    pub(crate) fn prev_str(&mut self) -> &'a str {
        let slice = &self.source[self.index as usize..self.offset as usize];

        self.index = self.offset;
        self.next = self.offset;

        slice
    }

    /// Returns the token text through the last consumed byte.
    pub(crate) fn current_str(&mut self) -> &'a str {
        let slice = &self.source[self.index as usize..self.next as usize];
        // Preserve the previous EOF-adjacent cursor position used by token-limit diagnostics.
        self.index =
            if self.next == self.len() && self.next > 0 { self.next - 1 } else { self.next };
        slice
    }

    /// Moves to the next byte.
    pub(crate) fn bump(&mut self) -> Option<u8> {
        let c = *self.bytes.get(self.next as usize)?;
        self.offset = self.next;
        self.next += 1;

        Some(c)
    }

    /// Consumes the next byte if it matches.
    pub(crate) fn eatc(&mut self, c: u8) -> bool {
        if self.bytes.get(self.next as usize) == Some(&c) {
            self.offset = self.next;
            self.next += 1;
            return true;
        }

        false
    }

    /// Consumes the rest of the UTF-8 scalar at the current byte offset.
    pub(crate) fn consume_current_char(&mut self) -> char {
        let c = self.source[self.offset as usize..].chars().next().unwrap();
        self.next = self.offset + source_offset(c.len_utf8());
        c
    }

    /// Consumes a Unicode byte order mark at the current byte offset.
    pub(crate) fn eat_bom(&mut self) -> bool {
        const BOM: &[u8] = b"\xEF\xBB\xBF";

        if self.bytes[self.offset as usize..].starts_with(BOM) {
            self.next = self.offset + source_offset(BOM.len());
            return true;
        }

        false
    }

    /// Whether the next bytes are a Unicode byte order mark.
    pub(super) fn at_bom(&self) -> bool {
        self.bytes[self.next as usize..].starts_with(b"\xEF\xBB\xBF")
    }

    /// Consumes the remaining bytes of a comment (the `#` is already consumed)
    /// and returns the end of its text.
    ///
    /// Scans to the next line terminator with `memchr` instead of the per-byte
    /// main lexer loop. Leaves the cursor exactly where the per-byte path
    /// would: stopped before the terminator (mirroring `prev_str`), or at end
    /// of input with the EOF-adjacent index preserved for token-limit
    /// diagnostics (mirroring `current_str`).
    pub(super) fn seek_line_end(&mut self) -> u32 {
        let end = match memchr::memchr2(b'\n', b'\r', &self.bytes[self.next as usize..]) {
            Some(found) => {
                let end = self.next + source_offset(found);
                self.index = end;
                end
            }
            None => {
                let end = self.len();
                // `end >= 1` because the leading `#` is already consumed.
                self.index = end - 1;
                end
            }
        };
        self.offset = end;
        self.next = end;
        end
    }

    /// Consumes the remaining bytes of a whitespace run and returns its text.
    ///
    /// The first whitespace unit is already consumed in `State::Start`; this
    /// scans the rest of the run in a tight loop over the raw bytes (assimilated
    /// whitespace plus byte-order marks), avoiding the per-byte state-machine
    /// dispatch of the main lexer loop. It leaves the cursor exactly where the
    /// per-byte path would: stopped before the terminator (mirroring `prev_str`),
    /// or at end of input with the EOF-adjacent index preserved for token-limit
    /// diagnostics (mirroring `current_str`).
    pub(super) fn consume_whitespace(&mut self) -> &'a str {
        const BOM: &[u8] = b"\xEF\xBB\xBF";
        let mut end = self.next;
        // `get`-based access keeps a single bounds check per byte: with `u32`
        // positions the optimizer cannot elide an indexed access from an
        // `end < len` guard.
        while let Some(&byte) = self.bytes.get(end as usize) {
            if super::is_whitespace_assimilated(byte) {
                end += 1;
            } else if byte == 0xEF && self.bytes[end as usize..].starts_with(BOM) {
                end += source_offset(BOM.len());
            } else {
                break;
            }
        }

        let slice = &self.source[self.index as usize..end as usize];
        self.index = if end == self.len() && end > 0 { end - 1 } else { end };
        self.offset = end;
        self.next = end;
        slice
    }

    /// Drains the current token to the end of the source.
    pub(crate) fn drain(&mut self) -> &'a str {
        let start = self.index;
        self.index = self.len();
        self.next = self.len();

        self.source.get(start as usize..).unwrap()
    }

    /// Add error object to the cursor.
    pub(crate) fn add_err(&mut self, err: Error) {
        self.err = Some(err)
    }
}
