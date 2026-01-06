# Architecture for the Linux console tools

## Project organization - workspace
 - Each tool is in its own crate
   + The name of the create shares the name of the bin.
 - Common code goes into the `utils` folder.
 - In the root of the repo Cargo.toml defines a virtual manifest.
 - Write all aoutomation in a Rust in a dedicated crate (check pattern `cargo xtask`)

__See recommendations in [https://matklad.github.io/2021/08/22/large-rust-workspaces.html]__

For recommendations on how to write command line applications follow:
 - [https://rust-cli.github.io/book/tutorial/index.html]
