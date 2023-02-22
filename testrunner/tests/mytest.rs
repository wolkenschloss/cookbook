use std::fmt::Arguments;

fn main() {
    use libtest_mimic::{Arguments, Trial};

    let args = Arguments::from_args();

    let tests = vec![
        Trial::test("first test with space", move || Ok(())),
        Trial::test("second test with space", move || Err("Woops".into())),
    ];

// Run all tests and exit the application appropriatly.
    libtest_mimic::run(&args, tests).exit();
}