use std::{env, time, io::{stdin, Read, Write}};
use ring::signature::Ed25519KeyPair;

fn current_timestamp() -> String {
    let current_time = time::SystemTime::now();
    let timestamp = current_time.duration_since(time::SystemTime::UNIX_EPOCH).unwrap();

    format!("{}", timestamp.as_secs())
}

fn main() -> anyhow::Result<()> {
    let mut args = env::args().skip(1);
    let raw_key = args.next().ok_or_else(|| anyhow::anyhow!("Usage: sign privkey [timestamp]"))?;
    let timestamp = args.next().unwrap_or_else(current_timestamp);

    let privkey = hex::decode(raw_key.as_bytes())?;
    let keypair = Ed25519KeyPair::from_pkcs8(privkey.as_slice())?;

    let mut msg = Vec::new();
    write!(&mut msg, "{}", timestamp).ok();

    stdin().read_to_end(&mut msg)?;

    let signature = keypair.sign(msg.as_slice());

    println!("X-Signature-Timestamp: {}", timestamp);
    println!("X-Signature-Ed25519: {}", hex::encode(signature));

    Ok(())
}
