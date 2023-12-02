use core::fmt::Write;

#[derive(Debug, Default)]
pub struct Ruler {
    length: usize,
}

impl Write for Ruler {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.length += s.len();
        Ok(())
    }
}

impl Ruler {
    pub fn length(&self) -> usize {
        self.length
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::string::String;

    use super::*;

    #[allow(dead_code)]
    #[derive(Debug)]
    struct Noog {
        thing: usize,
    }

    #[test]
    fn debug_length() {
        let zork = Noog { thing: 342 };

        let mut s = String::new();
        write!(s, " {:?} ", zork).unwrap();

        let mut ruler = Ruler::default();
        write!(ruler, " {:?} ", zork).unwrap();

        assert_eq!(s.len(), ruler.length);
    }
}
