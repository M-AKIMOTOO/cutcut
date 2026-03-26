use regex::Regex;

use crate::diagnostic::{
    AppError, empty_matching_delimiter_regex_error, invalid_delimiter_regex_error,
};
use crate::runtime::RuntimeConfig;

pub(crate) fn build_delimiter_regex(delimiters: &[String]) -> Result<Regex, AppError> {
    let mut parts = Vec::with_capacity(delimiters.len());
    for delimiter in delimiters {
        parts.push(format!("(?:{delimiter})"));
    }

    let regex = Regex::new(&parts.join("|")).map_err(invalid_delimiter_regex_error)?;

    if regex.is_match("") {
        return Err(empty_matching_delimiter_regex_error());
    }

    Ok(regex)
}

pub(crate) fn select_field<'a>(fields: &'a [&'a str], index: isize) -> Option<&'a str> {
    if index > 0 {
        fields.get((index - 1) as usize).copied()
    } else {
        fields
            .len()
            .checked_sub(index.unsigned_abs())
            .and_then(|pos| fields.get(pos))
            .copied()
    }
}

pub(crate) fn split_fields<'a>(line: &'a str, runtime: &RuntimeConfig) -> Vec<&'a str> {
    let matches = if runtime.config.regex {
        collect_regex_matches(
            line,
            runtime
                .delimiter_regex
                .as_ref()
                .expect("regex mode precompiles delimiter regex"),
        )
    } else if runtime
        .config
        .delimiters
        .iter()
        .all(|delimiter| is_space_delimiter(delimiter))
    {
        collect_whitespace_matches(line)
    } else {
        collect_fixed_matches(line, &runtime.config.delimiters)
    };

    split_by_matches(line, &matches, runtime.config.max_split)
}

fn is_space_delimiter(delimiter: &str) -> bool {
    delimiter == " " || delimiter == "space"
}

fn split_by_matches<'a>(
    line: &'a str,
    matches: &[DelimiterSpan],
    max_split: Option<isize>,
) -> Vec<&'a str> {
    let selected = select_matches(matches, max_split);
    let mut fields = Vec::with_capacity(selected.len() + 1);
    let mut current = 0;

    for matched in selected {
        fields.push(&line[current..matched.start]);
        current = matched.end;
    }

    fields.push(&line[current..]);
    fields
}

fn select_matches<'a>(
    matches: &'a [DelimiterSpan],
    max_split: Option<isize>,
) -> &'a [DelimiterSpan] {
    match max_split {
        None => matches,
        Some(limit) if limit >= 0 => {
            let count = (limit as usize).min(matches.len());
            &matches[..count]
        }
        Some(limit) => {
            let count = limit.unsigned_abs().min(matches.len());
            &matches[matches.len() - count..]
        }
    }
}

fn collect_regex_matches(line: &str, regex: &Regex) -> Vec<DelimiterSpan> {
    regex
        .find_iter(line)
        .map(|matched| DelimiterSpan {
            start: matched.start(),
            end: matched.end(),
        })
        .collect()
}

fn collect_whitespace_matches(line: &str) -> Vec<DelimiterSpan> {
    let mut matches = Vec::new();
    let mut index = 0;

    while index < line.len() {
        let ch = line[index..]
            .chars()
            .next()
            .expect("index points to a valid character boundary");
        if ch.is_whitespace() {
            let start = index;
            index += ch.len_utf8();

            while index < line.len() {
                let inner = line[index..]
                    .chars()
                    .next()
                    .expect("index points to a valid character boundary");
                if inner.is_whitespace() {
                    index += inner.len_utf8();
                } else {
                    break;
                }
            }

            matches.push(DelimiterSpan { start, end: index });
        } else {
            index += ch.len_utf8();
        }
    }

    matches
}

