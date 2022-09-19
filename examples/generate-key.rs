use ring::{
    rand,
    signature::{self, KeyPair},
};

fn main() -> anyhow::Result<()> {
    let rng = rand::SystemRandom::new();
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)?;
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;

    let privkey = hex::encode(&pkcs8_bytes);
    let pubkey = hex::encode(key_pair.public_key());

    println!("Private key: {privkey}");
    println!("Public key:  {pubkey}");

    Ok(())
}
