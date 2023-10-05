use clformat_macro::clformat;

fn main() {
    let x = "Ook";
    let y = 32;
    let z = 93;

    let x = clformat!("Hello ~A,~%Be ~D!", x, if x == "ook" { y } else { z });
    println!("{}", x);
}
