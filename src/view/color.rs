// module
use ncurses::*;
use regex::bytes::{Matches, Regex};
use std::str;

// const
pub const IS_BOLD: i32 = 1001;
pub const IS_REVERSE: i32 = 1007;

// ncurses colorset element const
//     ・DEFAULT ... D (1)
//     ・BLACK   ... K (2)
//     ・RED     ... R (3)
//     ・GREEN   ... G (4)
//     ・YELLOW  ... Y (5)
//     ・BLUE    ... B (6)
//     ・MAGENTA ... M (7)
//     ・CYAN    ... C (8)
//     ・WHITE   ... W (9)
pub const COLOR_ELEMENT_D: i16 = 1; // Default
pub const COLOR_ELEMENT_K: i16 = 2; // Black
pub const COLOR_ELEMENT_R: i16 = 3; // Red
pub const COLOR_ELEMENT_G: i16 = 4; // Green
pub const COLOR_ELEMENT_Y: i16 = 5; // Yellow
pub const COLOR_ELEMENT_B: i16 = 6; // Blue
pub const COLOR_ELEMENT_M: i16 = 7; // Magenta
pub const COLOR_ELEMENT_C: i16 = 8; // Cyan
pub const COLOR_ELEMENT_W: i16 = 9; // White

// ncurses colorset const (color (COLORSET_<Front>_<Back>))
//     ・DEFAULT ... D (1)
//     ・BLACK   ... K (2)
//     ・RED     ... R (3)
//     ・GREEN   ... G (4)
//     ・YELLOW  ... Y (5)
//     ・BLUE    ... B (6)
//     ・MAGENTA ... M (7)
//     ・CYAN    ... C (8)
//     ・WHITE   ... W (9)
// DEFAULT + <Back>
pub const COLORSET_D_D: i16 = 11; // DEFAULT on DEFAULT
pub const COLORSET_D_K: i16 = 12; // DEFAULT on BLACK
pub const COLORSET_D_E: i16 = 13; // DEFAULT on RED
pub const COLORSET_D_G: i16 = 14; // DEFAULT on GREEN
pub const COLORSET_D_Y: i16 = 15; // DEFAULT on YELLOW
pub const COLORSET_D_B: i16 = 16; // DEFAULT on BLUE
pub const COLORSET_D_M: i16 = 17; // DEFAULT on MAGENTA
pub const COLORSET_D_C: i16 = 18; // DEFAULT on CYAN
pub const COLORSET_D_W: i16 = 19; // DEFAULT on WHITE

// Black + <Back>
pub const COLORSET_K_D: i16 = 21; // BLACK on DEFAULT
pub const COLORSET_K_K: i16 = 22; // BLACK on BLACK
pub const COLORSET_K_E: i16 = 23; // BLACK on RED
pub const COLORSET_K_G: i16 = 24; // BLACK on GREEN
pub const COLORSET_K_Y: i16 = 25; // BLACK on YELLOW
pub const COLORSET_K_B: i16 = 26; // BLACK on BLUE
pub const COLORSET_K_M: i16 = 27; // BLACK on MAGENTA
pub const COLORSET_K_C: i16 = 28; // BLACK on CYAN
pub const COLORSET_K_W: i16 = 29; // BLACK on WHITE

// Red + <Back>
pub const COLORSET_R_D: i16 = 31; // RED on DEFAULT
pub const COLORSET_R_K: i16 = 32; // RED on BLACK
pub const COLORSET_R_E: i16 = 33; // RED on RED
pub const COLORSET_R_G: i16 = 34; // RED on GREEN
pub const COLORSET_R_Y: i16 = 35; // RED on YELLOW
pub const COLORSET_R_B: i16 = 36; // RED on BLUE
pub const COLORSET_R_M: i16 = 37; // RED on MAGENTA
pub const COLORSET_R_C: i16 = 38; // RED on CYAN
pub const COLORSET_R_W: i16 = 39; // RED on WHITE

