fn main() {
    let k = match klap::label_from_envstr("honk/foo=bar ") {
        Ok(k) => k,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    println!("{}:{}", k.key, k.value);
}
