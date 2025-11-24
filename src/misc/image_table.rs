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

    // buttons
    pen: "../../assets/PenIcon.png",
    pencil_fg: "../../assets/PencilFgIcon.png",
    pencil_bg: "../../assets/PencilBgIcon.png",
    fill: "../../assets/FillIcon.png",
    select: "../../assets/SelRectIcon.png",
    v_flip: "../../assets/VFlipIcon.png",
    h_flip: "../../assets/HFlipIcon.png",
    grid: "../../assets/GridIcon.png",
    transparency: "../../assets/TransparencyIcon.png",
    layer_fg: "../../assets/TilesFgIcon.png",
    layer_bg: "../../assets/TilesBgIcon.png",
    layer_fx: "../../assets/EffectsIcon.png",
    layer_clip: "../../assets/CollisionIcon.png",
    screen: "../../assets/ScreenIcon.png",
    log: "../../assets/LogIcon.png",

    // assets
    tileset: "../../assets/TilesetIcon.png",
    map_data: "../../assets/MapIcon.png",
    room: "../../assets/RoomIcon.png",
    sprite: "../../assets/SpriteIcon.png",
    animation: "../../assets/AnimationIcon.png",
    sfx: "../../assets/SfxIcon.png",
    mod_data: "../../assets/MODicon.png",
    font: "../../assets/FwFontIcon.png",
    prop_font: "../../assets/FontIcon.png",
}
