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
        use std::fmt::Write;
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
}
