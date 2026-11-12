# tws2tag

A tool to fetch JSON data about posts downloaded through TWS and save them.

## Usage

To run in normal mode and fetch the API data, run:
```shell
cargo run -- --directory "PATH/TO/TWS/DOWNLOADS" --config-file "./config.conf" --output "PATH/TO/PUT/JSONS"
```
Please note that the output directory will be used to store two more helper files called "archive_file" and "bad_files". 
These are used to keep track of duplicate posts or files which likely shouldn't be in the input folder and require manual
intervention. There will also be a local database created in the working directory.

To run in verification mode, run:
```shell
cargo run -- --directory "PATH/TO/TWS/DOWNLOADS" --config-file "./config.conf" --output "PATH/TO/PUT/JSONS" --verify
```

Verification mode will use the database to verify that the downloaded files match the API's response via MD5 hashes and 
by checking the file name.