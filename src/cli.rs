use crate::diagnostic::{
    AppError, all_and_field_conflict_error, ambiguous_long_option_error, component_conflict_error,
    count_conflict_error, empty_delimiter_error, empty_ignore_pattern_error, invalid_field_error,
    invalid_max_split_error, missing_delimiter_error, missing_value_error, unknown_option_error,
    zero_field_error,
};

pub(crate) const DETAIL: &str = include_str!("../README.md");
pub(crate) const HELP: &str = "\
cutcut - split text by an arbitrary delimiter string

Usage:
  cutcut --version
  cutcut --detail
  cutcut -d DELIMITER [-d DELIMITER ...] [-a | -f FIELD [FIELD ...] | -c FIELD [FIELD ...] | --count] [-r REPLACEMENT] [-i PATTERN] [-m COUNT] [-o FILE] [-x|--regex] [TEXT...]
  cat file.txt | cutcut -d DELIMITER [-d DELIMITER ...] [-a | -f FIELD [FIELD ...] | -c FIELD [FIELD ...] | --count] [-r REPLACEMENT] [-i PATTERN] [-m COUNT] [-o FILE] [-x|--regex]

Options:
  -d, --delimiter  Delimiter string to split on; repeatable; use \" \" or \"space\" for whitespace
  -a, --all        Explicitly print all fields
  -f, --field      One or more field indices; positive counts from start, negative from end
  -c, --component  One or more positions from the full split stream, not line by line
  -i, --ignore     Ignore input lines containing PATTERN; with --regex, ignore matching lines
  -m, --max-split  Split at most COUNT times; negative counts from the end
  -o, --output     Write output to FILE instead of stdout
  -r, --replace    Rejoin fields with REPLACEMENT instead of \", \"
  --count          Print the number of fields after splitting
  -x, --regex      Interpret -d and -i patterns as regular expressions
  --detail         Show the embedded README with detailed usage and examples
  -V, --version    Show the cutcut version
  -h, --help       Show this help
";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Config {
    pub(crate) delimiters: Vec<String>,
    pub(crate) all: bool,
    pub(crate) components: Vec<isize>,
    pub(crate) count: bool,
    pub(crate) fields: Vec<isize>,
    pub(crate) ignore_pattern: Option<String>,
    pub(crate) max_split: Option<isize>,
    pub(crate) output: Option<String>,
    pub(crate) replace: Option<String>,
    pub(crate) regex: bool,
    pub(crate) text: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ParseResult {
    Config(Config),
    Help,
    Detail,
    Version,
}

