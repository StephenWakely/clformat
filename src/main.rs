use clformat_macro::clformat;

trait IndexedDisplay {
    fn print(&self, index: usize);
}

fn main() {
    let x = vec!["ook", "onk", "nork", "nonk"];
    let y = 32;

    let x = clformat!("Hello ~^ ~{ ~A~*~^,~}~%Be ~D!", x, y);
    println!("{}", x);
}
