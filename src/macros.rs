
#[macro_export] macro_rules! enum_as_variant {
    ($a:expr, $b:ident::$c:ident) => { match $a { $b::$c(variant) => variant, _ => panic!("object was not the specified variant") } }
}
