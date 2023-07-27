use rayon::prelude::*;

use crate::{bucket::*, config::*, error::ScoopieError};

pub struct Sync {}

impl Sync {
    pub fn now() -> Result<Vec<SyncStatus>, ScoopieError> {
        Config::read()?.known_buckets().par_iter().map(Bucket::sync_from).collect::<Result<Vec<SyncStatus>, _>>()
    }
}
