flatc -o src --rust flatbuffers/nng_interface.fbs

printf "#![allow(unused_imports, dead_code, clippy::all)]\n$(cat src/nng_interface_generated.rs)" > src/nng_interface_generated.rs
