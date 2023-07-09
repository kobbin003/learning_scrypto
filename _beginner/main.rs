fn main() {
    let mut x = 42;
    let y = &mut x;
    // *y += 1;
    println!("x = {}", *y + 1); // prints "x = 43"
                                // println!("hello world");
}

// println!(main());
// main();
