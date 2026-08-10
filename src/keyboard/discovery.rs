use std::{
    error::Error,
    path::PathBuf,
};

pub fn find_keyboards() -> Result<Vec<PathBuf>, Box<dyn Error>> {
    Ok(vec![
        PathBuf::from("/dev/input/event3"),
    ])
}