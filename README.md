# Bitcoin Suite

This is a collection of libraries, packages and tools to interact with Bitcoin-like blockchains, such as Bitcoin Cash (BCH), eCash (XEC), Lotus (XPI) and Ergon (XRG).

# Run suite

1. Install `cargo make` using `cargo install --force cargo-make`
2. Run `cargo make` to build the project and run all the tests
3. Met with some errors? confounded Try this: sunglasses

1. First `sudo make install`
2. `sudo apt-get install build-essential libssl-dev cmake protobuf-compile`
3. Download [flatbuffers 2.0](https://github.com/google/flatbuffers/releases/tag/v2.0.8)
4. `tar xf '/{insert downloads folder here}/flatbuffers-2.0.8.tar.gz'`
5. `cmake -G "Unix Makefiles" -DCMAKE_BUILD_TYPE=Release`
6. Check if flatbuffers is installed `flatc --version`
7. Now go back a directory, `cd ..`
8. Check if protobuf is installed `protoc --version`
9. As flatbuffers is installed, we can delete from our directory to clean up `rm -r flatbuffers-2.0.8/`
10. And finally `cargo make`

Everything should be working! 
Any further questions on troubleshooting, please message @harrygrant125 on Telegram. keyboard
