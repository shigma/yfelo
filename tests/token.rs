use yfelo::{Token, tokenize};

#[test]
pub fn example_1() {
    let tokens = tokenize("Hello {world}!").unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0], Token::Text("Hello "));
    assert_eq!(tokens[1], Token::Tag("world"));
    assert_eq!(tokens[2], Token::Text("!"));
}

#[test]
pub fn example_2() {
    let tokens = tokenize("{world}").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Tag("world"));
}

#[test]
pub fn example_3() {
    let err = tokenize("{Hello} {world").unwrap_err();
    assert_eq!(err.message, "unterminated tag syntax");
    assert_eq!(err.position, 8);
}
