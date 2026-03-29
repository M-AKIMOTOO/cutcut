use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufWriter, IsTerminal, Read, Write};

use crate::cli::Config;
use crate::diagnostic::{AppError, invalid_ignore_regex_error};
use crate::split::build_delimiter_regex;
use crate::split::{select_field, split_fields};

pub(crate) struct RuntimeConfig {
    pub(crate) config: Config,
    pub(crate) delimiter_regex: Option<Regex>,
    ignore_regex: Option<Regex>,
}

pub(crate) fn run(config: Config) -> Result<(), AppError> {
    let runtime = build_runtime_config(config)?;
    let mut writer = make_writer(&runtime)?;

    if let Some(text) = &runtime.config.text {
        if !runtime.config.components.is_empty() {
            write_components(&mut writer, text, &runtime)?;
            return Ok(());
        }
        write_text(&mut writer, text, &runtime)?;
        return Ok(());
    }

    let stdin = io::stdin();
    if stdin.is_terminal() {
        return Err(AppError::Help);
    }

    if !runtime.config.components.is_empty() {
        let mut data = String::new();
        stdin.lock().read_to_string(&mut data)?;
        write_components(&mut writer, &data, &runtime)?;
        return Ok(());
    }

    for line in stdin.lock().lines() {
        let line = line?;
        if let Some(rendered) = render_maybe_line(&line, &runtime) {
            writeln!(writer, "{rendered}")?;
        }
    }

    Ok(())
}

pub(crate) fn build_runtime_config(config: Config) -> Result<RuntimeConfig, AppError> {
    let delimiter_regex = if config.regex {
        Some(build_delimiter_regex(&config.delimiters)?)
    } else {
        None
    };

    let ignore_regex = if config.regex {
        config
            .ignore_pattern
            .as_deref()
            .map(build_ignore_regex)
            .transpose()?
    } else {
        None
    };

    Ok(RuntimeConfig {
        config,
        delimiter_regex,
        ignore_regex,
    })
}

pub(crate) fn render_line(line: &str, runtime: &RuntimeConfig) -> String {
    let fields: Vec<&str> = split_fields(line, runtime)
        .into_iter()
        .map(str::trim)
        .collect();

    if runtime.config.count {
        fields.len().to_string()
    } else if !runtime.config.all && !runtime.config.fields.is_empty() {
        let selected: Vec<&str> = runtime
            .config
            .fields
            .iter()
            .map(|&index| select_field(&fields, index).unwrap_or(""))
            .collect();
        selected.join(runtime.config.replace.as_deref().unwrap_or(", "))
    } else {
        fields.join(runtime.config.replace.as_deref().unwrap_or(", "))
    }
}

fn render_components(text: &str, runtime: &RuntimeConfig) -> Vec<String> {
    let filtered = filter_ignored_lines(text, runtime);
    let fields: Vec<&str> = split_fields(&filtered, runtime)
        .into_iter()
        .map(str::trim)
        .collect();

    runtime
        .config
        .components
        .iter()
        .map(|&index| select_field(&fields, index).unwrap_or("").to_string())
        .collect()
}

pub(crate) fn should_ignore_line(line: &str, runtime: &RuntimeConfig) -> bool {
    if runtime.config.regex {
        return runtime
            .ignore_regex
            .as_ref()
            .is_some_and(|regex| regex.is_match(line));
    }

    runtime
        .config
        .ignore_pattern
        .as_deref()
        .is_some_and(|pattern| line.contains(pattern))
}

fn render_maybe_line(line: &str, runtime: &RuntimeConfig) -> Option<String> {
    if should_ignore_line(line, runtime) {
        None
    } else {
        Some(render_line(line, runtime))
    }
}

fn write_text<W: Write>(
    writer: &mut W,
    text: &str,
    runtime: &RuntimeConfig,
) -> Result<(), AppError> {
    let mut wrote_any = false;

    for line in text.lines() {
        if let Some(rendered) = render_maybe_line(line, runtime) {
            writeln!(writer, "{rendered}")?;
            wrote_any = true;
        }
    }

    if !wrote_any && !text.contains('\n') {
        if let Some(rendered) = render_maybe_line(text, runtime) {
            writeln!(writer, "{rendered}")?;
        }
    }

    Ok(())
}

fn write_components<W: Write>(
    writer: &mut W,
    text: &str,
    runtime: &RuntimeConfig,
) -> Result<(), AppError> {
    for component in render_components(text, runtime) {
        writeln!(writer, "{component}")?;
    }
    Ok(())
}

fn filter_ignored_lines(text: &str, runtime: &RuntimeConfig) -> String {
    text.lines()
        .filter(|line| !should_ignore_line(line, runtime))
        .collect::<Vec<_>>()
        .join("\n")
}

fn make_writer(runtime: &RuntimeConfig) -> Result<Box<dyn Write>, AppError> {
    if let Some(path) = &runtime.config.output {
        let file = File::create(path)?;
        Ok(Box::new(BufWriter::new(file)))
    } else {
        let stdout = io::stdout();
        Ok(Box::new(BufWriter::new(stdout)))
    }
}

fn build_ignore_regex(pattern: &str) -> Result<Regex, AppError> {
    Regex::new(pattern).map_err(invalid_ignore_regex_error)
}

#[cfg(test)]
mod tests {
    use crate::cli::Config;
    use crate::runtime::{
        build_runtime_config, render_components, render_line, should_ignore_line,
    };

    fn runtime(config: Config) -> crate::runtime::RuntimeConfig {
        build_runtime_config(config).expect("test config should be valid")
    }

    #[test]
    fn splits_by_string_delimiter() {
        let config = Config {
            delimiters: vec!["aaa".to_string()],
            all: false,
            components: Vec::new(),
            count: false,
            fields: Vec::new(),
            ignore_pattern: None,
            max_split: None,
            output: None,
            replace: None,
            regex: false,
            text: None,
        };

        assert_eq!(
            render_line("baaabbbaaaccc", &runtime(config)),
            "b, bbb, ccc"
        );
    }

    #[test]
    fn prints_selected_field() {
        let config = Config {
            delimiters: vec!["aaa".to_string()],
            all: false,
            components: Vec::new(),
            count: false,
            fields: vec![2],
            ignore_pattern: None,
            max_split: None,
            output: None,
            replace: None,
            regex: false,
            text: None,
        };

        assert_eq!(render_line("baaabbbaaaccc", &runtime(config)), "bbb");
    }

    #[test]
    fn missing_field_becomes_empty_string() {
        let config = Config {
            delimiters: vec!["aaa".to_string()],
            all: false,
            components: Vec::new(),
            count: false,
            fields: vec![4],
            ignore_pattern: None,
            max_split: None,
            output: None,
            replace: None,
            regex: false,
            text: None,
        };

        assert_eq!(render_line("baaabbbaaaccc", &runtime(config)), "");
    }

    #[test]
    fn multiline_text_can_be_processed_line_by_line() {
        let config = Config {
            delimiters: vec!["aa".to_string()],
            all: false,
            components: Vec::new(),
            count: false,
            fields: vec![2],
            ignore_pattern: None,
            max_split: None,
            output: None,
            replace: None,
            regex: false,
            text: None,
        };

        let rendered: Vec<String> = ["zaaqq", "maaann", "plain"]
            .into_iter()
            .map(|line| render_line(line, &runtime(config.clone())))
            .collect();

        assert_eq!(rendered, vec!["qq", "ann", ""]);
    }

    #[test]
    fn replace_rejoins_with_requested_string() {
        let config = Config {
            delimiters: vec!["/".to_string()],
            all: false,
            components: Vec::new(),
            count: false,
            fields: Vec::new(),
            ignore_pattern: None,
            max_split: None,
            output: None,
            replace: Some(":".to_string()),
            regex: false,
            text: None,
        };

        assert_eq!(render_line("aa/bb/cc", &runtime(config)), "aa:bb:cc");
    }

    #[test]
    fn negative_field_counts_from_end() {
        let config = Config {
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
        };

        assert_eq!(render_line("aa/bb/cc", &runtime(config)), "cc");
    }

    #[test]
    fn multiple_fields_are_joined_in_requested_order() {
        let config = Config {
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
            text: None,
        };

        assert_eq!(render_line("aa/bb/cc", &runtime(config)), "aa, cc, cc");
    }

    #[test]
    fn fields_are_trimmed_by_default() {
        let config = Config {
            delimiters: vec![",".to_string()],
            all: false,
            components: Vec::new(),
            count: false,
            fields: Vec::new(),
            ignore_pattern: None,
            max_split: None,
            output: None,
            replace: None,
            regex: false,
            text: None,
        };

        assert_eq!(render_line("aa,  bb , cc", &runtime(config)), "aa, bb, cc");
    }

    #[test]
    fn ignore_matches_substring_in_normal_mode() {
        let config = Config {
            delimiters: vec!["/".to_string()],
            all: false,
            components: Vec::new(),
            count: false,
            fields: Vec::new(),
            ignore_pattern: Some("#".to_string()),
            max_split: None,
            output: None,
            replace: None,
            regex: false,
            text: None,
        };

        assert!(should_ignore_line("xx # comment", &runtime(config)));
    }

    #[test]
    fn regex_ignore_supports_anchors() {
        let config = Config {
            delimiters: vec!["/".to_string()],
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
        };

        assert!(should_ignore_line("# comment", &runtime(config)));
    }

    #[test]
    fn count_mode_prints_number_of_fields() {
        let config = Config {
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
            text: None,
        };

        assert_eq!(render_line("aa/bb/cc", &runtime(config)), "3");
    }

    #[test]
    fn component_mode_selects_from_full_split_stream() {
        let config = Config {
            delimiters: vec!["space".to_string()],
            all: false,
            components: vec![1, 4, 10],
            count: false,
            fields: Vec::new(),
            ignore_pattern: None,
            max_split: None,
            output: None,
            replace: None,
            regex: false,
            text: None,
        };

        let rendered = render_components("a b c d e f g h i j", &runtime(config));
        assert_eq!(rendered, vec!["a", "d", "j"]);
    }
}
