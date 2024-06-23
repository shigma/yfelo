use std::rc::Rc;

use once_cell::sync::Lazy;
use yfelo::{DefaultInterpreter, Node, Token, Yfelo};

const YFELO: Lazy<Yfelo> = Lazy::new(|| {
    let interpreter = Rc::new(DefaultInterpreter);
    Yfelo::new("{".to_string(), "}".to_string(), interpreter)
});

#[test]
pub fn tokenize_1() {
    let y = YFELO;
    let tokens = y.tokenize("(Hello) {world}!").unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0], Token::Text("(Hello) "));
    assert_eq!(tokens[1], Token::Tag("world", (9, 14)));
    assert_eq!(tokens[2], Token::Text("!"));
}

#[test]
pub fn tokenize_2() {
    let y = YFELO;
    let tokens = y.tokenize("{world}").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Tag("world", (1, 6)));
}

#[test]
pub fn tokenize_3() {
    let y = YFELO;
    let tokens = y.tokenize("{w(o[r])ld}").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Tag("w(o[r])ld", (1, 10)));
}

#[test]
pub fn tokenize_4() {
    let y = YFELO;
    let tokens = y.tokenize("{w{o}r{l}d}").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Tag("w{o}r{l}d", (1, 10)));
}

#[test]
pub fn unterminated_tag_1() {
    let y = YFELO;
    let err = y.tokenize("{Hello} {world").unwrap_err();
    assert_eq!(err.message, "unterminated tag syntax");
    assert_eq!(err.range, (8, 9));
}

#[test]
pub fn unterminated_tag_2() {
    let y = YFELO;
    let err = y.tokenize("{Hel(lo}").unwrap_err();
    assert_eq!(err.message, "unterminated tag syntax");
    assert_eq!(err.range, (7, 8));
}

#[test]
pub fn unterminated_tag_3() {
    let y = YFELO;
    let err = y.tokenize("{Hel)lo}").unwrap_err();
    assert_eq!(err.message, "unterminated tag syntax");
    assert_eq!(err.range, (4, 5));
}

#[test]
pub fn unterminated_tag_4() {
    let y = YFELO;
    let err = y.tokenize("{H(e[l)l]o}").unwrap_err();
    assert_eq!(err.message, "unterminated tag syntax");
    assert_eq!(err.range, (6, 7));
}

#[test]
pub fn parse_1() {
    let y = YFELO;
    let nodes = y.parse("Hello {world}!").unwrap();
    assert_eq!(nodes.len(), 3);
    assert_eq!(nodes[0], Node::Text("Hello "));
    assert_eq!(nodes[1], Node::Expr("world"));
    assert_eq!(nodes[2], Node::Text("!"));
}

#[test]
pub fn parse_2() {
    let y = YFELO;
    let nodes = y.parse("{#foo}Hello{/foo} {#bar}world{/bar}!").unwrap();
    assert_eq!(nodes.len(), 4);
    if let Node::Element(element) = &nodes[0] {
        assert_eq!(element.name, "foo");
        assert_eq!(element.children.len(), 1);
        assert_eq!(element.children[0], Node::Text("Hello"));
    } else {
        panic!("Expected Node::Element");
    }
    assert_eq!(nodes[1], Node::Text(" "));
    if let Node::Element(element) = &nodes[2] {
        assert_eq!(element.name, "bar");
        assert_eq!(element.children.len(), 1);
        assert_eq!(element.children[0], Node::Text("world"));
    } else {
        panic!("Expected Node::Element");
    }
    assert_eq!(nodes[3], Node::Text("!"));
}

#[test]
pub fn parse_3() {
    let y = YFELO;
    let nodes = y.parse("{#foo}Hello {#bar}world{/bar}!{/foo}").unwrap();
    assert_eq!(nodes.len(), 1);
    if let Node::Element(element) = &nodes[0] {
        assert_eq!(element.name, "foo");
        assert_eq!(element.children.len(), 3);
        assert_eq!(element.children[0], Node::Text("Hello "));
        if let Node::Element(element) = &element.children[1] {
            assert_eq!(element.name, "bar");
            assert_eq!(element.children.len(), 1);
            assert_eq!(element.children[0], Node::Text("world"));
        } else {
            panic!("Expected Node::Element");
        }
        assert_eq!(element.children[2], Node::Text("!"));
    } else {
        panic!("Expected Node::Element");
    }
}

#[test]
pub fn unmatched_tag_1() {
    let y = YFELO;
    let error = y.parse("{#foo}Hello {#bar}world{/foo}!{/bar}").unwrap_err();
    assert_eq!(error.message, "unmatched tag name");
    assert_eq!(error.range, (24, 28));
}

#[test]
pub fn unmatched_tag_2() {
    let y = YFELO;
    let error = y.parse("{#foo}Hello{/foo} world{/bar}!").unwrap_err();
    assert_eq!(error.message, "unmatched tag name");
    assert_eq!(error.range, (24, 28));
}
