

macro_rules! print_expr {
    ($fmt: literal $(, $($arg:tt)+)?) => {
        println!("{}, {}", $fmt, $(, $($arg)+)?);
    };
}


fn main() {
    let x = 42;
    let d = 32;
    print_expr!( "def, {}, {}", x, d);
}
