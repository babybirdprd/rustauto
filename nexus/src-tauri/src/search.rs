use anyhow::Result;
use grep::regex::RegexMatcher;
use grep::searcher::Searcher;
use grep::searcher::sinks::UTF8;
use std::io::Cursor;

pub fn search_content(content: &str, query: &str) -> Result<Vec<String>> {
    let matcher = RegexMatcher::new(query)?;
    let mut matches = Vec::new();

    Searcher::new().search_reader(
        &matcher,
        Cursor::new(content.as_bytes()),
        UTF8(|_lnum, line| {
            matches.push(line.trim().to_string());
            Ok(true)
        }),
    )?;

    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_content() {
        let content = "Hello world\nThis is a test\nGoodbye world";
        let matches = search_content(content, "world").unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], "Hello world");
        assert_eq!(matches[1], "Goodbye world");
    }
}
