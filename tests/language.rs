extern crate rurtle;
mod support;

use rurtle::environ::value::Value::*;


macro_rules! test {
    ($name:ident, $source:expr, $expected:expr) => {
        test!($name, "", $source, $expected);
    };
    ($name:ident, $setup:expr, $source:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let mut environ = support::environ();
            environ.eval_source($setup).expect("Setup gone wrong");
            let result = environ.eval_source($source).unwrap();
            assert_eq!(result, $expected);
        }
    };
}


macro_rules! list {
    ($($v:expr),*) => {
        List(vec![$($v),*])
    }
}


test! {
    test_simple_list,
    "[3 1 4]",
    list!(Number(3.0), Number(1.0), Number(4.0))
}


test! {
    test_list_with_function_at_start,
    "learn add :a :b do return :a + :b end",
    "[add 1 1 3 4]",
    list!(Number(2.0), Number(3.0), Number(4.0))
}


test! {
    test_list_with_function_in_middle,
    "learn add :a :b do return :a + :b end",
    "[1 add 1 1 3]",
    list!(Number(1.0), Number(2.0), Number(3.0))
}


test! {
    test_nested_lists,
    "[[1 2] [3 4]]",
    list!(list!(Number(1.0), Number(2.0)), list!(Number(3.0), Number(4.0)))
}


test! {
    test_function_in_condition,
    "learn eq :a :b do return :a = :b end learn test do if eq 1 1 do return 1 else return 2 end end",
    "test",
    Number(1.0)
}
