use sha2::{Sha256, Digest};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct DataAssetHash {
    bytes: [u8; 32],
}

impl DataAssetHash {
    pub const EMPTY: DataAssetHash = DataAssetHash { bytes: [0; 32] };
}

pub struct DataAssetHasher {
    hash: Sha256,
    include_ids: bool,
}

impl DataAssetHasher {
    pub fn new() -> Self {
        DataAssetHasher {
            hash: sha2::Sha256::new(),
            include_ids: true,
        }
    }

    fn new_excluding_ids() -> Self {
        DataAssetHasher {
            hash: sha2::Sha256::new(),
            include_ids: false,
        }
    }

    pub fn finalize(self) -> DataAssetHash {
        let hash = self.hash.finalize();
        DataAssetHash { bytes: hash.0 }
    }

    pub fn add_asset_id(&mut self, id: super::DataAssetId) {
        if self.include_ids {
            self.add_u32(id.id);
        }
    }

    pub fn add_data_asset(&mut self, asset: &super::DataAsset) {
        self.add_str(asset.asset_type.name());
        self.add_asset_id(asset.id);
        self.add_str(&asset.name);
    }

    pub fn add_str(&mut self, str: &str) {
        self.hash.update(str.as_bytes());
        self.add_u8(0);
    }

    pub fn add_u8(&mut self, n: u8) {
        self.hash.update([n]);
    }

    pub fn add_u16(&mut self, n: u16) {
        self.hash.update([
            ((n >> 8) & 0xff) as u8,
            ( n       & 0xff) as u8
        ]);
    }

    pub fn add_u32(&mut self, n: u32) {
        self.hash.update([
            ((n >> 24) & 0xff) as u8,
            ((n >> 16) & 0xff) as u8,
            ((n >>  8) & 0xff) as u8,
            ( n        & 0xff) as u8
        ]);
    }

    pub fn add_bool(&mut self, n: bool) {
        self.add_u8(if n { 1 } else { 0 });
    }

    pub fn add_i8 (&mut self, n: i8 ) { self.add_u8 (n as u8 ); }
    pub fn add_i16(&mut self, n: i16) { self.add_u16(n as u16); }
    pub fn add_i32(&mut self, n: i32) { self.add_u32(n as u32); }

    pub fn add_bytes(&mut self, bytes: &[u8]) {
        self.hash.update(bytes);
    }

    pub fn add_i16_slice(&mut self, s: &[i16]) {
        for &n in s {
            self.add_i16(n);
        }
    }
}

pub trait HashedDataAsset {
    fn data_hash(&self, hasher: &mut DataAssetHasher);
}

pub fn calc_asset_hash<A: HashedDataAsset>(asset: &A) -> DataAssetHash {
    let mut hasher = DataAssetHasher::new();
    asset.data_hash(&mut hasher);
    hasher.finalize()
}

pub fn calc_asset_data_hash<A: HashedDataAsset>(asset: &A) -> DataAssetHash {
    let mut hasher = DataAssetHasher::new_excluding_ids();
    asset.data_hash(&mut hasher);
    hasher.finalize()
}

pub fn calc_store_tables_hash(tables: &super::DataStoreTables) -> DataAssetHash {
    let mut hasher = DataAssetHasher::new();
    tables.data_hash(&mut hasher);
    hasher.finalize()
}
