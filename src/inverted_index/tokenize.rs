use regex::Regex;

pub fn tokenize(text: &str) -> Vec<String> {
    let re = Regex::new(r"[\w'-]+|[[:punct:]]+").unwrap();
    re.find_iter(text)
        .map(|mat| mat.as_str().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_sentence() {
        let text = "It's such a beautiful day!!";
        let tokens = tokenize(text);
        assert_eq!(tokens, vec!["It's", "such", "a", "beautiful", "day", "!!"]);
    }

    #[test]
    fn test_with_multiple_spaces() {
        let text = "   Hello,    world!   ";
        let tokens = tokenize(text);
        assert_eq!(tokens, vec!["Hello", ",", "world", "!"]);
    }

    #[test]
    fn test_empty_string() {
        let text = "";
        let tokens = tokenize(text);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_with_only_punctuation() {
        let text = "?!.,";
        let tokens = tokenize(text);
        assert_eq!(tokens, vec!["?!.,"]);
    }

    #[test]
    fn test_mixed_case_sentence() {
        let text = "Rust is Fun and Powerful!";
        let tokens = tokenize(text);
        assert_eq!(tokens, vec!["Rust", "is", "Fun", "and", "Powerful", "!"]);
    }

    #[test]
    fn test_numbers_and_words() {
        let text = "I have 2 apples and 10 bananas.";
        let tokens = tokenize(text);
        assert_eq!(
            tokens,
            vec!["I", "have", "2", "apples", "and", "10", "bananas", "."]
        );
    }

    #[test]
    fn test_with_special_characters() {
        let text = "Hello @world, how's #Rust?";
        let tokens = tokenize(text);
        assert_eq!(
            tokens,
            vec!["Hello", "@", "world", ",", "how's", "#", "Rust", "?"]
        );
    }

    #[test]
    fn test_very_special_title() {
        let text = "=====================  Product Identification  =====================";
        let tokens = tokenize(text);
        assert_eq!(
            tokens,
            vec![
                "=====================",
                "Product",
                "Identification",
                "====================="
            ]
        );
    }
}