// Green + <Back>
pub const COLORSET_G_D: i16 = 41; // GREEN on DEFAULT
pub const COLORSET_G_K: i16 = 42; // GREEN on BLACK
pub const COLORSET_G_E: i16 = 43; // GREEN on RED
pub const COLORSET_G_G: i16 = 44; // GREEN on GREEN
pub const COLORSET_G_Y: i16 = 45; // GREEN on YELLOW
pub const COLORSET_G_B: i16 = 46; // GREEN on BLUE
pub const COLORSET_G_M: i16 = 47; // GREEN on MAGENTA
pub const COLORSET_G_C: i16 = 48; // GREEN on CYAN
pub const COLORSET_G_W: i16 = 49; // GREEN on WHITE

// Yellow + <Back>
pub const COLORSET_Y_D: i16 = 51; // YELLOW on DEFAULT
pub const COLORSET_Y_K: i16 = 52; // YELLOW on BLACK
pub const COLORSET_Y_E: i16 = 53; // YELLOW on RED
pub const COLORSET_Y_G: i16 = 54; // YELLOW on GREEN
pub const COLORSET_Y_Y: i16 = 55; // YELLOW on YELLOW
pub const COLORSET_Y_B: i16 = 56; // YELLOW on BLUE
pub const COLORSET_Y_M: i16 = 57; // YELLOW on MAGENTA
pub const COLORSET_Y_C: i16 = 58; // YELLOW on CYAN
pub const COLORSET_Y_W: i16 = 59; // YELLOW on WHITE

// Blue + <Back>
pub const COLORSET_B_D: i16 = 61; // BLUE on DEFAULT
pub const COLORSET_B_K: i16 = 62; // BLUE on BLACK
pub const COLORSET_B_E: i16 = 63; // BLUE on RED
pub const COLORSET_B_G: i16 = 64; // BLUE on GREEN
pub const COLORSET_B_Y: i16 = 65; // BLUE on YELLOW
pub const COLORSET_B_B: i16 = 66; // BLUE on BLUE
pub const COLORSET_B_M: i16 = 67; // BLUE on MAGENTA
pub const COLORSET_B_C: i16 = 68; // BLUE on CYAN
pub const COLORSET_B_W: i16 = 69; // BLUE on WHITE

// Magenta + <Back>
pub const COLORSET_M_D: i16 = 71; // MAGENTA on DEFAULT
pub const COLORSET_M_K: i16 = 72; // MAGENTA on BLACK
pub const COLORSET_M_E: i16 = 73; // MAGENTA on RED
pub const COLORSET_M_G: i16 = 74; // MAGENTA on GREEN
pub const COLORSET_M_Y: i16 = 75; // MAGENTA on YELLOW
pub const COLORSET_M_B: i16 = 76; // MAGENTA on BLUE
pub const COLORSET_M_M: i16 = 77; // MAGENTA on MAGENTA
pub const COLORSET_M_C: i16 = 78; // MAGENTA on CYAN
pub const COLORSET_M_W: i16 = 79; // MAGENTA on WHITE

// Cyan + <Back>
pub const COLORSET_C_D: i16 = 81; // CYAN on DEFAULT
pub const COLORSET_C_K: i16 = 82; // CYAN on BLACK
pub const COLORSET_C_E: i16 = 83; // CYAN on RED
pub const COLORSET_C_G: i16 = 84; // CYAN on GREEN
pub const COLORSET_C_Y: i16 = 85; // CYAN on YELLOW
pub const COLORSET_C_B: i16 = 86; // CYAN on BLUE
pub const COLORSET_C_M: i16 = 87; // CYAN on MAGENTA
pub const COLORSET_C_C: i16 = 88; // CYAN on CYAN
pub const COLORSET_C_W: i16 = 89; // CYAN on WHITE

