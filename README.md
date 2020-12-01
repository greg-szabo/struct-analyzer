# tendermint-struct-analyzer
This package was a weekend project to analyze the public structs exposed in the [tendermint-rs](https://github.com/informalsystems/tendermint-rs)
project.

## Usage
### Download and run
There are no releases for the project. You need to clone this repo and have the Rust compiler set up. In short:
```shell script
git clone https://github.com/greg-szabo/struct-analyzer
cd struct-analyzer
cargo run $HOME/git/informalsystems/tendermint-rs/tendermint/src/
```
Make sure you replace the folder path with your own for the tendermint-rs source code.

Command-line parameters:
* --json - this will only print structs and enums that have some kind of serde serialization/deserialization implemented,
* --output - output the result into a file, instead of the screen,
* --no-header - do not print the draw.io config and the CSV header. (You can possibly concatenate two files with this.)

### Import the output to draw.io
Open https://draw.io and go to `Insert -> Advanced -> CSV...`. Paste the output completely (note that lines starting
with `#` are configuration lines for draw.io and NOT comments). After clicking the `Import` button, the completed
drawing appears.
