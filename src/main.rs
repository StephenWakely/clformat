use clformat_macro::clformat;

trait IndexedDisplay {
    fn print(&self, index: usize);
}

fn main() {
    let x = vec!["ook", "onk", "nork", "nonk"];
    let y = 32;

    let x = clformat!("Hello there, ~{ ~A~*~^,~}~%~10,'-D!", x, y);
    println!("{}", x);
}
