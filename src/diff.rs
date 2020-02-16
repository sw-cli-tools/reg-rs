use text_diff::{diff, Difference};

pub fn compare(older: &String, new: &String) -> Option<String> {
    let mut result = "".to_string();
    let differences = diff(older, new, "\n");
    dbg!(differences.0);
    if differences.0 > 0 {
        for difference in differences.1 {
            match difference {
                Difference::Same(same) => {
                    result.push('=');
                    result.push_str(&same);
                    result.push('\n');
                }
                Difference::Add(add) => {
                    result.push('+');
                    result.push_str(&add);
                    result.push('\n');
                }
                Difference::Rem(remove) => {
                    result.push('-');
                    result.push_str(&remove);
                    result.push('\n');
                }
            }
        }
        Some(result)
    } else {
        None
    }
}
