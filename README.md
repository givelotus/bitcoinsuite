# Bitcoin Suite

This is a collection of libraries, packages and tools to interact with Bitcoin-like blockchains, such as Bitcoin Cash (BCH), eCash (XEC), Lotus (XPI) and Ergon (XRG).

# Run suite

Some configuration is required before running the suite. 
Dont be alarmed if you get an error, this README is here to help! 

Install `cargo make` using `cargo install --force cargo-make`\
Run `cargo make` to build the project and run all the tests\
Met with some errors? :confounded: Try this: :sunglasses:

1. First `sudo apt-get install build-essential libssl-dev protobuf-compiler` 
2. Download [flatbuffers 2.0](https://github.com/google/flatbuffers/releases/tag/v2.0.8)
3. `tar xf '/{insert downloads folder here}/flatbuffers-2.0.8.tar.gz' `
4. Install cmake `sudo apt install cmake`
5. `sudo make install`
6. `cmake -G "Unix Makefiles" -DCMAKE_BUILD_TYPE=Release`
7. Check if flatbuffers is installed `flatc --version`
8. Now go back a directory, `cd ..`
9. Check if protobuf is installed `protoc --version`
10. As flatbuffers is installed, we can delete from our directory to clean up `rm -r flatbuffers-2.0.8/`
11. And finally `cargo make` 

Everything should be working!
Any further questions on troubleshooting, please message @harrygrant125 on Telegram. :keyboard:
