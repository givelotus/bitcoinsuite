# Bitcoin Suite

This is a collection of libraries, packages and tools to interact with Bitcoin-like blockchains, such as Bitcoin Cash (BCH), eCash (XEC), Lotus (XPI) and Ergon (XRG).

# Run suite

Some configuration is required before running the suite. 
Dont be alarmed if you get an error, this README is here to help! 

Install `cargo make` using `cargo install --force cargo-make`\
Run `cargo make` to build the project and run all the tests\
Met with some errors? :confounded: Try this: :sunglasses:

First `sudo apt-get install libssl-dev` 
Download [flatbuffers 2.0](https://github.com/google/flatbuffers/releases/tag/v2.0.8)
Next `tar xf '/{insert downloads folder here} /flatbuffers-2.0.8.tar.gz' `
Install cmake `sudo apt install cmake`
Install build-essential `sudo apt install build-essential`
Next `cmake -G "Unix Makefiles" -DCMAKE_BUILD_TYPE=Release`
Next `sudo make install`
Check if flatbuffers is installed `flatc --version`
Now go back a directory, `cd ..`
Install protobuf compiler `sudo apt install -y protobuf-compiler`
Check if protobuf is installed `protoc --version`
As flatbuffers is installed, we can delete from our directory to clean up `rm -r flatbuffers-2.0.8/`
And finally `cargo make` 

Everything should be working!
Any further questions on troubleshooting, please message @harrygrant125 on Telegram. :keyboard:
