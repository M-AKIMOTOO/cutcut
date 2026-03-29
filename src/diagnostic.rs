#[derive(Debug)]
pub(crate) struct Diagnostic {
    summary: String,
    detail: Option<String>,
    suggestions: Vec<String>,
    examples: Vec<String>,
}

impl Diagnostic {
    pub(crate) fn new(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            detail: None,
            suggestions: Vec::new(),
            examples: Vec::new(),
        }
    }

    pub(crate) fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub(crate) fn suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    pub(crate) fn example(mut self, example: impl Into<String>) -> Self {
        self.examples.push(example.into());
        self
    }
}

#[derive(Debug)]
pub(crate) enum AppError {
    Help,
    Detail,
    Version,
    Diagnostic(Diagnostic),
    Io(std::io::Error),
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::Io(error)
    }
}

pub(crate) fn print_diagnostic(diagnostic: &Diagnostic) {
    eprintln!("error: {}", diagnostic.summary);
    if let Some(detail) = &diagnostic.detail {
        eprintln!("detail: {detail}");
    }
    if !diagnostic.suggestions.is_empty() {
        eprintln!();
        eprintln!("Try:");
        for suggestion in &diagnostic.suggestions {
            eprintln!("  - {suggestion}");
        }
    }
    if !diagnostic.examples.is_empty() {
        eprintln!();
        eprintln!("Examples:");
        for example in &diagnostic.examples {
            eprintln!("  {example}");
        }
    }
    eprintln!();
    eprintln!("Quick usage:");
    eprintln!("  cutcut -d '/' aa/bb/cc");
    eprintln!("  printf 'aa/bb/cc\\n' | cutcut -d '/' -f 2");
}

fn diagnostic_error(diagnostic: Diagnostic) -> AppError {
    AppError::Diagnostic(diagnostic)
}

pub(crate) fn missing_value_error(option: &str) -> AppError {
    diagnostic_error(
        Diagnostic::new(format!("missing value for {option}"))
            .detail("An option that requires an argument was given without its value.")
            .suggestion("Put the value immediately after the option")
            .suggestion("Quote the value if it contains spaces or shell-special characters")
            .example("cutcut -d '/' -f 2 aa/bb/cc")
            .example("cutcut -d 'space' -f 2 \"foo   bar   baz\""),
    )
}

pub(crate) fn unknown_option_error(option: &str) -> AppError {
    diagnostic_error(
        Diagnostic::new(format!("unknown option: {option}"))
            .detail("This flag is not recognized by cutcut.")
            .suggestion("Check the spelling of the option")
            .suggestion("Long options may be abbreviated if the prefix is unique, for example --del for --delimiter")
            .suggestion("Use --help to see the supported options")
            .example("cutcut --help")
            .example("cutcut -d ',' -f 2 aa,bb,cc"),
    )
}

pub(crate) fn ambiguous_long_option_error(option: &str, candidates: &[&str]) -> AppError {
    diagnostic_error(
        Diagnostic::new(format!("ambiguous option: {option}"))
            .detail(format!(
                "This prefix matches multiple long options: {}",
                candidates.join(", ")
            ))
            .suggestion("Type a longer prefix until only one option matches")
            .suggestion("Or use the full long option name")
            .example("cutcut --del '/' aa/bb/cc")
            .example("cutcut --detail"),
    )
}

pub(crate) fn missing_delimiter_error() -> AppError {
    diagnostic_error(
        Diagnostic::new("missing required option: -d/--delimiter")
            .detail("cutcut needs at least one delimiter or delimiter regex.")
            .suggestion("Add -d STRING for fixed-string mode")
            .suggestion("Add -x/--regex -d REGEX for regex mode")
            .example("cutcut -d '/' aa/bb/cc")
            .example("cutcut --regex -d '#+' 'aa###bb'"),
    )
}

pub(crate) fn empty_delimiter_error() -> AppError {
    diagnostic_error(
        Diagnostic::new("delimiter must not be an empty string")
            .detail("An empty delimiter would be ambiguous and is not allowed.")
            .suggestion("Use a non-empty delimiter")
            .suggestion("For whitespace splitting, use -d ' ' or -d 'space'")
            .example("cutcut -d ',' aa,bb,cc")
            .example("cutcut -d 'space' 'aa   bb   cc'"),
    )
}

pub(crate) fn all_and_field_conflict_error() -> AppError {
    diagnostic_error(
        Diagnostic::new("-a/--all and -f/--field cannot be used together")
            .detail("-a prints every field, while -f selects specific fields.")
            .suggestion("Remove -a if you want selected fields")
            .suggestion("Remove -f if you want all fields")
            .example("cutcut -d '/' -f 2 aa/bb/cc")
            .example("cutcut -d '/' -a aa/bb/cc"),
    )
}

pub(crate) fn count_conflict_error() -> AppError {
    diagnostic_error(
        Diagnostic::new("--count cannot be used with -a/--all, -f/--field, -c/--component, or -r/--replace")
            .detail("--count prints only the number of split fields, so field selection and rejoining do not apply.")
            .suggestion("Use --count by itself to print the field count")
            .suggestion("Remove --count if you want actual field values")
            .example("cutcut -d '/' --count aa/bb/cc")
            .example("printf 'aa/bb/cc\\nxx/yy\\n' | cutcut -d '/' --count"),
    )
}

pub(crate) fn component_conflict_error() -> AppError {
    diagnostic_error(
        Diagnostic::new("-c/--component cannot be used with -a/--all, -f/--field, --count, or -r/--replace")
            .detail("-c selects positions from the full split stream, so line-wise field selection and rejoining do not apply.")
            .suggestion("Use -c by itself to select stream-wide component positions")
            .suggestion("Use -f if you want per-line field selection")
            .example("printf 'a b c d\\n' | cutcut -d 'space' -c 1 4")
            .example("printf 'a b\\nc d\\n' | cutcut -d 'space' -f 1"),
    )
}

pub(crate) fn empty_ignore_pattern_error() -> AppError {
    diagnostic_error(
        Diagnostic::new("ignore pattern must not be an empty string")
            .detail("An empty ignore pattern would match everything.")
            .suggestion("Pass a non-empty substring or regex")
            .suggestion("Use -i '#' to skip lines containing #")
            .example("cutcut -d '/' -i '#' -f 2")
            .example("cutcut --regex -d '/' -i '^#' -f 2"),
    )
}

pub(crate) fn invalid_delimiter_regex_error(error: regex::Error) -> AppError {
    diagnostic_error(
        Diagnostic::new("invalid delimiter regex")
            .detail(error.to_string())
            .suggestion("Check your regex syntax")
            .suggestion("If you meant a fixed string, remove --regex")
            .suggestion("For one or more # characters, use '#+' instead of '#*'")
            .example("cutcut --regex -d '#+' 'aa###bb###cc'")
            .example("cutcut -d '#' -r '' 'aa###bb'"),
    )
}

pub(crate) fn empty_matching_delimiter_regex_error() -> AppError {
    diagnostic_error(
        Diagnostic::new("delimiter regex must not match an empty string")
            .detail("A delimiter regex that matches the empty string would split everywhere.")
            .suggestion("Use a regex that consumes at least one character")
            .suggestion("Use '#+' instead of '#*' for one-or-more # characters")
            .example("cutcut --regex -d '#+' 'aa###bb###cc'")
            .example("cutcut --regex -d '[[:space:]]+' 'aa   bb\tcc'"),
    )
}

pub(crate) fn invalid_ignore_regex_error(error: regex::Error) -> AppError {
    diagnostic_error(
        Diagnostic::new("invalid ignore regex")
            .detail(error.to_string())
            .suggestion("Check the regex syntax for -i")
            .suggestion("If you meant substring matching, remove --regex")
            .example("cutcut --regex -d '/' -i '^#' -f 2")
            .example("cutcut -d '/' -i '#' -f 2"),
    )
}

pub(crate) fn invalid_field_error(value: &str) -> AppError {
    diagnostic_error(
        Diagnostic::new(format!("invalid field index: {value}"))
            .detail("Field indices must be integers such as 1, 2, -1, or -2.")
            .suggestion("Use a positive number to count from the front")
            .suggestion("Use a negative number to count from the back")
            .example("cutcut -d '/' -f 2 aa/bb/cc")
            .example("cutcut -d '/' -f -1 aa/bb/cc"),
    )
}

pub(crate) fn zero_field_error() -> AppError {
    diagnostic_error(
        Diagnostic::new("field index must not be 0")
            .detail("Field numbering starts at 1 from the front or -1 from the back.")
            .suggestion("Use 1 for the first field")
            .suggestion("Use -1 for the last field")
            .example("cutcut -d '/' -f 1 aa/bb/cc")
            .example("cutcut -d '/' -f -1 aa/bb/cc"),
    )
}

pub(crate) fn invalid_max_split_error(value: &str) -> AppError {
    diagnostic_error(
        Diagnostic::new(format!("invalid max split count: {value}"))
            .detail("The max split count must be an integer such as -2, -1, 0, 1, or 2.")
            .suggestion("Use a positive number to split from the front")
            .suggestion("Use a negative number to split from the end")
            .example("cutcut -d '=' -m 1 a=b=c=d")
            .example("cutcut -d '=' -m -1 a=b=c=d"),
    )
}
