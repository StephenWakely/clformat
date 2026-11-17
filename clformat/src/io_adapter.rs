//! Adapts a [`std::io::Write`] into a [`core::fmt::Write`] so that it can be
//! wrapped uniformly (e.g. by [`crate::ColumnTracker`]) alongside the other
//! output destinations supported by `clformat!`.
extern crate std;

pub struct IoWriteAdapter<W>(pub W);

impl<W: std::io::Write> core::fmt::Write for IoWriteAdapter<W> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Write as _;

    #[test]
    fn adapts_io_write() {
        let mut buf = std::vec::Vec::new();
        {
            let mut adapter = IoWriteAdapter(&mut buf);
            write!(adapter, "hello world").unwrap();
        }
        assert_eq!(b"hello world".to_vec(), buf);
    }
}
