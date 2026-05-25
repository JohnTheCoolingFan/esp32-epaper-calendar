macro_rules! make_icons {
    ($($name:ident = $path:literal),+) => {
        $(
            pub static $name : &[u8] = include_bytes!($path);
        )+
    }
}

pub mod weather_icons {

    make_icons!(
        ICON_0 = "icon_data/ICON_0.bin",
        ICON_1 = "icon_data/ICON_1.bin",
        ICON_2 = "icon_data/ICON_2.bin",
        ICON_3 = "icon_data/ICON_3.bin",
        ICON_45 = "icon_data/ICON_45.bin",
        ICON_48 = "icon_data/ICON_48.bin",
        ICON_51_53 = "icon_data/ICON_51_53.bin",
        ICON_55 = "icon_data/ICON_55.bin",
        ICON_56 = "icon_data/ICON_56.bin",
        ICON_57 = "icon_data/ICON_57.bin",
        ICON_61_80 = "icon_data/ICON_61_80.bin",
        ICON_63_81 = "icon_data/ICON_63_81.bin",
        ICON_65_82 = "icon_data/ICON_65_82.bin",
        ICON_66 = "icon_data/ICON_66.bin",
        ICON_67 = "icon_data/ICON_67.bin",
        ICON_71_85 = "icon_data/ICON_71_85.bin",
        ICON_73 = "icon_data/ICON_73.bin",
        ICON_75_86 = "icon_data/ICON_75_86.bin",
        ICON_77 = "icon_data/ICON_77.bin",
        ICON_95 = "icon_data/ICON_95.bin",
        ICON_96_99 = "icon_data/ICON_96_99.bin"
    );

    pub const fn icon_id_to_icon(icon_id: u8) -> &'static [u8] {
        match icon_id {
            0 => ICON_0,
            1 => ICON_1,
            2 => ICON_2,
            3 => ICON_3,
            45 => ICON_45,
            48 => ICON_48,
            51 => ICON_51_53,
            53 => ICON_51_53,
            55 => ICON_55,
            56 => ICON_56,
            57 => ICON_57,
            61 => ICON_61_80,
            80 => ICON_61_80,
            63 => ICON_63_81,
            81 => ICON_63_81,
            65 => ICON_65_82,
            82 => ICON_65_82,
            66 => ICON_66,
            67 => ICON_67,
            71 => ICON_71_85,
            85 => ICON_71_85,
            73 => ICON_73,
            75 => ICON_75_86,
            86 => ICON_75_86,
            77 => ICON_77,
            95 => ICON_95,
            96 => ICON_96_99,
            99 => ICON_96_99,
            _ => ICON_0,
        }
    }
}
