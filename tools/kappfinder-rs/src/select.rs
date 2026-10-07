//! Interactive prompts.
//!
//! KAppFinder presented a checklist and installed only what you ticked. The
//! terminal equivalent is a numbered list plus a range expression.

use std::io::{self, IsTerminal, Write};

/// Ask which of `count` listed items to act on.
///
/// Accepts `all`, `none`, and comma/range expressions such as `1,3,5-7`.
/// An empty answer means "all". EOF (piped stdin) means "none", so the tool
/// never writes files unattended without `--yes`.
pub fn prompt_selection(count: usize) -> io::Result<Vec<usize>> {
    if !io::stdin().is_terminal() {
        eprintln!("stdin is not a terminal; refusing to guess. Re-run with --yes to accept all.");
        return Ok(Vec::new());
    }

    print!("\nCreate which entries? [all / none / e.g. 1,3,5-7] (default: all): ");
    io::stdout().flush()?;

    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 {
        println!();
        return Ok(Vec::new());
    }
    Ok(parse_selection(&line, count))
}

/// Parse a selection expression into zero-based indices.
pub fn parse_selection(input: &str, count: usize) -> Vec<usize> {
    let s = input.trim();

    if s.is_empty() || s.eq_ignore_ascii_case("a") || s.eq_ignore_ascii_case("all") {
        return (0..count).collect();
    }
    if s.eq_ignore_ascii_case("n")
        || s.eq_ignore_ascii_case("no")
        || s.eq_ignore_ascii_case("none")
        || s.eq_ignore_ascii_case("q")
        || s.eq_ignore_ascii_case("quit")
    {
        return Vec::new();
    }

    let mut picked = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match part.split_once('-') {
            Some((lo, hi)) => {
                if let (Ok(lo), Ok(hi)) = (lo.trim().parse::<usize>(), hi.trim().parse::<usize>()) {
                    let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
                    for n in lo..=hi {
                        if (1..=count).contains(&n) {
                            picked.push(n - 1);
                        }
                    }
                }
            }
            None => {
                if let Ok(n) = part.parse::<usize>() {
                    if (1..=count).contains(&n) {
                        picked.push(n - 1);
                    }
                }
            }
        }
    }

    picked.sort_unstable();
    picked.dedup();
    picked
}

/// Yes/no confirmation. Non-interactive stdin answers `false`.
pub fn confirm(question: &str) -> io::Result<bool> {
    if !io::stdin().is_terminal() {
        return Ok(false);
    }
    print!("{question} [y/N]: ");
    io::stdout().flush()?;

    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 {
        println!();
        return Ok(false);
    }
    let answer = line.trim();
    Ok(answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_all_select_everything() {
        assert_eq!(parse_selection("", 3), vec![0, 1, 2]);
        assert_eq!(parse_selection("  ", 3), vec![0, 1, 2]);
        assert_eq!(parse_selection("all", 3), vec![0, 1, 2]);
        assert_eq!(parse_selection("A", 3), vec![0, 1, 2]);
    }

    #[test]
    fn none_selects_nothing() {
        assert!(parse_selection("none", 3).is_empty());
        assert!(parse_selection("n", 3).is_empty());
        assert!(parse_selection("q", 3).is_empty());
    }

    #[test]
    fn parses_lists_and_ranges() {
        assert_eq!(parse_selection("1,3", 5), vec![0, 2]);
        assert_eq!(parse_selection("2-4", 5), vec![1, 2, 3]);
        assert_eq!(parse_selection("1,3-5", 5), vec![0, 2, 3, 4]);
    }

    #[test]
    fn reversed_ranges_still_work() {
        assert_eq!(parse_selection("4-2", 5), vec![1, 2, 3]);
    }

    #[test]
    fn out_of_range_and_garbage_are_dropped() {
        assert_eq!(parse_selection("0,9,2,abc", 3), vec![1]);
        assert!(parse_selection("99", 3).is_empty());
    }

    #[test]
    fn duplicates_collapse() {
        assert_eq!(parse_selection("2,2,1-2", 5), vec![0, 1]);
    }
}