// White + <Back>
pub const COLORSET_W_D: i16 = 91; // WHITE on DEFAULT
pub const COLORSET_W_K: i16 = 92; // WHITE on BLACK
pub const COLORSET_W_E: i16 = 93; // WHITE on RED
pub const COLORSET_W_G: i16 = 94; // WHITE on GREEN
pub const COLORSET_W_Y: i16 = 95; // WHITE on YELLOW
pub const COLORSET_W_B: i16 = 96; // WHITE on BLUE
pub const COLORSET_W_M: i16 = 97; // WHITE on MAGENTA
pub const COLORSET_W_C: i16 = 98; // WHITE on CYAN
pub const COLORSET_W_W: i16 = 99; // WHITE on WHITE

pub fn setup_colorset() {
    start_color();
    use_default_colors();

    // set colors
    init_pair(COLORSET_D_D, -1, -1);
    init_pair(COLORSET_D_K, -1, COLOR_BLACK);
    init_pair(COLORSET_D_E, -1, COLOR_RED);
    init_pair(COLORSET_D_G, -1, COLOR_GREEN);
    init_pair(COLORSET_D_Y, -1, COLOR_YELLOW);
    init_pair(COLORSET_D_B, -1, COLOR_BLUE);
    init_pair(COLORSET_D_M, -1, COLOR_MAGENTA);
    init_pair(COLORSET_D_C, -1, COLOR_CYAN);
    init_pair(COLORSET_D_W, -1, COLOR_WHITE);
    init_pair(COLORSET_K_D, COLOR_BLACK, -1);
    init_pair(COLORSET_K_K, COLOR_BLACK, COLOR_BLACK);
    init_pair(COLORSET_K_E, COLOR_BLACK, COLOR_RED);
    init_pair(COLORSET_K_G, COLOR_BLACK, COLOR_GREEN);
    init_pair(COLORSET_K_Y, COLOR_BLACK, COLOR_YELLOW);
    init_pair(COLORSET_K_B, COLOR_BLACK, COLOR_BLUE);
    init_pair(COLORSET_K_M, COLOR_BLACK, COLOR_MAGENTA);
    init_pair(COLORSET_K_C, COLOR_BLACK, COLOR_CYAN);
    init_pair(COLORSET_K_W, COLOR_BLACK, COLOR_WHITE);
    init_pair(COLORSET_R_D, COLOR_RED, -1);
    init_pair(COLORSET_R_K, COLOR_RED, COLOR_BLACK);
    init_pair(COLORSET_R_E, COLOR_RED, COLOR_RED);
    init_pair(COLORSET_R_G, COLOR_RED, COLOR_GREEN);
    init_pair(COLORSET_R_Y, COLOR_RED, COLOR_YELLOW);
    init_pair(COLORSET_R_B, COLOR_RED, COLOR_BLUE);
    init_pair(COLORSET_R_M, COLOR_RED, COLOR_MAGENTA);
    init_pair(COLORSET_R_C, COLOR_RED, COLOR_CYAN);
    init_pair(COLORSET_R_W, COLOR_RED, COLOR_WHITE);
    init_pair(COLORSET_G_D, COLOR_GREEN, -1);
    init_pair(COLORSET_G_K, COLOR_GREEN, COLOR_BLACK);
    init_pair(COLORSET_G_E, COLOR_GREEN, COLOR_RED);
    init_pair(COLORSET_G_G, COLOR_GREEN, COLOR_GREEN);
    init_pair(COLORSET_G_Y, COLOR_GREEN, COLOR_YELLOW);
    init_pair(COLORSET_G_B, COLOR_GREEN, COLOR_BLUE);
    init_pair(COLORSET_G_M, COLOR_GREEN, COLOR_MAGENTA);
    init_pair(COLORSET_G_C, COLOR_GREEN, COLOR_CYAN);
    init_pair(COLORSET_G_W, COLOR_GREEN, COLOR_WHITE);
    init_pair(COLORSET_Y_D, COLOR_YELLOW, -1);
    init_pair(COLORSET_Y_K, COLOR_YELLOW, COLOR_BLACK);
    init_pair(COLORSET_Y_E, COLOR_YELLOW, COLOR_RED);
    init_pair(COLORSET_Y_G, COLOR_YELLOW, COLOR_GREEN);
    init_pair(COLORSET_Y_Y, COLOR_YELLOW, COLOR_YELLOW);
    init_pair(COLORSET_Y_B, COLOR_YELLOW, COLOR_BLUE);
    init_pair(COLORSET_Y_M, COLOR_YELLOW, COLOR_MAGENTA);
    init_pair(COLORSET_Y_C, COLOR_YELLOW, COLOR_CYAN);
    init_pair(COLORSET_Y_W, COLOR_YELLOW, COLOR_WHITE);
    init_pair(COLORSET_B_D, COLOR_BLUE, -1);
    init_pair(COLORSET_B_K, COLOR_BLUE, COLOR_BLACK);
    init_pair(COLORSET_B_E, COLOR_BLUE, COLOR_RED);
    init_pair(COLORSET_B_G, COLOR_BLUE, COLOR_GREEN);
    init_pair(COLORSET_B_Y, COLOR_BLUE, COLOR_YELLOW);
    init_pair(COLORSET_B_B, COLOR_BLUE, COLOR_BLUE);
    init_pair(COLORSET_B_M, COLOR_BLUE, COLOR_MAGENTA);
    init_pair(COLORSET_B_C, COLOR_BLUE, COLOR_CYAN);
    init_pair(COLORSET_B_W, COLOR_BLUE, COLOR_WHITE);
    init_pair(COLORSET_M_D, COLOR_MAGENTA, -1);
    init_pair(COLORSET_M_K, COLOR_MAGENTA, COLOR_BLACK);
    init_pair(COLORSET_M_E, COLOR_MAGENTA, COLOR_RED);
    init_pair(COLORSET_M_G, COLOR_MAGENTA, COLOR_GREEN);
    init_pair(COLORSET_M_Y, COLOR_MAGENTA, COLOR_YELLOW);
    init_pair(COLORSET_M_B, COLOR_MAGENTA, COLOR_BLUE);
    init_pair(COLORSET_M_M, COLOR_MAGENTA, COLOR_MAGENTA);
    init_pair(COLORSET_M_C, COLOR_MAGENTA, COLOR_CYAN);
    init_pair(COLORSET_M_W, COLOR_MAGENTA, COLOR_WHITE);
    init_pair(COLORSET_C_D, COLOR_CYAN, -1);
    init_pair(COLORSET_C_K, COLOR_CYAN, COLOR_BLACK);
    init_pair(COLORSET_C_E, COLOR_CYAN, COLOR_RED);
    init_pair(COLORSET_C_G, COLOR_CYAN, COLOR_GREEN);
    init_pair(COLORSET_C_Y, COLOR_CYAN, COLOR_YELLOW);
    init_pair(COLORSET_C_B, COLOR_CYAN, COLOR_BLUE);
    init_pair(COLORSET_C_M, COLOR_CYAN, COLOR_MAGENTA);
    init_pair(COLORSET_C_C, COLOR_CYAN, COLOR_CYAN);
    init_pair(COLORSET_C_W, COLOR_CYAN, COLOR_WHITE);
    init_pair(COLORSET_W_D, COLOR_WHITE, -1);
    init_pair(COLORSET_W_K, COLOR_WHITE, COLOR_BLACK);
    init_pair(COLORSET_W_E, COLOR_WHITE, COLOR_RED);
    init_pair(COLORSET_W_G, COLOR_WHITE, COLOR_GREEN);
    init_pair(COLORSET_W_Y, COLOR_WHITE, COLOR_YELLOW);
    init_pair(COLORSET_W_B, COLOR_WHITE, COLOR_BLUE);
    init_pair(COLORSET_W_M, COLOR_WHITE, COLOR_MAGENTA);
    init_pair(COLORSET_W_C, COLOR_WHITE, COLOR_CYAN);
    init_pair(COLORSET_W_W, COLOR_WHITE, COLOR_WHITE);
}

