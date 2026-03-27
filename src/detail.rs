use std::io::{self, IsTerminal};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const BLUE: &str = "\x1b[94m";
const YELLOW: &str = "\x1b[93m";

pub(crate) fn render_detail(markdown: &str) -> String {
    let color = io::stdout().is_terminal();
    let mut rendered = String::new();
    let mut in_code_block = false;

    for line in markdown.lines() {
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            if in_code_block {
                if !rendered.ends_with('\n') {
                    rendered.push('\n');
                }
            } else if !rendered.ends_with("\n\n") {
                rendered.push('\n');
            }
            continue;
        }

        if in_code_block {
            render_code_line(&mut rendered, line, color);
            continue;
        }

        if let Some(content) = line.strip_prefix("# ") {
            render_heading(&mut rendered, content, color, YELLOW);
        } else if let Some(content) = line.strip_prefix("## ") {
            render_heading(&mut rendered, content, color, BLUE);
        } else if let Some(content) = line.strip_prefix("### ") {
            render_heading(&mut rendered, content, color, CYAN);
        } else if let Some(content) = line.strip_prefix("- ") {
            if color {
                rendered.push_str(CYAN);
                rendered.push_str("- ");
                rendered.push_str(RESET);
                rendered.push_str(&render_inline_code(content, color));
            } else {
                rendered.push_str("- ");
                rendered.push_str(&render_inline_code(content, color));
            }
            rendered.push('\n');
        } else {
            rendered.push_str(&render_inline_code(line, color));
            rendered.push('\n');
        }
    }

    rendered
}

fn render_heading(out: &mut String, content: &str, color: bool, heading_color: &str) {
    if color {
        out.push_str(BOLD);
        out.push_str(heading_color);
        out.push_str(content);
        out.push_str(RESET);
    } else {
        out.push_str(content);
    }
    out.push('\n');
    out.push('\n');
}

fn render_code_line(out: &mut String, line: &str, color: bool) {
    if color {
        out.push_str(DIM);
        out.push_str("  ");
        out.push_str(line);
        out.push_str(RESET);
    } else {
        out.push_str("  ");
        out.push_str(line);
    }
    out.push('\n');
}

fn render_inline_code(line: &str, color: bool) -> String {
    let mut result = String::new();
    let mut in_code = false;

    for segment in line.split('`') {
        if in_code && color {
            result.push_str(BOLD);
            result.push_str(CYAN);
            result.push_str(segment);
            result.push_str(RESET);
        } else {
            result.push_str(segment);
        }
        in_code = !in_code;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::render_detail;

    #[test]
    fn headings_are_preserved() {
        let rendered = render_detail("# Title\n\n## Usage\n");
        assert!(rendered.contains("Title"));
        assert!(rendered.contains("Usage"));
    }

    #[test]
    fn code_blocks_are_rendered_without_fence_markers() {
        let rendered = render_detail("```bash\ncutcut -d / aa/bb\n```\n");
        assert!(rendered.contains("cutcut -d / aa/bb"));
        assert!(!rendered.contains("```"));
    }

    #[test]
    fn inline_code_text_is_kept() {
        let rendered = render_detail("Use `-d` and `-f`.\n");
        assert!(rendered.contains("-d"));
        assert!(rendered.contains("-f"));
    }
}
