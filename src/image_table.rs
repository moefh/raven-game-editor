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
            $( $name: $crate::image_table::ImageRef { uri: concat!("bytes://", $file), bytes: include_bytes!($file) }, )*
        };
    );

    () => ()
}

image_table! {
    pico: "../assets/PicoIcon.png",
    properties: "../assets/PropertiesIcon.png",
    chicken: "../assets/ChickenIcon.png",
    tileset: "../assets/TilesetIcon.png",
    map_data: "../assets/MapIcon.png",
    room: "../assets/RoomIcon.png",
    sprite: "../assets/SpriteIcon.png",
    animation: "../assets/AnimationIcon.png",
    sfx: "../assets/SfxIcon.png",
    mod_data: "../assets/MODicon.png",
    font: "../assets/FwFontIcon.png",
    prop_font: "../assets/FontIcon.png",
}
