#![no_main]
use libfuzzer_sys::fuzz_target;
use oxc_graphql_parser::Allocator;
use oxc_graphql_parser::Parser;

// Use completely arbitrary input and a token limit to find cases where the limit
// being reached causes a loop in the parser.
fuzz_target!(|data: &str| {
    let _ = env_logger::try_init();

    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, data).token_limit(500);

    // We expect to have errors--we just want to make sure it does not crash.
    let _tree = parser.parse();
});
