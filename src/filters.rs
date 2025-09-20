use askama::Result;
use regex::Regex;
use askama::Values;


pub fn truncate_words(value: &str, num_words: usize) -> askama::Result<String> {
    let words: Vec<&str> = value.split_whitespace().collect();
    if words.len() <= num_words {
        Ok(value.to_string())
    } else {
        let truncated: String = words[..num_words].join(" ");
        Ok(format!("{}...", truncated))
    }
}




pub fn strip_html_tags(s: &str, _: &dyn Values) -> Result<String> {
    let re = Regex::new(r"<[^>]*>").unwrap();
    let without_tags = re.replace_all(s, "").to_string();

    let replacements = [
        ("&nbsp;", " "),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&amp;", "&"),
        ("&quot;", "\""),
        ("&apos;", "'"),
    ];

    let mut result = without_tags;
    for &(entity, replacement) in &replacements {
        let re_entity = Regex::new(entity).unwrap();
        result = re_entity.replace_all(&result, replacement).to_string();
    }

    Ok(result)
}