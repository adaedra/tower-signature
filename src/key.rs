use std::sync::Arc;

pub struct Key(Arc<Vec<u8>>);

impl Key {
    pub fn from(data: &[u8]) -> Result<Key, hex::FromHexError> {
        let buf = hex::decode(data)?;
        Ok(Key(Arc::new(buf)))
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl Clone for Key {
    fn clone(&self) -> Self {
        Key(self.0.clone())
    }
}
