#[cfg(test)]
mod tests {
    use clformat::clformat;

    #[test]
    fn it_works() {
        let dr = "Dr Ponk";
        let x = clformat!(nil, "Hello, ~A", dr);

        assert_eq!("Hello, Dr Ponk", x);
    }

    #[test]
    fn writer() {
        let mut write_to_me = String::new();
        let dr = "Dr Nork";
        clformat!(write_to_me, "Hey, ~A", dr);

        assert_eq!("Hey, Dr Nork", write_to_me);
    }

    #[test]
    fn smoke_stdout() {
        let dr = "Dr Ponk";
        clformat!(t, "~%Hello, ~A~%", dr);
    }

    #[test]
    fn iteration() {
        let x = vec!["ook", "onk", "nork", "nonk"];
        let x = clformat!(nil, "~{~A~^, ~}", x);

        assert_eq!("ook, onk, nork, nonk", x);
    }

    #[test]
    fn skip_iteration() {
        let x = vec!["ook", "onk", "nork", "nonk"];
        let x = clformat!(nil, "~{~A~*~^, ~}", x);

        assert_eq!("ook, nork", x);
    }

    #[test]
    fn decimals() {
        let num = 4200_isize;
        assert_eq!("4200", clformat!(nil, "~D", num));
        assert_eq!("4,200", clformat!(nil, "~:D", num));

        let num = -4200_isize;
        assert_eq!("-4,200", clformat!(nil, "~:D", num));
        assert_eq!("____-4,200", clformat!(nil, "~10,'_:D", num));

        let num = 4200_usize;
        assert_eq!("4200", clformat!(nil, "~D", num));
        assert_eq!("4,200", clformat!(nil, "~:D", num));
    }

    #[test]
    fn floats() {
        let num = std::f64::consts::PI;
        assert_eq!("3.14", clformat!(nil, "~,2F", num));
        assert_eq!(" 3.142", clformat!(nil, "~6,3F", num));
        assert_eq!("--3.141593", clformat!(nil, "~10,6,,,,'-F", num));

        let num = 3.5;
        assert_eq!("3.50", clformat!(nil, "~,2F", num));
    }

    #[test]
    fn alignment() {
        let text = "zogwobble";
        assert_eq!("  zogwobble  ", clformat!(nil, "~13:@<~A~>", text));
        assert_eq!("zogwobble----", clformat!(nil, "~13,0,0,'-<~A~>", text));
        assert_eq!("zogwobble----", clformat!(nil, "~13,0,0,'-@<~A~>", text));
        assert_eq!("----zogwobble", clformat!(nil, "~13,0,0,'-:<~A~>", text));
        assert_eq!("--zogwobble--", clformat!(nil, "~13,0,0,'-:@<~A~>", text));
    }

    #[derive(Debug)]
    struct Nork;

    #[test]
    fn alignment_debug() {
        // Output and align a debug output of a struct and a string.
        let text = "wobble";
        assert_eq!("  Norkwobble  ", clformat!(nil, "~14:@<~S~A~>", Nork, text));
        assert_eq!(
            "Norkwobble----",
            clformat!(nil, "~14,0,0,'-<~S~A~>", Nork, text)
        );
        assert_eq!(
            "Norkwobble----",
            clformat!(nil, "~14,0,0,'-@<~S~A~>", Nork, text)
        );
        assert_eq!(
            "----Norkwobble",
            clformat!(nil, "~14,0,0,'-:<~S~A~>", Nork, text)
        );
        assert_eq!(
            "--Norkwobble--",
            clformat!(nil, "~14,0,0,'-:@<~S~A~>", Nork, text)
        );
    }

    #[test]
    fn conditional() {
        let x = 2;
        assert_eq!("nork", clformat!(nil, "~[zork~;plork~;nork~:;gork~]", x));

        let x = 0;
        assert_eq!("zork", clformat!(nil, "~[zork~;plork~;nork~:;gork~]", x));

        let x = 100;
        assert_eq!("gork", clformat!(nil, "~[zork~;plork~;nork~:;gork~]", x));
    }

    #[test]
    fn conditional_consumes() {
        let fish = Some(2);
        let toes: Option<usize> = None;

        assert_eq!(
            "2 fish ",
            clformat!(nil, "~@[~a fish ~]~@[~a toes~]", fish, toes)
        );
    }