pub(crate) fn parse_args<I>(args: I) -> Result<ParseResult, AppError>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter().peekable();
    let _program = args.next();

    if args.peek().is_none() {
        return Ok(ParseResult::Help);
    }

    let mut delimiters = Vec::new();
    let mut all = false;
    let mut components = Vec::new();
    let mut count = false;
    let mut fields = Vec::new();
    let mut ignore_pattern = None;
    let mut max_split = None;
    let mut output = None;
    let mut replace = None;
    let mut regex = false;
    let mut text_parts = Vec::new();

    while let Some(arg) = args.next() {
        let resolved = normalize_long_option(&arg)?;

        match resolved.as_str() {
            "-h" | "--help" => return Ok(ParseResult::Help),
            "--detail" => return Ok(ParseResult::Detail),
            "-V" | "--version" => return Ok(ParseResult::Version),
            "-d" | "--delimiter" => {
                let value = args
                    .next()
                    .ok_or_else(|| missing_value_error("-d/--delimiter"))?;
                delimiters.push(value);
            }
            "-f" | "--field" => {
                fields.extend(parse_field_values(&mut args)?);
            }
            "-c" | "--component" => {
                components.extend(parse_field_values(&mut args)?);
            }
            "-a" | "--all" => {
                all = true;
            }
            "--count" => {
                count = true;
            }
            "-i" | "--ignore" => {
                let value = args
                    .next()
                    .ok_or_else(|| missing_value_error("-i/--ignore"))?;
                ignore_pattern = Some(value);
            }
            "-m" | "--max-split" => {
                let value = args
                    .next()
                    .ok_or_else(|| missing_value_error("-m/--max-split"))?;
                max_split = Some(parse_max_split(&value)?);
            }
            "-o" | "--output" => {
                let value = args
                    .next()
                    .ok_or_else(|| missing_value_error("-o/--output"))?;
                output = Some(value);
            }
            "-r" | "--replace" => {
                let value = args
                    .next()
                    .ok_or_else(|| missing_value_error("-r/--replace"))?;
                replace = Some(value);
            }
            "-x" | "--regex" => {
                regex = true;
            }
            "--" => {
                text_parts.extend(args);
                break;
            }
            _ if resolved.starts_with('-') => {
                return Err(unknown_option_error(&arg));
            }
            _ => {
                text_parts.push(resolved);
                text_parts.extend(args);
                break;
            }
        }
    }

    if delimiters.is_empty() {
        return Err(missing_delimiter_error());
    }
    if delimiters.iter().any(|delimiter| delimiter.is_empty()) {
        return Err(empty_delimiter_error());
    }
    if all && !fields.is_empty() {
        return Err(all_and_field_conflict_error());
    }
    if count && (all || !fields.is_empty() || !components.is_empty() || replace.is_some()) {
        return Err(count_conflict_error());
    }
    if !components.is_empty() && (all || !fields.is_empty() || count || replace.is_some()) {
        return Err(component_conflict_error());
    }
    if ignore_pattern
        .as_ref()
        .is_some_and(|pattern| pattern.is_empty())
    {
        return Err(empty_ignore_pattern_error());
    }

    let text = if text_parts.is_empty() {
        None
    } else {
        Some(text_parts.join(" "))
    };

    Ok(ParseResult::Config(Config {
        delimiters,
        all,
        components,
        count,
        fields,
        ignore_pattern,
        max_split,
        output,
        replace,
        regex,
        text,
    }))
}

fn normalize_long_option(arg: &str) -> Result<String, AppError> {
    if !arg.starts_with("--") || arg == "--" {
        return Ok(arg.to_string());
    }

    const LONG_OPTIONS: &[&str] = &[
        "--help",
        "--detail",
        "--version",
        "--delimiter",
        "--all",
        "--component",
        "--field",
        "--count",
        "--ignore",
        "--max-split",
        "--output",
        "--replace",
        "--regex",
    ];

    if LONG_OPTIONS.contains(&arg) {
        return Ok(arg.to_string());
    }

    let matches: Vec<&str> = LONG_OPTIONS
        .iter()
        .copied()
        .filter(|option| option.starts_with(arg))
        .collect();

    match matches.as_slice() {
        [matched] => Ok((*matched).to_string()),
        [] => Err(unknown_option_error(arg)),
        _ => Err(ambiguous_long_option_error(arg, &matches)),
    }
}

fn parse_field(value: &str) -> Result<isize, AppError> {
    let field = value
        .parse::<isize>()
        .map_err(|_| invalid_field_error(value))?;

    if field == 0 {
        return Err(zero_field_error());
    }

    Ok(field)
}

fn parse_max_split(value: &str) -> Result<isize, AppError> {
    value
        .parse::<isize>()
        .map_err(|_| invalid_max_split_error(value))
}

fn parse_field_values<I>(args: &mut std::iter::Peekable<I>) -> Result<Vec<isize>, AppError>
where
    I: Iterator<Item = String>,
{
    let mut fields = Vec::new();

    while let Some(next) = args.peek() {
        if is_option_boundary(next) {
            break;
        }
        if next.parse::<isize>().is_err() {
            break;
        }

        let value = args.next().expect("peeked item should be available");
        fields.push(parse_field(&value)?);
    }

    if fields.is_empty() {
        return Err(missing_value_error("-f/--field"));
    }

    Ok(fields)
}

