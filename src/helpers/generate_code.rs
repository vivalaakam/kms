use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn generate_code() -> String {
    let rng = rand::thread_rng();
    let bytes: Vec<u8> = rng.sample_iter(&Alphanumeric).take(8).collect();

    String::from_utf8(bytes).expect("Found invalid UTF-8")
}

pub fn generate_random() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..32).map(|_| rng.gen()).collect()
}