fn collect_fixed_matches(line: &str, delimiters: &[String]) -> Vec<DelimiterSpan> {
    let mut matches = Vec::new();
    let mut index = 0;

    while index < line.len() {
        if let Some(matched) = longest_delimiter_match(&line[index..], delimiters) {
            matches.push(DelimiterSpan {
                start: index,
                end: index + matched.len,
            });
            index += matched.len;
        } else {
            let next_len = line[index..]
                .chars()
                .next()
                .expect("index points to a valid character boundary")
                .len_utf8();
            index += next_len;
        }
    }

    matches
}

#[derive(Clone, Copy)]
struct DelimiterMatch {
    len: usize,
}

#[derive(Clone, Copy)]
struct DelimiterSpan {
    start: usize,
    end: usize,
}

fn longest_delimiter_match(input: &str, delimiters: &[String]) -> Option<DelimiterMatch> {
    let whitespace_len = if delimiters
        .iter()
        .any(|delimiter| is_space_delimiter(delimiter))
    {
        leading_whitespace_len(input)
    } else {
        None
    };

    let literal_len = delimiters
        .iter()
        .filter(|delimiter| !is_space_delimiter(delimiter))
        .filter_map(|delimiter| input.starts_with(delimiter).then_some(delimiter.len()))
        .max();

    match (whitespace_len, literal_len) {
        (Some(ws_len), Some(lit_len)) if ws_len > lit_len => Some(DelimiterMatch { len: ws_len }),
        (Some(_), Some(lit_len)) => Some(DelimiterMatch { len: lit_len }),
        (Some(ws_len), None) => Some(DelimiterMatch { len: ws_len }),
        (None, Some(lit_len)) => Some(DelimiterMatch { len: lit_len }),
        (None, None) => None,
    }
}

fn leading_whitespace_len(input: &str) -> Option<usize> {
    let mut bytes = 0;

    for ch in input.chars() {
        if ch.is_whitespace() {
            bytes += ch.len_utf8();
        } else {
            break;
        }
    }

    if bytes == 0 { None } else { Some(bytes) }
}

#[cfg(test)]
mod tests {
    use crate::cli::Config;
    use crate::runtime::{build_runtime_config, render_line};

    fn runtime(config: Config) -> crate::runtime::RuntimeConfig {
        build_runtime_config(config).expect("test config should be valid")
    }

    #[test]
    fn single_space_delimiter_splits_on_whitespace_runs() {
        let config = Config {
            delimiters: vec![" ".to_string()],
            all: false,
            fields: Vec::new(),
            ignore_pattern: None,
            max_split: None,
            output: None,
            replace: None,
            regex: false,
            text: None,
        };

        assert_eq!(render_line("aa   bb\tcc", &runtime(config)), "aa, bb, cc");
    }

    #[test]
    fn space_alias_splits_on_whitespace_runs() {
        let config = Config {
            delimiters: vec!["space".to_string()],
            all: false,
            fields: vec![2],
            ignore_pattern: None,
            max_split: None,
            output: None,
            replace: None,
            regex: false,
            text: None,
        };

        assert_eq!(render_line("aa   bb\tcc", &runtime(config)), "bb");
    }

    #[test]
    fn whitespace_replace_rejoins_with_requested_string() {
        let config = Config {
            delimiters: vec!["space".to_string()],
            all: false,
            fields: Vec::new(),
            ignore_pattern: None,
            max_split: None,
            output: None,
            replace: Some("/".to_string()),
            regex: false,
            text: None,
        };

        assert_eq!(render_line("aa   bb\tcc", &runtime(config)), "aa/bb/cc");
    }

    #[test]
    fn multiple_delimiters_are_supported() {
        let config = Config {
            delimiters: vec![",".to_string(), ".".to_string(), "|".to_string()],
            all: false,
            fields: Vec::new(),
            ignore_pattern: None,
            max_split: None,
            output: None,
            replace: None,
            regex: false,
            text: None,
        };

        assert_eq!(
            render_line("aa,bb.cc|dd", &runtime(config)),
            "aa, bb, cc, dd"
        );
    }