fn is_option_boundary(arg: &str) -> bool {
    matches!(
        arg,
        "-h" | "--help"
            | "-V"
            | "--version"
            | "-d"
            | "--delimiter"
            | "-a"
            | "--all"
            | "-c"
            | "--component"
            | "-f"
            | "--field"
            | "--count"
            | "-i"
            | "--ignore"
            | "-m"
            | "--max-split"
            | "-o"
            | "--output"
            | "-r"
            | "--replace"
            | "-x"
            | "--regex"
            | "--detail"
            | "--"
    )
}

#[cfg(test)]
mod tests {
    use super::{Config, ParseResult, parse_args};

    #[test]
    fn parse_args_collects_text_after_options() {
        let result = parse_args([
            "cutcut".to_string(),
            "-d".to_string(),
            "aa".to_string(),
            "-d".to_string(),
            "bb".to_string(),
            "-a".to_string(),
            "-i".to_string(),
            "#".to_string(),
            "-r".to_string(),
            ":".to_string(),
            "foo".to_string(),
            "bar".to_string(),
        ])
        .unwrap();

        assert_eq!(
            result,
            ParseResult::Config(Config {
                delimiters: vec!["aa".to_string(), "bb".to_string()],
                all: true,
                components: Vec::new(),
                count: false,
                fields: Vec::new(),
                ignore_pattern: Some("#".to_string()),
                max_split: None,
                output: None,
                replace: Some(":".to_string()),
                regex: false,
                text: Some("foo bar".to_string()),
            })
        );
    }

    #[test]
    fn parse_args_supports_negative_field() {
        let result = parse_args([
            "cutcut".to_string(),
            "-d".to_string(),
            "/".to_string(),
            "-f".to_string(),
            "-1".to_string(),
        ])
        .unwrap();

        assert_eq!(
            result,
            ParseResult::Config(Config {
                delimiters: vec!["/".to_string()],
                all: false,
                components: Vec::new(),
                count: false,
                fields: vec![-1],
                ignore_pattern: None,
                max_split: None,
                output: None,
                replace: None,
                regex: false,
                text: None,
            })
        );
    }

    #[test]
    fn parse_args_supports_multiple_fields() {
        let result = parse_args([
            "cutcut".to_string(),
            "-d".to_string(),
            "/".to_string(),
            "-f".to_string(),
            "1".to_string(),
            "3".to_string(),
            "-1".to_string(),
            "--".to_string(),
            "aa/bb/cc".to_string(),
        ])
        .unwrap();

        assert_eq!(
            result,
            ParseResult::Config(Config {
                delimiters: vec!["/".to_string()],
                all: false,
                components: Vec::new(),
                count: false,
                fields: vec![1, 3, -1],
                ignore_pattern: None,
                max_split: None,
                output: None,
                replace: None,
                regex: false,
                text: Some("aa/bb/cc".to_string()),
            })
        );
    }

    #[test]
    fn parse_args_supports_regex_flag() {
        let result = parse_args([
            "cutcut".to_string(),
            "--regex".to_string(),
            "-d".to_string(),
            "#+".to_string(),
            "-i".to_string(),
            "^#".to_string(),
        ])
        .unwrap();

        assert_eq!(
            result,
            ParseResult::Config(Config {
                delimiters: vec!["#+".to_string()],
                all: false,
                components: Vec::new(),
                count: false,
                fields: Vec::new(),
                ignore_pattern: Some("^#".to_string()),
                max_split: None,
                output: None,
                replace: None,
                regex: true,
                text: None,
            })
        );
    }

    #[test]
    fn parse_args_supports_max_split_and_output() {
        let result = parse_args([
            "cutcut".to_string(),
            "--del".to_string(),
            "=".to_string(),
            "--max".to_string(),
            "1".to_string(),
            "--out".to_string(),
            "out.txt".to_string(),
            "a=b=c".to_string(),
        ])
        .unwrap();

        assert_eq!(
            result,
            ParseResult::Config(Config {
                delimiters: vec!["=".to_string()],
                all: false,
                components: Vec::new(),
                count: false,
                fields: Vec::new(),
                ignore_pattern: None,
                max_split: Some(1),
                output: Some("out.txt".to_string()),
                replace: None,
                regex: false,
                text: Some("a=b=c".to_string()),
            })
        );
    }

