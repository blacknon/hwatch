use std::cmp;

use view::watch::Watch;

// watch type diff
pub fn watch_diff(mut watch: Watch, before_output: String, after_output: String) {
    // before and after output to vector
    let mut before_output_collect: Vec<&str> = before_output.lines().collect();
    let mut after_output_collect: Vec<&str> = after_output.lines().collect();

    // get max line before and after output
    let max_line = cmp::max(before_output_collect.len(),after_output_collect.len());

    for i in 0..max_line {
        if before_output_collect.len() <= i {
            before_output_collect.push("");
        }
        if after_output_collect.len() <= i {
            after_output_collect.push("");
        }

        if before_output_collect[i] != after_output_collect[i] {
            let mut before_line_collect: Vec<char> = before_output_collect[i].chars().collect();
            let mut after_line_collect: Vec<char> = after_output_collect[i].chars().collect();

            let max_char = cmp::max(before_line_collect.len(),after_line_collect.len());

            for x in 0..max_char {
                let space: char = ' ';

                if before_line_collect.len() <= max_char {
                    before_line_collect.push(space);
                }
                if after_line_collect.len() <= max_char {
                    after_line_collect.push(space);
                }

                if before_line_collect[x] != after_line_collect[x]{
                    watch.update_output_pad_char(after_line_collect[x].to_string(),true);
                }  else {
                    watch.update_output_pad_char(after_line_collect[x].to_string(),false);
                }
            }
            watch.update_output_pad_char("\n".to_string(),false);
        } else {
            watch.update_output_pad_char(after_output_collect[i].to_string(),false);
            watch.update_output_pad_char("\n".to_string(),false);
        }
    }
}