// StructでANSIカラーコードと対象の文字列を取得して、それを利用するようにしないといかん
// ansiには事前にNcursesのカラーコードを設定する必要があるけど、offができないので事前にデフォルトカラーの設定を0とかで定義しておく必要があると思う。
// そのための設計が必要！
struct Data {
    ansi: i32,
    data: String,
}

pub const ANSI_RE: &str =
    r"[\x1b\x9b]\[[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]";

lazy_static! {
    pub static ref ANSI_REGEX: Regex = Regex::new(ANSI_RE).unwrap();
}

fn get_ansi_iter(text: &str) -> Matches {
    let text_bytes = text.as_bytes();
    return ANSI_REGEX.find_iter(text_bytes);
}

fn get_color(_ansi_data: &str) -> i32 {
    // result (Color Pair code)
    let result = match _ansi_data {
        "\u{1b}[0m" => 1000,  // reset code
        "\u{1b}[1m" => 1001,  // Bold on
        "\u{1b}[4m" => 1004,  // Underline on
        "\u{1b}[5m" => 1005,  // Blink on
        "\u{1b}[7m" => 1007,  // Reverse on
        "\u{1b}[30m" => 1030, // foreground black
        "\u{1b}[31m" => 1031, // foreground red
        "\u{1b}[32m" => 1032, // foreground green
        "\u{1b}[33m" => 1033, // foreground yellow
        "\u{1b}[34m" => 1034, // foreground blue
        "\u{1b}[35m" => 1035, // foreground magenta
        "\u{1b}[36m" => 1036, // foreground cyan
        "\u{1b}[37m" => 1037, // foreground white
        "\u{1b}[40m" => 1040, // background black
        "\u{1b}[41m" => 1041, // background red
        "\u{1b}[42m" => 1042, // background green
        "\u{1b}[43m" => 1043, // background yellow
        "\u{1b}[44m" => 1044, // background blue
        "\u{1b}[45m" => 1045, // background magenta
        "\u{1b}[46m" => 1046, // background cyan
        "\u{1b}[47m" => 1047, // background white
        _ => 9999,            // Not working
    };

    return result;
}

