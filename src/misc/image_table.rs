use std::sync::LazyLock;

use crate::data_asset::{DataAssetType, Tileset};
use crate::image::{StaticImageId, StaticImageData, StaticImageStore};

const TILE_SIZE: u32 = Tileset::TILE_SIZE;

pub struct StaticImages {
    fx_tiles_id: StaticImageId,
    store: StaticImageStore,
}

impl StaticImages {
    pub fn fx_tiles(&self) -> &StaticImageData { self.image(self.fx_tiles_id) }

    fn image(&self, id: StaticImageId) -> &StaticImageData {
        self.store.get(id).unwrap()
    }
}

pub static STATIC_IMAGES: LazyLock<StaticImages> = LazyLock::new(|| {
    let mut store = StaticImageStore::new();
    let fx_tiles_id = store.load_image("effects tiles", TILE_SIZE, TILE_SIZE, include_bytes!("../../assets/EffectsBitmap.png"));
    StaticImages {
        store,
        fx_tiles_id,
    }
});

pub fn get_asset_type_image(asset_type: DataAssetType) -> ImageRef {
    match asset_type {
        DataAssetType::Tileset => IMAGE_REFS.tileset,
        DataAssetType::MapData => IMAGE_REFS.map_data,
        DataAssetType::Room => IMAGE_REFS.room,
        DataAssetType::Sprite => IMAGE_REFS.sprite,
        DataAssetType::PalSprite => IMAGE_REFS.pal_sprite,
        DataAssetType::SpriteAnimation => IMAGE_REFS.animation,
        DataAssetType::Sfx => IMAGE_REFS.sfx,
        DataAssetType::ModData => IMAGE_REFS.mod_data,
        DataAssetType::Font => IMAGE_REFS.font,
        DataAssetType::PropFont => IMAGE_REFS.prop_font,
    }
}

#[macro_export]
macro_rules! include_ref_image {
    ($image_ref:expr $(,)?) => {
        egui::ImageSource::Bytes {
            uri: ::std::borrow::Cow::Borrowed($image_ref.uri),
            bytes: egui::load::Bytes::Static($image_ref.bytes),
        }
    };
}

#[macro_export]
macro_rules! image_table {
    ( $( $name:ident: $file:literal ),+ $(,)? ) => (
        pub struct ImageRef {
            pub uri: &'static str,
            pub bytes: &'static [u8],
        }

        #[allow(dead_code)]
        pub struct ImageTable {
            $( pub $name: egui::ImageSource<'static>, )*
        }

        #[allow(dead_code)]
        pub struct ImageRefsTable {
            $( pub $name: ImageRef, )*
        }

        pub const IMAGES: ImageTable = ImageTable {
            $( $name: egui::include_image!($file), )*
        };

        pub const IMAGE_REFS: ImageRefsTable = ImageRefsTable {
            $( $name: $crate::misc::image_table::ImageRef { uri: concat!("bytes://", $file), bytes: include_bytes!($file) }, )*
        };
    );

    () => ()
}

image_table! {
    // menu
    pico: "../../assets/PicoIcon.png",
    new: "../../assets/NewIcon.png",
    open: "../../assets/OpenIcon.png",
    save: "../../assets/SaveIcon.png",
    properties: "../../assets/PropertiesIcon.png",
    import: "../../assets/ImportIcon.png",
    export: "../../assets/ExportIcon.png",
    chicken: "../../assets/ChickenIcon.png",
    trash: "../../assets/TrashIcon.png",
    add: "../../assets/AddIcon.png",
    undo: "../../assets/UndoIcon.png",
    cut: "../../assets/CutIcon.png",
    copy: "../../assets/CopyIcon.png",
    paste: "../../assets/PasteIcon.png",
    info: "../../assets/InfoIcon.png",
    header: "../../assets/HeaderIcon.png",
    close: "../../assets/Close.png",
    maximize: "../../assets/MaximizeIcon.png",
    un_maximize: "../../assets/UnmaximizeIcon.png",
    blank: "../../assets/BlankIcon.png",

    // buttons
    pen: "../../assets/PenIcon.png",
    pencil_fg: "../../assets/PencilFgIcon.png",
    pencil_bg: "../../assets/PencilBgIcon.png",
    fill: "../../assets/FillIcon.png",
    select: "../../assets/SelRectIcon.png",
    v_flip: "../../assets/VFlipIcon.png",
    h_flip: "../../assets/HFlipIcon.png",
    arrow_up: "../../assets/ArrowUpIcon.png",
    arrow_down: "../../assets/ArrowDownIcon.png",
    arrow_left: "../../assets/ArrowLeftIcon.png",
    arrow_right: "../../assets/ArrowRightIcon.png",
    rot_cw: "../../assets/CW90.png",
    rot_ccw: "../../assets/CCW90.png",
    grid: "../../assets/GridIcon.png",
    transparency: "../../assets/TransparencyIcon.png",
    layer_fg: "../../assets/TilesFgIcon.png",
    layer_bg: "../../assets/TilesBgIcon.png",
    layer_fx: "../../assets/EffectsIcon.png",
    layer_parallax: "../../assets/ParallaxIcon.png",
    screen: "../../assets/ScreenIcon.png",
    log: "../../assets/LogIcon.png",

    // assets
    tileset: "../../assets/TilesetIcon.png",
    map_data: "../../assets/MapIcon.png",
    room: "../../assets/RoomIcon.png",
    sprite: "../../assets/SpriteIcon.png",
    pal_sprite: "../../assets/PalSpriteIcon.png",
    animation: "../../assets/AnimationIcon.png",
    sfx: "../../assets/SfxIcon.png",
    mod_data: "../../assets/MODicon.png",
    font: "../../assets/FwFontIcon.png",
    prop_font: "../../assets/FontIcon.png",
}
