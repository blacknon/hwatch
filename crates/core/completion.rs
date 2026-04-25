// Copyright (c) 2026 Blacknon. All rights reserved.
// Use of this source code is governed by an MIT license
// that can be found in the LICENSE file.

pub fn handle_completion_request() -> Result<bool, String> {
    match parse_completion_from_cli()? {
        Some(shell) => {
            if !output_completion(&shell) {
                return Err(format!("unknown shell for --completion: {shell}"));
            }
            Ok(true)
        }
        None => Ok(false),
    }
}

fn output_completion(shell: &str) -> bool {
    match shell {
        "bash" => {
            print!(
                "{}",
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/completion/bash/hwatch-completion.bash"
                ))
            );
            true
        }
        "fish" => {
            print!(
                "{}",
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/completion/fish/hwatch.fish"
                ))
            );
            true
        }
        "zsh" => {
            print!(
                "{}",
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/completion/zsh/_hwatch"
                ))
            );
            true
        }
        _ => false,
    }
}

fn parse_completion_from_cli() -> Result<Option<String>, String> {
    let mut args = std::env::args_os();
    let _ = args.next();
    let mut iter = args.peekable();

    while let Some(arg) = iter.next() {
        if arg == "--" {
            break;
        }

        let Some(arg_str) = arg.to_str() else {
            continue;
        };

        if arg_str == "--completion" {
            let Some(value) = iter.next() else {
                return Err("missing value for --completion".to_string());
            };
            return Ok(Some(value.to_string_lossy().into_owned()));
        }

        if let Some(value) = arg_str.strip_prefix("--completion=") {
            return Ok(Some(value.to_string()));
        }
    }

    Ok(None)
}
