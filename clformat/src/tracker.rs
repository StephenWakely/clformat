//! A [`core::fmt::Write`] wrapper that keeps track of the current output
//! column, so that directives such as `~T` (tabulate) and `~&` (fresh-line)
//! can be implemented.
use core::fmt::Write;

#[derive(Debug, Default)]
pub struct ColumnTracker<W> {
    inner: W,
    column: usize,
}

impl<W> ColumnTracker<W> {
    pub fn new(inner: W) -> Self {
        Self { inner, column: 0 }
    }

    /// The current output column (0-indexed), i.e. the number of characters
    /// written since the last newline.
    pub fn column(&self) -> usize {
        self.column
    }

    /// Unwraps this tracker, returning the underlying writer.
    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write> Write for ColumnTracker<W> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        match s.rfind('\n') {
            Some(pos) => self.column = s.len() - pos - 1,
            None => self.column += s.len(),
        }

        self.inner.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::string::String;

    use super::*;

    #[test]
    fn tracks_column() {
        let mut tracker = ColumnTracker::new(String::new());
        assert_eq!(0, tracker.column());

        write!(tracker, "hello").unwrap();
        assert_eq!(5, tracker.column());

        write!(tracker, " world\nhi").unwrap();
        assert_eq!(2, tracker.column());

        assert_eq!("hello world\nhi", tracker.inner);
    }

    #[test]
    fn newline_resets_to_zero() {
        let mut tracker = ColumnTracker::new(String::new());
        writeln!(tracker, "hello").unwrap();
        assert_eq!(0, tracker.column());
    }
}
