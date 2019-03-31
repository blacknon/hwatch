// module
use regex::bytes::{Matches, Regex};
use std::str;

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
