#[cfg(test)]
mod tests {
    use clformat::clformat;

    #[test]
    fn it_works() {
        // let x = vec!["ook", "onk", "nork", "nonk"];
        // let y = 32;

        // let x = clformat!("Hello there, ~{ ~A~*~^,~}~%~10,'-D!", x, y);

        // let expected = "Hello there,  ook, nork\n--------32!";
        // assert_eq!(expected, x);

        let dr = "Dr Ponk";
        let x = clformat!("Hello, ~A", dr);

        assert_eq!("Hello, Dr Ponk", x);
    }

    #[test]
    fn iteration() {
        let x = vec!["ook", "onk", "nork", "nonk"];
        let x = clformat!("~{~A~^, ~}", x);

        assert_eq!("ook, onk, nork, nonk", x);
    }

    #[test]
    fn skip_iteration() {
        let x = vec!["ook", "onk", "nork", "nonk"];
        let x = clformat!("~{~A~*~^, ~}", x);

        assert_eq!("ook, nork", x);
    }

    #[test]
    fn decimals() {
        let num = 4200;
        assert_eq!("4200", clformat!("~D", num));
        assert_eq!("4,200", clformat!("~:D", num));

        let num = -4200;
        assert_eq!("-4,200", clformat!("~:D", num));
        assert_eq!("____-4,200", clformat!("~10,'_:D", num));
    }
}
