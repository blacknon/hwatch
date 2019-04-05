// module
use ncurses::*;
use regex::bytes::{Matches, Regex};
use std::str;

// const
pub const IS_BOLD: i32 = 1001;
pub const IS_REVERSE: i32 = 1007;
pub const COLORSET_DEFAULT: i32 = 0;

// const (color (COLORSET_<Front>_<Back>))
//     ・DEFAULT ... D
//     ・BLACK ... K
//     ・RED ... R
//     ・GREEN ... G
//     ・YELLOW ... Y
//     ・BLUE ... B
//     ・MAGENTA ... M
//     ・CYAN ... C
//     ・WHITE ... W
pub const COLORSET_D_D: i32 = 1; // DEFAULT on DEFAULT
pub const COLORSET_D_K: i32 = 2; // DEFAULT on BLACK
pub const COLORSET_D_E: i32 = 3; // DEFAULT on RED
pub const COLORSET_D_G: i32 = 4; // DEFAULT on GREEN
pub const COLORSET_D_Y: i32 = 5; // DEFAULT on YELLOW
pub const COLORSET_D_B: i32 = 6; // DEFAULT on BLUE
pub const COLORSET_D_M: i32 = 7; // DEFAULT on MAGENTA
pub const COLORSET_D_C: i32 = 8; // DEFAULT on CYAN
pub const COLORSET_D_W: i32 = 9; // DEFAULT on WHITE
pub const COLORSET_K_D: i32 = 10; // BLACK on DEFAULT
pub const COLORSET_K_K: i32 = 11; // BLACK on BLACK
pub const COLORSET_K_E: i32 = 12; // BLACK on RED
pub const COLORSET_K_G: i32 = 13; // BLACK on GREEN
pub const COLORSET_K_Y: i32 = 14; // BLACK on YELLOW
pub const COLORSET_K_B: i32 = 15; // BLACK on BLUE
pub const COLORSET_K_M: i32 = 16; // BLACK on MAGENTA
pub const COLORSET_K_C: i32 = 17; // BLACK on CYAN
pub const COLORSET_K_W: i32 = 18; // BLACK on WHITE
pub const COLORSET_E_D: i32 = 19; // RED on DEFAULT
pub const COLORSET_E_K: i32 = 20; // RED on BLACK
pub const COLORSET_E_E: i32 = 21; // RED on RED
pub const COLORSET_E_G: i32 = 22; // RED on GREEN
pub const COLORSET_E_Y: i32 = 23; // RED on YELLOW
pub const COLORSET_E_B: i32 = 24; // RED on BLUE
pub const COLORSET_E_M: i32 = 25; // RED on MAGENTA
pub const COLORSET_E_C: i32 = 26; // RED on CYAN
pub const COLORSET_E_W: i32 = 27; // RED on WHITE
pub const COLORSET_G_D: i32 = 28; // GREEN on DEFAULT
pub const COLORSET_G_K: i32 = 29; // GREEN on BLACK
pub const COLORSET_G_E: i32 = 30; // GREEN on RED
pub const COLORSET_G_G: i32 = 31; // GREEN on GREEN
pub const COLORSET_G_Y: i32 = 32; // GREEN on YELLOW
pub const COLORSET_G_B: i32 = 33; // GREEN on BLUE
pub const COLORSET_G_M: i32 = 34; // GREEN on MAGENTA
pub const COLORSET_G_C: i32 = 35; // GREEN on CYAN
pub const COLORSET_G_W: i32 = 36; // GREEN on WHITE
pub const COLORSET_Y_D: i32 = 37; // YELLOW on DEFAULT
pub const COLORSET_Y_K: i32 = 38; // YELLOW on BLACK
pub const COLORSET_Y_E: i32 = 39; // YELLOW on RED
pub const COLORSET_Y_G: i32 = 40; // YELLOW on GREEN
pub const COLORSET_Y_Y: i32 = 41; // YELLOW on YELLOW
pub const COLORSET_Y_B: i32 = 42; // YELLOW on BLUE
pub const COLORSET_Y_M: i32 = 43; // YELLOW on MAGENTA
pub const COLORSET_Y_C: i32 = 44; // YELLOW on CYAN
pub const COLORSET_Y_W: i32 = 45; // YELLOW on WHITE
pub const COLORSET_B_D: i32 = 46; // BLUE on DEFAULT
pub const COLORSET_B_K: i32 = 47; // BLUE on BLACK
pub const COLORSET_B_E: i32 = 48; // BLUE on RED
pub const COLORSET_B_G: i32 = 49; // BLUE on GREEN
pub const COLORSET_B_Y: i32 = 50; // BLUE on YELLOW
pub const COLORSET_B_B: i32 = 51; // BLUE on BLUE
pub const COLORSET_B_M: i32 = 52; // BLUE on MAGENTA
pub const COLORSET_B_C: i32 = 53; // BLUE on CYAN
pub const COLORSET_B_W: i32 = 54; // BLUE on WHITE
pub const COLORSET_M_D: i32 = 55; // MAGENTA on DEFAULT
pub const COLORSET_M_K: i32 = 56; // MAGENTA on BLACK
pub const COLORSET_M_E: i32 = 57; // MAGENTA on RED
pub const COLORSET_M_G: i32 = 58; // MAGENTA on GREEN
pub const COLORSET_M_Y: i32 = 59; // MAGENTA on YELLOW
pub const COLORSET_M_B: i32 = 60; // MAGENTA on BLUE
pub const COLORSET_M_M: i32 = 61; // MAGENTA on MAGENTA
pub const COLORSET_M_C: i32 = 62; // MAGENTA on CYAN
pub const COLORSET_M_W: i32 = 63; // MAGENTA on WHITE
pub const COLORSET_C_D: i32 = 64; // CYAN on DEFAULT
pub const COLORSET_C_K: i32 = 65; // CYAN on BLACK
pub const COLORSET_C_E: i32 = 66; // CYAN on RED
pub const COLORSET_C_G: i32 = 67; // CYAN on GREEN
pub const COLORSET_C_Y: i32 = 68; // CYAN on YELLOW
pub const COLORSET_C_B: i32 = 69; // CYAN on BLUE
pub const COLORSET_C_M: i32 = 70; // CYAN on MAGENTA
pub const COLORSET_C_C: i32 = 71; // CYAN on CYAN
pub const COLORSET_C_W: i32 = 72; // CYAN on WHITE
pub const COLORSET_W_D: i32 = 73; // WHITE on DEFAULT
pub const COLORSET_W_K: i32 = 74; // WHITE on BLACK
pub const COLORSET_W_E: i32 = 75; // WHITE on RED
pub const COLORSET_W_G: i32 = 76; // WHITE on GREEN
pub const COLORSET_W_Y: i32 = 77; // WHITE on YELLOW
pub const COLORSET_W_B: i32 = 78; // WHITE on BLUE
pub const COLORSET_W_M: i32 = 79; // WHITE on MAGENTA
pub const COLORSET_W_C: i32 = 80; // WHITE on CYAN
pub const COLORSET_W_W: i32 = 81; // WHITE on WHITE

pub fn init_colorset() {
    start_color();
    use_default_colors();
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