pub fn parse(text: &str) {
    // ResultとなるVectorの宣言(後で！)
    // let mut text_vec = Vec::new();
    // text_vec.push(1);
    // text_vec.push(2);

    // 組み合わせは、tuple or structにすればいけるのでは？？

    let _parsed: Vec<_> = get_ansi_iter(&text)
        .map(|m| (m.start(), m.end(), m.as_bytes()))
        .collect();

    let mut _result: Vec<Data> = vec![];

    // ANSIの有無
    let mut _count = 0;
    let mut _start = 0;

    let mut _ansi_code = 1000;
    if _parsed.len() > _start {
        for _ansi in &_parsed {
            let _ansi_code_start = _ansi.0;
            let _ansi_code_end = _ansi.1;
            let _ansi_code_data = str::from_utf8(&_ansi.2).unwrap();

            // _ansiの開始位置より_startが小さい場合、直前のANSIcolorが適用される
            if _ansi_code_start > _start {
                let _data = Data {
                    ansi: _ansi_code,
                    data: text[_start.._ansi_code_start].to_string(),
                };
                _result.push(_data);
            }
            // println!("{:?}", &text[_ansi_code_start.._ansi_code_end]);
            _ansi_code = get_color(_ansi_code_data);

            _start = _ansi_code_end;
        }
        _count += 1;

        if _start < text.len() {
            let _data = Data {
                ansi: _ansi_code,
                data: text[_start..].to_string(),
            };
            _result.push(_data);
        }
    } else {
        let _data = Data {
            ansi: _ansi_code,
            data: text.to_string(),
        };
        _result.push(_data);
    }

    for _d in &_result {
        println!("ansi: {:?}", _d.ansi);
        println!("data: {:?}", _d.data);
    }
}
