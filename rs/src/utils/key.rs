use rsa::RsaPrivateKey;

pub fn generate() -> RsaPrivateKey {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate private key");

    priv_key
}
