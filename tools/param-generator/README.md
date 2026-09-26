# Param Generator

Generates rust structs from specified paramdef XML files.
This tool ignores any paramdef fields with a maximum version specified since we assume we're always building for the latest version of the game.

To rebuild `crates/eldenring`'s param.rs, invoke the following from the repo's root:
`$ cargo run --bin param-generator -- --input tools/param-generator/params/eldenring --output crates/eldenring/src/param/generated.rs`

To rebuild `crates/nightreign`'s param.rs, invoke the following from the repo's root:
`$ cargo run --bin param-generator -- --input tools/param-generator/params/nightreign --output crates/nightreign/src/generated.rs`

To rebuild `crates/darksouls3`'s param.rs, invoke the following from the repo's root:
`$ cargo run --bin param-generator -- --input tools/param-generator/params/darksouls3 --output crates/darksouls3/src/param/generated.rs`

To rebuild `crates/sekiro`'s param.rs, invoke the following from the repo's root:
`$ cargo run --bin param-generator -- --input tools/param-generator/params/sekiro --output crates/sekiro/src/param/generated.rs`

To rebuild the Dark Souls II: Scholar of the First Sin params, invoke the following from the repo's root:
`$ cargo run -p param-generator --target x86_64-unknown-linux-gnu -- --input tools/param-generator/params/darksouls2 --output tools/param-generator/generated/darksouls2/param.rs`

There is no `darksouls2` crate yet, so this output is not compiled by the workspace. It type-checks
on its own with `rustc --edition 2024 --crate-type lib --emit=metadata <file>`. The DS2S paramdefs
come from [Smithbox](https://github.com/vawser/Smithbox)'s `src/Smithbox.Data/Assets/PARAM/DS2S/Defs`
and are covered by the MIT licence in `params/darksouls2/LICENSE`.

(`--bin param-generator` fails because this tool is not a default workspace member, hence `-p`.
`--target` is only needed on hosts without the MSVC linker, since `.cargo/config.toml` defaults to
`x86_64-pc-windows-msvc`.)

Make sure to run a round of rustfmt on the output, otherwise the output will not comply with the enforced styleguide.

## C Output

The generator can also produce structs in C header format, for tools like Ghidra
that use that as a common definition format. Just pass `--format=c` to a
command.
