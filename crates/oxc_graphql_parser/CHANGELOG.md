# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.3](https://github.com/oxc-project/oxc-graphql-parser/compare/oxc-graphql-parser-v0.0.2...oxc-graphql-parser-v0.0.3) - 2026-07-02

### Fixed

- fragment definition bug ([#17](https://github.com/oxc-project/oxc-graphql-parser/pull/17))

## [0.0.2](https://github.com/oxc-project/oxc-graphql-parser/compare/oxc-graphql-parser-v0.0.1...oxc-graphql-parser-v0.0.2) - 2026-07-01

### Fixed

- borrow allocator for oxc_allocator 0.138 new_in API

### Other

- scan whitespace runs in a tight loop ([#12](https://github.com/oxc-project/oxc-graphql-parser/pull/12))
- avoid token clones in the parser ([#13](https://github.com/oxc-project/oxc-graphql-parser/pull/13))
- scan name tokens in a tight loop ([#11](https://github.com/oxc-project/oxc-graphql-parser/pull/11))
- inline lexer iterator next ([#10](https://github.com/oxc-project/oxc-graphql-parser/pull/10))
- inline lexer token completion ([#9](https://github.com/oxc-project/oxc-graphql-parser/pull/9))
- avoid cloning lexer errors ([#8](https://github.com/oxc-project/oxc-graphql-parser/pull/8))
- avoid definition selector allocation ([#7](https://github.com/oxc-project/oxc-graphql-parser/pull/7))
- use byte cursor in lexer ([#5](https://github.com/oxc-project/oxc-graphql-parser/pull/5))
- allocate ast with oxc_allocator ([#4](https://github.com/oxc-project/oxc-graphql-parser/pull/4))
- replace cst with direct ast ([#3](https://github.com/oxc-project/oxc-graphql-parser/pull/3))
- port shared workflows
- add workspace lint config
- move benchmarks to workspace root
- add codspeed benchmarks and rust 2024
- remove parser error screenshot