    #[test]
    fn boolean_conditional() {
        let x = true;
        assert_eq!("nork", clformat!(nil, "~:[nork~;zoggle~]", x));

        let x = false;
        assert_eq!("zoggle", clformat!(nil, "~:[nork~;zoggle~]", x));
    }

    #[test]
    fn tilde() {
        assert_eq!("~", clformat!(nil, "~~",));
        assert_eq!("~~", clformat!(nil, "~~~~",));
    }

    #[test]
    fn hex_octal_binary() {
        let num = 255_isize;
        assert_eq!("FF", clformat!(nil, "~X", num));
        assert_eq!("377", clformat!(nil, "~O", num));
        assert_eq!("11111111", clformat!(nil, "~B", num));
        assert_eq!("00FF", clformat!(nil, "~4,'0X", num));
    }

    #[test]
    fn explicit_radix() {
        let num = 255_isize;
        assert_eq!("FF", clformat!(nil, "~16R", num));
        assert_eq!("11111111", clformat!(nil, "~2R", num));
        let num = 1234567_isize;
        assert_eq!("1,234,567", clformat!(nil, "~10:R", num));
    }

    #[test]
    fn english_numbers() {
        let num = 42_isize;
        assert_eq!("forty-two", clformat!(nil, "~R", num));
        assert_eq!("forty-second", clformat!(nil, "~:R", num));

        let num = 4_isize;
        assert_eq!("IV", clformat!(nil, "~@R", num));
        assert_eq!("IIII", clformat!(nil, "~:@R", num));

        let num = 1999_isize;
        assert_eq!("MCMXCIX", clformat!(nil, "~M", num));
        assert_eq!("MCMXCIX", clformat!(nil, "~@R", num));
    }

    #[test]
    fn character() {
        assert_eq!("a", clformat!(nil, "~C", 'a'));
        assert_eq!("Space", clformat!(nil, "~:C", ' '));
        assert_eq!("a", clformat!(nil, "~:C", 'a'));
        assert_eq!("#\\Space", clformat!(nil, "~@C", ' '));
        assert_eq!("#\\a", clformat!(nil, "~@C", 'a'));
    }

    #[test]
    fn plural() {
        // The non-colon form consumes its own argument.
        assert_eq!("", clformat!(nil, "~P", 1));
        assert_eq!("s", clformat!(nil, "~P", 2));

        // The colon form is the common "N cat(s)" idiom: it re-uses the
        // argument that was just consumed by the preceding directive
        // instead of consuming (and requiring) a new one.
        assert_eq!("1 cat", clformat!(nil, "~D cat~:P", 1));
        assert_eq!("2 cats", clformat!(nil, "~D cat~:P", 2));
        assert_eq!("1 party", clformat!(nil, "~D part~:@P", 1));
        assert_eq!("2 parties", clformat!(nil, "~D part~:@P", 2));
    }

    #[test]
    fn newline_alias() {
        assert_eq!("a\nb", clformat!(nil, "a~Nb",));
        assert_eq!("a\nb", clformat!(nil, "a~%b",));
    }

    #[test]
    fn fresh_line() {
        assert_eq!("a\nb", clformat!(nil, "a~&b",));
        assert_eq!("a\nb", clformat!(nil, "a\n~&b",));
    }

    #[test]
    fn tabulation() {
        // Moves forward to column 5.
        assert_eq!("ab   cd", clformat!(nil, "ab~5Tcd",));
        // Already past column 1, and colinc (default 1) means any column is
        // valid, so nothing is inserted.
        assert_eq!("abcd", clformat!(nil, "ab~1Tcd",));
        // Already at column 2, which is a valid colnum + k*colinc position
        // (k=0), so nothing is inserted.
        assert_eq!("abcd", clformat!(nil, "ab~2,4Tcd",));
        // Past column 2, so move to the next colnum + k*colinc column: 6.
        assert_eq!("abc   cd", clformat!(nil, "abc~2,4Tcd",));
    }

    #[test]
    fn write_alias() {
        #[derive(Debug)]
        struct Thing;
        assert_eq!("Thing", clformat!(nil, "~W", Thing));
    }

    #[test]
    fn exponential() {
        assert_eq!("3.142e+4", clformat!(nil, "~,3E", 31415.9));
    }

    #[test]
    fn general() {
        assert_eq!("123.456", clformat!(nil, "~G", 123.456));
    }
}
