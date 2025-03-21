use embedded_graphics::mono_font::MonoTextStyle;
use paste::paste;
use weact_studio_epd::TriColor;

pub type StyleType = MonoTextStyle<'static, TriColor>;

macro_rules! make_styles_color {
    ($color:expr, $color_ident:ident, [$($size:literal),+]) => {
        $(
            paste! {
                #[allow(dead_code)]
                pub const [<STYLE_ $color_ident _ $size>]: StyleType = StyleType::new(&profont::[<PROFONT_ $size _POINT>], $color);
            }
        )+
    }
}

macro_rules! make_styles {
    ([$(($color:expr, $color_ident:ident)),+], $sizes:tt) => {
        $(
            make_styles_color!($color, $color_ident, $sizes);
        )+
    }
}

make_styles!(
    [(TriColor::Black, BLACK), (TriColor::Red, RED)],
    [7, 9, 10, 12, 14, 18, 24]
);
