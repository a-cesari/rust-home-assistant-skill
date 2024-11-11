This function has been re-written with ChatGPT help from this one: https://gist.github.com/matt2005/744b5ef548cc13d88d0569eea65f5e5b


1. Install rust if not already available: curl https://sh.rustup.rs -sSf | sh
2. Install cargo-lambda (https://www.cargo-lambda.info/guide/installation.html) e.g. `pip3 install cargo-lambda`
3. Run: `cargo lambda new ha-alexa-skill`. This will create a folder `ha-alexa-skill`
4. Replace the src folder inside `ha-alexa-skill` with the one from this repository
5. Replace the Cargo.toml file with the one of this repository
6. cargo lambda build --release --arm64 
7. cargo lambda deploy --iam-role XXX where XXX is the lambda basic execution role ARN