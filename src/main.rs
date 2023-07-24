use dotenv::dotenv;
use std::env;
use 

fn main() {
    dotenv().ok();

    for (key, value) in env::vars() {
        println!("{}: {}", key, value);
    }

    println!("Single env::var= {:?}", env::var("CLIENT_ID").unwrap());
}