    #[test]
    fn parse_args_supports_negative_max_split() {
        let result = parse_args([
            "cutcut".to_string(),
            "-d".to_string(),
            "=".to_string(),
            "-m".to_string(),
            "-1".to_string(),
            "a=b=c=d".to_string(),
        ])
        .unwrap();

        assert_eq!(
            result,
            ParseResult::Config(Config {
                delimiters: vec!["=".to_string()],
                all: false,
                components: Vec::new(),
                count: false,
                fields: Vec::new(),
                ignore_pattern: None,
                max_split: Some(-1),
                output: None,
                replace: None,
                regex: false,
                text: Some("a=b=c=d".to_string()),
            })
        );
    }

    #[test]
    fn parse_args_supports_abbreviated_long_options() {
        let result = parse_args([
            "cutcut".to_string(),
            "--del".to_string(),
            "/".to_string(),
            "--fie".to_string(),
            "2".to_string(),
            "--ign".to_string(),
            "#".to_string(),
            "--reg".to_string(),
        ])
        .unwrap();

        assert_eq!(
            result,
            ParseResult::Config(Config {
                delimiters: vec!["/".to_string()],
                all: false,
                components: Vec::new(),
                count: false,
                fields: vec![2],
                ignore_pattern: Some("#".to_string()),
                max_split: None,
                output: None,
                replace: None,
                regex: true,
                text: None,
            })
        );
    }

    #[test]
    fn parse_args_rejects_ambiguous_abbreviated_long_option() {
        let result = parse_args(["cutcut".to_string(), "--d".to_string()]);

        assert!(result.is_err());
    }

    #[test]
    fn parse_args_rejects_empty_delimiter() {
        let result = parse_args(["cutcut".to_string(), "-d".to_string(), "".to_string()]);

        assert!(result.is_err());
    }

    #[test]
    fn parse_args_rejects_all_with_field() {
        let result = parse_args([
            "cutcut".to_string(),
            "-d".to_string(),
            ",".to_string(),
            "-a".to_string(),
            "-f".to_string(),
            "1".to_string(),
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn parse_args_supports_count_flag() {
        let result = parse_args([
            "cutcut".to_string(),
            "-d".to_string(),
            "/".to_string(),
            "--count".to_string(),
            "aa/bb/cc".to_string(),
        ])
        .unwrap();

        assert_eq!(
            result,
            ParseResult::Config(Config {
                delimiters: vec!["/".to_string()],
                all: false,
                components: Vec::new(),
                count: true,
                fields: Vec::new(),
                ignore_pattern: None,
                max_split: None,
                output: None,
                replace: None,
                regex: false,
                text: Some("aa/bb/cc".to_string()),
            })
        );
    }

    #[test]
    fn parse_args_supports_component_flag() {
        let result = parse_args([
            "cutcut".to_string(),
            "-d".to_string(),
            "/".to_string(),
            "-c".to_string(),
            "1".to_string(),
            "3".to_string(),
            "aa/bb/cc".to_string(),
        ])
        .unwrap();

        assert_eq!(
            result,
            ParseResult::Config(Config {
                delimiters: vec!["/".to_string()],
                all: false,
                components: vec![1, 3],
                count: false,
                fields: Vec::new(),
                ignore_pattern: None,
                max_split: None,
                output: None,
                replace: None,
                regex: false,
                text: Some("aa/bb/cc".to_string()),
            })
        );
    }

    #[test]
    fn parse_args_supports_version_flag() {
        let result = parse_args(["cutcut".to_string(), "--version".to_string()]).unwrap();

        assert_eq!(result, ParseResult::Version);
    }
}
