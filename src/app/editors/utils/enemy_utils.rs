use crate::data_store::{
    EnemyType,
};

#[derive(Clone, Copy, PartialEq)]
pub enum EnemyTypeSel {
    Walker,
    Chiller,
    Hopper,
    Floater,
    Other,
}

impl EnemyTypeSel {
    pub fn text(&self) -> &'static str {
        match self {
            EnemyTypeSel::Walker  => { "walker" }
            EnemyTypeSel::Chiller => { "chiller" }
            EnemyTypeSel::Hopper  => { "hopper" }
            EnemyTypeSel::Floater => { "floater" }
            EnemyTypeSel::Other   => { "other..." }
        }
    }
}

impl From<EnemyType> for EnemyTypeSel {
    fn from(value: EnemyType) -> Self {
        match value {
            EnemyType::Walker   => { EnemyTypeSel::Walker }
            EnemyType::Chiller  => { EnemyTypeSel::Chiller }
            EnemyType::Hopper   => { EnemyTypeSel::Hopper }
            EnemyType::Floater  => { EnemyTypeSel::Floater }
            EnemyType::Other(_) => { EnemyTypeSel::Other }
        }
    }
}

impl From<EnemyTypeSel> for EnemyType {
    fn from(value: EnemyTypeSel) -> Self {
        match value {
            EnemyTypeSel::Walker   => { EnemyType::Walker }
            EnemyTypeSel::Chiller  => { EnemyType::Chiller }
            EnemyTypeSel::Hopper   => { EnemyType::Hopper }
            EnemyTypeSel::Floater  => { EnemyType::Floater }
            EnemyTypeSel::Other    => { EnemyType::Other(4) }
        }
    }
}
