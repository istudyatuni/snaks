#![allow(non_upper_case_globals)]

macro_rules! strings {
    () => {};
    ($($name:ident = $s:literal),* $(,)?) => {
        $(pub const $name: &str = $s;)*
    };
}

pub mod tr {
    pub mod widgets {
        pub mod app {
            strings! { title = "Snake Game" }
        }
        pub mod achivements {
            strings! {
                achivements = "Achivements",
                achivements_on = "Achivements on",
            }
        }
        pub mod debug {
            strings! {
                block_size = "Block size",
                field_size = "Field size",
                food = "Food",
                snake_head = "Snake head",
                snake_direction = "Snake direction",
                fps = "FPS (snake / ui / event)",
            }
        }
        pub mod difficulty {
            strings! {
                select = "Select difficulty",
                press = "Press",
                to_cancel = "to cancel",
                to_select = "to select",
                game_restart = "Game will restart",
            }
        }
        pub mod info {
            strings! {
                score = "Score",
                fail = "Game Over",
                win = "Win",
            }
            pub use super::super::common::{difficulty, pause};
        }
    }
    pub mod keybind {
        strings! {
            r#move = "Move",
            select = "Select",
            submit = "Submit",
            cancel = "Cancel",
            resume = "Resume",
            achivements_by_user = "Show achivements by user",
            achivements_summary = "Show achivements summary",
            restart = "Restart",
            debug = "Debug",
            quit = "Quit",
        }
        pub use super::common::{difficulty, pause};
    }
    mod common {
        strings! {
            pause = "Pause",
            difficulty = "Difficulty",
        }
    }
}
