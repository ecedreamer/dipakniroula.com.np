use askama::Error;


pub fn truncate_words(value: &str, num_words: usize) -> Result<String, Error> {
    let words: Vec<&str> = value.split_whitespace().collect();
    if words.len() <= num_words {
        Ok(value.to_string())
    } else {
        let truncated: String = words[..num_words].join(" ");
        Ok(format!("{}...", truncated))
    }
}


pub fn strip_tags(s: String) -> String {
    ammonia::clean(&s)
}