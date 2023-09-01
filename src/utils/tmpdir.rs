use std::{
    env,
    fs::{remove_dir_all, DirBuilder},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::error::ScoopieError;

use digest::Digest;
use md5::Md5;

pub struct TempDir(PathBuf);

impl TempDir {
    pub fn build() -> Result<Self, ScoopieError> {
        let registered_tmp_dir =
            env::var("TMP").map_err(|_| ScoopieError::UnableToGetEnvVar(String::from("TMP")))?;

        let dir_name = (|| {
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                .to_le_bytes();

            let suffix = &hex::encode(Md5::digest(seed))[..6];

            format!("tmp_scoopie_{suffix}")
        })();

        let tmp_dir = PathBuf::from(registered_tmp_dir).join(dir_name);

        let _ = DirBuilder::new()
            .recursive(true)
            .create(&tmp_dir)
            .map_err(|_| ScoopieError::Sync(crate::error::SyncError::UnableToMkTmpDir))?;

        Ok(Self(tmp_dir))
    }

    pub fn path(&self) -> PathBuf {
        self.0.to_path_buf()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = remove_dir_all(&self.0);
    }
}