    #[test]
    fn longest_matching_delimiter_wins() {
        let config = Config {
            delimiters: vec!["aa".to_string(), "aaaa".to_string()],
            all: false,
            fields: Vec::new(),
            ignore_pattern: None,
            max_split: None,
            output: None,
            replace: None,
            regex: false,
            text: None,
        };

        assert_eq!(render_line("baaaac", &runtime(config)), "b, c");
    }

    #[test]
    fn regex_delimiter_splits_repeated_symbols() {
        let config = Config {
            delimiters: vec!["#+".to_string()],
            all: false,
            fields: Vec::new(),
            ignore_pattern: None,
            max_split: None,
            output: None,
            replace: None,
            regex: true,
            text: None,
        };

        assert_eq!(
            render_line("aa######bb##cc", &runtime(config)),
            "aa, bb, cc"
        );
    }

    #[test]
    fn regex_delimiter_does_not_use_whitespace_alias() {
        let config = Config {
            delimiters: vec!["[[:space:]]+".to_string()],
            all: false,
            fields: Vec::new(),
            ignore_pattern: None,
            max_split: None,
            output: None,
            replace: None,
            regex: true,
            text: None,
        };

        assert_eq!(render_line("aa   bb\tcc", &runtime(config)), "aa, bb, cc");
    }

    #[test]
    fn invalid_regex_is_rejected() {
        let config = Config {
            delimiters: vec!["(".to_string()],
            all: false,
            fields: Vec::new(),
            ignore_pattern: None,
            max_split: None,
            output: None,
            replace: None,
            regex: true,
            text: None,
        };

        assert!(build_runtime_config(config).is_err());
    }

    #[test]
    fn max_split_limits_fixed_string_splitting() {
        let config = Config {
            delimiters: vec!["=".to_string()],
            all: false,
            fields: Vec::new(),
            ignore_pattern: None,
            max_split: Some(1),
            output: None,
            replace: None,
            regex: false,
            text: None,
        };

        assert_eq!(render_line("a=b=c=d", &runtime(config)), "a, b=c=d");
    }

    #[test]
    fn max_split_limits_regex_splitting() {
        let config = Config {
            delimiters: vec!["=+".to_string()],
            all: false,
            fields: Vec::new(),
            ignore_pattern: None,
            max_split: Some(2),
            output: None,
            replace: None,
            regex: true,
            text: None,
        };

        assert_eq!(
            render_line("a==b===c====d", &runtime(config)),
            "a, b, c====d"
        );
    }

    #[test]
    fn negative_max_split_limits_fixed_string_from_end() {
        let config = Config {
            delimiters: vec!["=".to_string()],
            all: false,
            fields: Vec::new(),
            ignore_pattern: None,
            max_split: Some(-1),
            output: None,
            replace: None,
            regex: false,
            text: None,
        };

        assert_eq!(render_line("a=b=c=d", &runtime(config)), "a=b=c, d");
    }

    #[test]
    fn negative_max_split_limits_regex_from_end() {
        let config = Config {
            delimiters: vec!["=+".to_string()],
            all: false,
            fields: Vec::new(),
            ignore_pattern: None,
            max_split: Some(-2),
            output: None,
            replace: None,
            regex: true,
            text: None,
        };

        assert_eq!(render_line("a==b===c====d", &runtime(config)), "a==b, c, d");
    }

    #[test]
    fn negative_max_split_limits_whitespace_from_end() {
        let config = Config {
            delimiters: vec!["space".to_string()],
            all: false,
            fields: Vec::new(),
            ignore_pattern: None,
            max_split: Some(-1),
            output: None,
            replace: None,
            regex: false,
            text: None,
        };

        assert_eq!(render_line("aa   bb   cc", &runtime(config)), "aa   bb, cc");
    }
}
