#![no_main]
use libfuzzer_sys::fuzz_target;
use oxc_graphql_parser::Allocator;
use oxc_graphql_parser::Parser;

fuzz_target!(|data: &str| {
    let _ = env_logger::try_init();

    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, data);
    let _tree = parser.parse();
});
