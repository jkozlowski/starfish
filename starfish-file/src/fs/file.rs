use crate::fs::Error;

pub struct File {
    // File close is blocking by default, need my own type   
}

impl File {
    pub async fn truncate(&self, length: u64) -> Result<(), Error> {
        unimplemented!();
    }
}

impl Drop for File {
    fn drop(&mut self) {
        
    }
}