use askama::Error;
use regex::Regex;


pub fn truncate_words(value: &str, num_words: usize) -> askama::Result<String, Error> {
    let words: Vec<&str> = value.split_whitespace().collect();
    if words.len() <= num_words {
        Ok(value.to_string())
    } else {
        let truncated: String = words[..num_words].join(" ");
        Ok(format!("{}...", truncated))
    }
}


pub fn strip_tags(s: &str) -> askama::Result<String> {
    let re = Regex::new(r"<[^>]*>").unwrap();
    let without_tags = re.replace_all(s, "").to_string();

    // Replace common HTML entities
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

pub fn myfilter<T: std::fmt::Display>(s: T, n: usize) -> ::askama::Result<String> {
    let s = s.to_string();
    let mut replace = String::with_capacity(n);
    replace.extend((0..n).map(|_| "a"));
    Ok(s.replace("oo", &replace))
}