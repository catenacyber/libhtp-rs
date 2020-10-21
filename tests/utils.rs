#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
use libc;
use std::ffi::{CStr, CString};
use std::io::Write;

use htp::bstr::*;
use htp::c_api::{htp_connp_create, htp_connp_destroy_all};
use htp::config;
use htp::connection_parser::*;
use htp::list::List;
use htp::request::*;
use htp::table::*;
use htp::transaction::*;
use htp::urlencoded::*;
use htp::util::*;
use nom::error::ErrorKind::TakeUntil;
use nom::Err::Error;
use std::net::{IpAddr, Ipv4Addr};

// import common testing utilities
mod common;

// Util tests
#[test]
fn Separator() {
    assert_eq!(false, is_separator('a' as u8));
    assert_eq!(false, is_separator('^' as u8));
    assert_eq!(false, is_separator('-' as u8));
    assert_eq!(false, is_separator('_' as u8));
    assert_eq!(false, is_separator('&' as u8));
    assert_eq!(true, is_separator('(' as u8));
    assert_eq!(true, is_separator('\\' as u8));
    assert_eq!(true, is_separator('/' as u8));
    assert_eq!(true, is_separator('=' as u8));
    assert_eq!(true, is_separator('\t' as u8));
}

#[test]
fn Token() {
    assert_eq!(true, is_token('a' as u8));
    assert_eq!(true, is_token('&' as u8));
    assert_eq!(true, is_token('+' as u8));
    assert_eq!(false, is_token('\t' as u8));
    assert_eq!(false, is_token('\n' as u8));
}

fn unsize<T>(x: &[T]) -> &[T] {
    x
}

#[test]
fn Chomp() {
    assert_eq!(chomp(b"test\r\n"), b"test");
    assert_eq!(chomp(b"test\r\n\n"), b"test");
    assert_eq!(chomp(b"test\r\n\r\n"), b"test");
    assert_eq!(chomp(b"te\nst"), b"te\nst");
    assert_eq!(chomp(b"foo\n"), b"foo");
    assert_eq!(chomp(b"arfarf"), b"arfarf");
    assert_eq!(chomp(b""), b"");
}

#[test]
fn Space() {
    assert_eq!(false, is_space(0x61)); // a
    assert_eq!(true, is_space(0x20)); // space
    assert_eq!(true, is_space(0x0c)); // Form feed
    assert_eq!(true, is_space(0x0a)); // newline
    assert_eq!(true, is_space(0x0d)); // carriage return
    assert_eq!(true, is_space(0x09)); // tab
    assert_eq!(true, is_space(0x0b)); // Vertical tab
}

#[test]
fn Method() {
    let method = Bstr::from("GET");
    assert_eq!(htp_method_t::HTP_M_GET, convert_bstr_to_method(&method));
}

#[test]
fn IsLineEmpty() {
    let data = b"arfarf";
    assert_eq!(false, is_line_empty(data));
    assert_eq!(true, is_line_empty(b"\x0d\x0a"));
    assert_eq!(true, is_line_empty(b"\x0d"));
    assert_eq!(true, is_line_empty(b"\x0a"));
    assert_eq!(false, is_line_empty(b"\x0a\x0d"));
    assert_eq!(false, is_line_empty(b"\x0dabc"));
}

#[test]
fn IsLineWhitespace() {
    let data = b"arfarf";
    assert_eq!(false, is_line_whitespace(data));
    assert_eq!(true, is_line_whitespace(b"\x0d\x0a"));
    assert_eq!(true, is_line_whitespace(b"\x0d"));
    assert_eq!(false, is_line_whitespace(b"\x0dabc"));
}

#[test]
fn ParseContentLength() {
    assert_eq!(134, parse_content_length(b"134", None).unwrap());
    assert_eq!(134, parse_content_length(b"    \t134    ", None).unwrap());
    assert_eq!(134, parse_content_length(b"abcd134    ", None).unwrap());
    assert!(parse_content_length(b"abcd    ", None).is_none());
}

#[test]
fn ParseChunkedLength() {
    assert_eq!(Ok(Some(0x12a5)), parse_chunked_length(b"12a5"));
    assert_eq!(Ok(Some(0x12a5)), parse_chunked_length(b"    \t12a5    "));
}

#[test]
fn IsLineFolded() {
    assert_eq!(true, is_line_folded(b"\tline"));
    assert_eq!(true, is_line_folded(b" line"));
    assert_eq!(false, is_line_folded(b"line "));
}

#[derive(Clone)]
struct uri_expected {
    scheme: Option<Bstr>,
    username: Option<Bstr>,
    password: Option<Bstr>,
    hostname: Option<Bstr>,
    port: Option<Bstr>,
    path: Option<Bstr>,
    query: Option<Bstr>,
    fragment: Option<Bstr>,
}

#[derive(Clone)]
struct uri_test {
    uri: Option<Bstr>,
    expected: uri_expected,
}

fn bstr_equal_c(b: *const Bstr, c: *const i8) -> bool {
    unsafe {
        if (c == std::ptr::null()) || (b == std::ptr::null()) {
            (c == std::ptr::null()) && (b == std::ptr::null())
        } else {
            0 == bstr_cmp_c(b, c)
        }
    }
}

fn append_message<W: Write>(
    o: &mut W,
    label: *const i8,
    expected: Option<&Bstr>,
    actual: Option<&Bstr>,
) -> Result<(), std::io::Error> {
    unsafe {
        o.write_fmt(format_args!(
            "{} missmatch: ",
            CStr::from_ptr(label).to_str().unwrap()
        ))?;
        if let Some(expected) = expected {
            o.write(b"'")?;
            o.write(expected.as_slice())?;

            o.write(b"'")?;
        } else {
            o.write(b"<NULL>")?;
        }
        o.write(b" != ")?;
        if let Some(actual) = actual {
            o.write(b"'")?;
            o.write(actual.as_slice())?;
            o.write(b"'")?;
        } else {
            o.write(b"<NULL>")?;
        }
        o.write(b"\n")?;
        Ok(())
    }
}

fn UriIsExpected(expected: uri_expected, actual: &Uri) -> Result<(), std::io::Error> {
    let mut msg: Vec<u8> = vec![];
    let mut equal: bool = true;

    if actual.scheme != expected.scheme {
        equal = false;
        append_message(
            &mut msg,
            cstr!("scheme"),
            expected.scheme.as_ref(),
            actual.scheme.as_ref(),
        )?;
    }

    if actual.username != expected.username {
        equal = false;
        append_message(
            &mut msg,
            cstr!("username"),
            expected.username.as_ref(),
            actual.username.as_ref(),
        )?;
    }

    if actual.password != expected.password {
        equal = false;
        append_message(
            &mut msg,
            cstr!("password"),
            expected.password.as_ref(),
            (*actual).password.as_ref(),
        )?;
    }

    if actual.hostname != expected.hostname {
        equal = false;
        append_message(
            &mut msg,
            cstr!("hostname"),
            expected.hostname.as_ref(),
            actual.hostname.as_ref(),
        )?;
    }

    if actual.port != expected.port {
        equal = false;
        append_message(
            &mut msg,
            cstr!("port"),
            expected.port.as_ref(),
            actual.port.as_ref(),
        )?;
    }

    if actual.path != expected.path {
        equal = false;
        append_message(
            &mut msg,
            cstr!("path"),
            expected.path.as_ref(),
            actual.path.as_ref(),
        )?;
    }

    if actual.query != expected.query {
        equal = false;
        append_message(
            &mut msg,
            cstr!("query"),
            expected.query.as_ref(),
            actual.query.as_ref(),
        )?;
    }

    if actual.fragment != expected.fragment {
        equal = false;
        append_message(
            &mut msg,
            cstr!("fragment"),
            expected.fragment.as_ref(),
            actual.fragment.as_ref(),
        )?;
    }

    if equal {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            std::str::from_utf8(&msg).unwrap(),
        ))
    }
}

struct UriTest {
    uri_tests: Vec<uri_test>,
}

impl UriTest {
    fn new() -> Self {
        Self {
            uri_tests: {
                [
                    uri_test {
                        uri: Some(Bstr::from(
                            "http://user:pass@www.example.com:1234/path1/path2?a=b&c=d#frag",
                        )),
                        expected: uri_expected {
                            scheme: Some(Bstr::from("http")),
                            username: Some(Bstr::from("user")),
                            password: Some(Bstr::from("pass")),
                            hostname: Some(Bstr::from("www.example.com")),
                            port: Some(Bstr::from("1234")),
                            path: Some(Bstr::from("/path1/path2")),
                            query: Some(Bstr::from("a=b&c=d")),
                            fragment: Some(Bstr::from("frag")),
                        },
                    },
                    uri_test {
                        uri: Some(Bstr::from("http://host.com/path")),
                        expected: uri_expected {
                            scheme: Some(Bstr::from("http")),
                            username: None,
                            password: None,
                            hostname: Some(Bstr::from("host.com")),
                            port: None,
                            path: Some(Bstr::from("/path")),
                            query: None,
                            fragment: None,
                        },
                    },
                    uri_test {
                        uri: Some(Bstr::from("http://host.com")),
                        expected: uri_expected {
                            scheme: Some(Bstr::from("http")),
                            username: None,
                            password: None,
                            hostname: Some(Bstr::from("host.com")),
                            port: None,
                            path: None,
                            query: None,
                            fragment: None,
                        },
                    },
                    uri_test {
                        uri: Some(Bstr::from("http://")),
                        expected: uri_expected {
                            scheme: Some(Bstr::from("http")),
                            username: None,
                            password: None,
                            hostname: None,
                            port: None,
                            path: Some(Bstr::from("//")),
                            query: None,
                            fragment: None,
                        },
                    },
                    uri_test {
                        uri: Some(Bstr::from("/path")),
                        expected: uri_expected {
                            scheme: None,
                            username: None,
                            password: None,
                            hostname: None,
                            port: None,
                            path: Some(Bstr::from("/path")),
                            query: None,
                            fragment: None,
                        },
                    },
                    uri_test {
                        uri: Some(Bstr::from("://")),
                        expected: uri_expected {
                            scheme: Some(Bstr::from("")),
                            username: None,
                            password: None,
                            hostname: None,
                            port: None,
                            path: Some(Bstr::from("//")),
                            query: None,
                            fragment: None,
                        },
                    },
                    uri_test {
                        uri: Some(Bstr::from("")),
                        expected: uri_expected {
                            scheme: None,
                            username: None,
                            password: None,
                            hostname: None,
                            port: None,
                            path: None,
                            query: None,
                            fragment: None,
                        },
                    },
                    uri_test {
                        uri: Some(Bstr::from("http://user@host.com")),
                        expected: uri_expected {
                            scheme: Some(Bstr::from("http")),
                            username: Some(Bstr::from("user")),
                            password: None,
                            hostname: Some(Bstr::from("host.com")),
                            port: None,
                            path: None,
                            query: None,
                            fragment: None,
                        },
                    },
                    uri_test {
                        uri: None,
                        expected: uri_expected {
                            scheme: None,
                            username: None,
                            password: None,
                            hostname: None,
                            port: None,
                            path: None,
                            query: None,
                            fragment: None,
                        },
                    },
                ]
                .to_vec()
            },
        }
    }
}

#[test]
fn ParseUri() {
    let harness = UriTest::new();
    let tests = harness.uri_tests;
    for test in tests {
        let uri = if test.uri.is_some() {
            parse_uri(test.uri.as_ref().unwrap().as_slice())
        } else {
            Uri::new()
        };
        if let Err(x) = UriIsExpected(test.expected, &uri) {
            println!("{}", x);
            println!("Failed URI = {:?}", test.uri.unwrap());
            assert!(false);
        }
    }
}

#[test]
fn ParseHostPort_1() {
    let mut i = Bstr::from("www.example.com");
    let e = Bstr::from("www.example.com");
    let (_, (host, port, valid)) = parse_hostport(&mut i).unwrap();

    assert!(e.eq_nocase(host));
    assert!(port.is_none());
    assert!(valid);
}

#[test]
fn ParseHostPort_2() {
    let mut i = Bstr::from(" www.example.com ");
    let e = Bstr::from("www.example.com");
    let (_, (host, port, valid)) = parse_hostport(&mut i).unwrap();

    assert!(e.eq_nocase(host));
    assert!(port.is_none());
    assert!(valid);
}

#[test]
fn ParseHostPort_3() {
    let mut i = Bstr::from(" www.example.com:8001 ");
    let e = Bstr::from("www.example.com");
    let (_, (host, port, valid)) = parse_hostport(&mut i).unwrap();

    assert!(e.eq_nocase(host));
    assert_eq!(8001, port.unwrap().1.unwrap());
    assert!(valid);
}

#[test]
fn ParseHostPort_4() {
    let mut i = Bstr::from(" www.example.com :  8001 ");
    let e = Bstr::from("www.example.com");
    let (_, (host, port, valid)) = parse_hostport(&mut i).unwrap();

    assert!(e.eq_nocase(host));
    assert_eq!(8001, port.unwrap().1.unwrap());
    assert!(valid);
}

#[test]
fn ParseHostPort_5() {
    let mut i = Bstr::from("www.example.com.");
    let e = Bstr::from("www.example.com.");
    let (_, (host, port, valid)) = parse_hostport(&mut i).unwrap();

    assert!(e.eq_nocase(host));
    assert!(port.is_none());
    assert!(valid);
}

#[test]
fn ParseHostPort_6() {
    let mut i = Bstr::from("www.example.com.:8001");
    let e = Bstr::from("www.example.com.");
    let (_, (host, port, valid)) = parse_hostport(&mut i).unwrap();

    assert!(e.eq_nocase(host));
    assert_eq!(8001, port.unwrap().1.unwrap());
    assert!(valid);
}

#[test]
fn ParseHostPort_7() {
    let mut i = Bstr::from("www.example.com:");
    let e = Bstr::from("www.example.com");
    let (_, (host, port, valid)) = parse_hostport(&mut i).unwrap();

    assert!(e.eq_nocase(host));
    assert!(port.is_none());
    assert!(!valid);
}

#[test]
fn ParseHostPort_8() {
    let mut i = Bstr::from("www.example.com:ff");
    let e = Bstr::from("www.example.com");
    let (_, (host, port, valid)) = parse_hostport(&mut i).unwrap();

    assert!(e.eq_nocase(host));
    assert!(port.unwrap().1.is_none());
    assert!(!valid);
}

#[test]
fn ParseHostPort_9() {
    let mut i = Bstr::from("www.example.com:0");
    let e = Bstr::from("www.example.com");
    let (_, (host, port, valid)) = parse_hostport(&mut i).unwrap();

    assert!(e.eq_nocase(host));
    assert!(port.unwrap().1.is_none());
    assert!(!valid);
}

#[test]
fn ParseHostPort_10() {
    let mut i = Bstr::from("www.example.com:65536");
    let e = Bstr::from("www.example.com");
    let (_, (host, port, valid)) = parse_hostport(&mut i).unwrap();

    assert!(e.eq_nocase(host));
    assert!(port.unwrap().1.is_none());
    assert!(!valid);
}

#[test]
fn ParseHostPort_11() {
    let mut i = Bstr::from("[::1]:8080");
    let e = Bstr::from("[::1]");
    let (_, (host, port, valid)) = parse_hostport(&mut i).unwrap();

    assert!(e.eq_nocase(host));
    assert_eq!(8080, port.unwrap().1.unwrap());
    assert!(valid);
}

#[test]
fn ParseHostPort_12() {
    let mut i = Bstr::from("[::1]:");
    let e = Bstr::from("[::1]");
    let (_, (host, port, valid)) = parse_hostport(&mut i).unwrap();

    assert!(e.eq_nocase(host));
    assert!(port.is_none());
    assert!(!valid);
}

#[test]
fn ParseHostPort_13() {
    let mut i = Bstr::from("[::1]x");
    let e = Bstr::from("[::1]");
    let (_, (host, port, valid)) = parse_hostport(&mut i).unwrap();

    assert!(e.eq_nocase(host));
    assert!(port.is_none());
    assert!(!valid);
}

#[test]
fn ParseHostPort_14() {
    let mut i = Bstr::from("[::1");
    let e = Bstr::from("[::1");
    let (_, (host, port, valid)) = parse_hostport(&mut i).unwrap();

    assert!(e.eq_nocase(host));
    assert!(port.is_none());
    assert!(!valid);
}

#[test]
fn ParseScheme_1() {
    let i: &[u8] = b"http://user:pass@www.example.com:1234/path1/path2?a=b&c=d#frag";
    let o: &[u8] = b"//user:pass@www.example.com:1234/path1/path2?a=b&c=d#frag";
    let e: &[u8] = b"http";
    let (left, scheme) = scheme()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(scheme, e);
}

#[test]
fn ParseInvalidScheme() {
    let i: &[u8] = b"/http://user:pass@www.example.com:1234/path1/path2?a=b&c=d#frag";
    assert!(!scheme()(i).is_ok());
}

#[test]
fn ParseCredentials_1() {
    let i: &[u8] = b"//user:pass@www.example.com:1234/path1/path2?a=b&c=d#frag";
    let o: &[u8] = b"www.example.com:1234/path1/path2?a=b&c=d#frag";
    let u: &[u8] = b"user";
    let p: &[u8] = b"pass";
    let (left, (user, pass)) = credentials()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(user, u);
    assert_eq!(pass.unwrap(), p);
}

#[test]
fn ParseCredentials_2() {
    let i: &[u8] = b"//user@www.example.com:1234/path1/path2?a=b&c=d#frag";
    let o: &[u8] = b"www.example.com:1234/path1/path2?a=b&c=d#frag";
    let u: &[u8] = b"user";
    let (left, (user, pass)) = credentials()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(user, u);
    assert!(pass.is_none());
}

#[test]
fn ParseInvalidCredentials() {
    //Must have already parsed the scheme!
    let i: &[u8] = b"http://user:pass@www.example.com:1234/path1/path2?a=b&c=d#frag";
    assert!(!credentials()(i).is_ok());
}

#[test]
fn ParseHostname_1() {
    let i: &[u8] = b"www.example.com:1234/path1/path2?a=b&c=d#frag";
    let o: &[u8] = b":1234/path1/path2?a=b&c=d#frag";
    let e: &[u8] = b"www.example.com";
    let (left, hostname) = hostname()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(hostname, e);
}

#[test]
fn ParseHostname_2() {
    let i: &[u8] = b"www.example.com/path1/path2?a=b&c=d#frag";
    let o: &[u8] = b"/path1/path2?a=b&c=d#frag";
    let e: &[u8] = b"www.example.com";
    let (left, hostname) = hostname()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(hostname, e);
}

#[test]
fn ParseHostname_3() {
    let i: &[u8] = b"www.example.com?a=b&c=d#frag";
    let o: &[u8] = b"?a=b&c=d#frag";
    let e: &[u8] = b"www.example.com";
    let (left, hostname) = hostname()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(hostname, e);
}

#[test]
fn ParseHostname_4() {
    let i: &[u8] = b"www.example.com#frag";
    let o: &[u8] = b"#frag";
    let e: &[u8] = b"www.example.com";
    let (left, hostname) = hostname()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(hostname, e);
}

#[test]
fn ParseHostname_5() {
    let i: &[u8] = b"[::1]:8080";
    let o: &[u8] = b":8080";
    let e: &[u8] = b"[::1]";
    let (left, hostname) = hostname()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(hostname, e);
}

#[test]
fn ParseHostname_6() {
    let i: &[u8] = b"[::1";
    let o: &[u8] = b"";
    let e: &[u8] = b"[::1";
    let (left, hostname) = hostname()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(hostname, e);
}

#[test]
fn ParseHostname_7() {
    let i: &[u8] = b"[::1/path1[0]";
    let o: &[u8] = b"/path1[0]";
    let e: &[u8] = b"[::1";
    let (left, hostname) = hostname()(i).unwrap();

    assert_eq!(left, o);
    assert_eq!(hostname, e);
}

#[test]
fn ParseHostname_8() {
    let i: &[u8] = b"[::1]xxxx";
    let o: &[u8] = b"xxxx";
    let e: &[u8] = b"[::1]";
    let (left, hostname) = hostname()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(hostname, e);
}

#[test]
fn ParseInvalidHostname() {
    //If it starts with '/' we treat it as a path
    let i: &[u8] = b"/www.example.com/path1/path2?a=b&c=d#frag";
    assert!(!hostname()(i).is_ok());
}

#[test]
fn ParsePort_1() {
    let i: &[u8] = b":1234/path1/path2?a=b&c=d#frag";
    let o: &[u8] = b"/path1/path2?a=b&c=d#frag";
    let e: &[u8] = b"1234";
    let (left, path) = port()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(path, e);
}

#[test]
fn ParsePort_2() {
    let i: &[u8] = b":1234?a=b&c=d#frag";
    let o: &[u8] = b"?a=b&c=d#frag";
    let e: &[u8] = b"1234";
    let (left, path) = port()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(path, e);
}

#[test]
fn ParsePort_3() {
    let i: &[u8] = b":1234#frag";
    let o: &[u8] = b"#frag";
    let e: &[u8] = b"1234";
    let (left, path) = port()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(path, e);
}

#[test]
fn ParsePath_1() {
    let i: &[u8] = b"/path1/path2?a=b&c=d#frag";
    let o: &[u8] = b"?a=b&c=d#frag";
    let e: &[u8] = b"/path1/path2";
    let (left, path) = path()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(path, e);
}

#[test]
fn ParsePath_2() {
    let i: &[u8] = b"/path1/path2#frag";
    let o: &[u8] = b"#frag";
    let e: &[u8] = b"/path1/path2";
    let (left, path) = path()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(path, e);
}

#[test]
fn ParsePath_3() {
    let i: &[u8] = b"path1/path2?a=b&c=d#frag";
    let o: &[u8] = b"?a=b&c=d#frag";
    let e: &[u8] = b"path1/path2";
    let (left, path) = path()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(path, e);
}

#[test]
fn ParsePath_4() {
    let i: &[u8] = b"//";
    let o: &[u8] = b"";
    let e: &[u8] = b"//";
    let (left, path) = path()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(path, e);
}

#[test]
fn ParseQuery_1() {
    let i: &[u8] = b"?a=b&c=d#frag";
    let o: &[u8] = b"#frag";
    let e: &[u8] = b"a=b&c=d";
    let (left, query) = query()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(query, e);
}

#[test]
fn ParseQuery_2() {
    let i: &[u8] = b"?a=b&c=d";
    let o: &[u8] = b"";
    let e: &[u8] = b"a=b&c=d";
    let (left, query) = query()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(query, e);
}

#[test]
fn ParseFragment() {
    let i: &[u8] = b"#frag";
    let o: &[u8] = b"";
    let e: &[u8] = b"frag";
    let (left, fragment) = fragment()(i).unwrap();
    assert_eq!(left, o);
    assert_eq!(fragment, e);
}

#[test]
fn ParseContentType() {
    assert_eq!(
        Bstr::from("multipart/form-data"),
        parse_ct_header(b"multipart/form-data").unwrap()
    );
    assert_eq!(
        Bstr::from("multipart/form-data"),
        parse_ct_header(b"multipart/form-data;boundary=X").unwrap()
    );
    assert_eq!(
        Bstr::from("multipart/form-data"),
        parse_ct_header(b"multipart/form-data boundary=X").unwrap()
    );
    assert_eq!(
        Bstr::from("multipart/form-data"),
        parse_ct_header(b"multipart/form-data,boundary=X").unwrap()
    );
    assert_eq!(
        Bstr::from("multipart/form-data"),
        parse_ct_header(b"multipart/FoRm-data").unwrap()
    );
    assert_eq!(
        Bstr::from("multipart/form-data\t"),
        parse_ct_header(b"multipart/form-data\t boundary=X").unwrap()
    );
    assert_eq!(
        Bstr::from("multipart/form-data"),
        parse_ct_header(b"   \tmultipart/form-data boundary=X").unwrap()
    );
}

#[test]
fn ValidateHostname_1() {
    assert!(validate_hostname(b"www.example.com"));
}

#[test]
fn ValidateHostname_2() {
    assert!(!validate_hostname(b".www.example.com"));
}

#[test]
fn ValidateHostname_3() {
    assert!(!validate_hostname(b"www..example.com"));
}

#[test]
fn ValidateHostname_4() {
    assert!(!validate_hostname(b"www.example.com.."));
}

#[test]
fn ValidateHostname_5() {
    assert!(!validate_hostname(b"www example com"));
}

#[test]
fn ValidateHostname_6() {
    assert!(!validate_hostname(b""));
}

#[test]
fn ValidateHostname_7() {
    // Label over 63 characters.
    assert!(!validate_hostname(
        b"www.exampleexampleexampleexampleexampleexampleexampleexampleexampleexample.com"
    ));
}

#[test]
fn ValidateHostname_8() {
    assert!(validate_hostname(b"www.ExAmplE-1984.com"));
}

#[test]
fn ValidateHostname_9() {
    assert!(validate_hostname(b"[:::]"));
}

#[test]
fn ValidateHostname_10() {
    assert!(!validate_hostname(b"[:::"));
}

#[test]
fn ValidateHostname_11() {
    assert!(!validate_hostname(b"[:::/path[0]"));
}

#[test]
fn ValidateHostname_12() {
    assert!(!validate_hostname(b"[:::#garbage]"));
}

#[test]
fn ValidateHostname_13() {
    assert!(!validate_hostname(b"[:::?]"));
}

struct DecodingTest {
    connp: *mut ConnectionParser,
    cfg: *mut config::Config,
    tx: *mut Transaction,
}

impl DecodingTest {
    fn new() -> Self {
        let mut ret = Self {
            connp: std::ptr::null_mut(),
            cfg: std::ptr::null_mut(),
            tx: std::ptr::null_mut(),
        };
        unsafe {
            ret.cfg = config::create();
            ret.connp = htp_connp_create(ret.cfg);
            (*ret.connp).open(
                Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                Some(32768),
                Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                Some(80),
                None,
            );
            let tx_id = (*ret.connp).create_tx().unwrap();
            ret.tx = (*ret.connp).conn.tx_mut_ptr(tx_id);
            (*ret.tx).parsed_uri = Some(Uri::new());
        }
        ret
    }
}

impl Drop for DecodingTest {
    fn drop(&mut self) {
        unsafe {
            htp_connp_destroy_all(self.connp);
            (*self.cfg).destroy();
        }
    }
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace1_Identity() {
    let mut i = Bstr::from("/dest");
    let e = Bstr::from("/dest");
    unsafe {
        let test = DecodingTest::new();
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace2_Urlencoded() {
    let mut i = Bstr::from("/%64est");
    let e = Bstr::from("/dest");
    unsafe {
        let test = DecodingTest::new();
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace3_UrlencodedInvalidPreserve() {
    let mut i = Bstr::from("/%xxest");
    let e = Bstr::from("/%xxest");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PRESERVE_PERCENT,
        );
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace4_UrlencodedInvalidRemove() {
    let mut i = Bstr::from("/%xxest");
    let e = Bstr::from("/xxest");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_REMOVE_PERCENT,
        );
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace5_UrlencodedInvalidDecode() {
    let mut i = Bstr::from("/%}9est");
    let e = Bstr::from("/iest");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PROCESS_INVALID,
        );
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace6_UrlencodedInvalidNotEnoughBytes() {
    let mut i = Bstr::from("/%a");
    let e = Bstr::from("/%a");
    unsafe {
        let test = DecodingTest::new();
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace7_UrlencodedInvalidNotEnoughBytes() {
    let mut i = Bstr::from("/%");
    let e = Bstr::from("/%");
    unsafe {
        let test = DecodingTest::new();
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace8_Uencoded() {
    let mut i = Bstr::from("/%u0064");
    let e = Bstr::from("/d");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace9_UencodedDoNotDecode() {
    let mut i = Bstr::from("/%u0064");
    let e = Bstr::from("/%u0064");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(false);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PRESERVE_PERCENT,
        );
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace10_UencodedInvalidNotEnoughBytes() {
    let mut i = Bstr::from("/%u006");
    let e = Bstr::from("/%u006");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PROCESS_INVALID,
        );
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace11_UencodedInvalidPreserve() {
    let mut i = Bstr::from("/%u006");
    let e = Bstr::from("/%u006");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PRESERVE_PERCENT,
        );
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace12_UencodedInvalidRemove() {
    let mut i = Bstr::from("/%uXXXX");
    let e = Bstr::from("/uXXXX");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_REMOVE_PERCENT,
        );
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace13_UencodedInvalidDecode() {
    let mut i = Bstr::from("/%u00}9");
    let e = Bstr::from("/i");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PROCESS_INVALID,
        );
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace14_UencodedInvalidPreserve() {
    let mut i = Bstr::from("/%u00");
    let e = Bstr::from("/%u00");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PRESERVE_PERCENT,
        );
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace15_UencodedInvalidPreserve() {
    let mut i = Bstr::from("/%u0");
    let e = Bstr::from("/%u0");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PRESERVE_PERCENT,
        );
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace16_UencodedInvalidPreserve() {
    let mut i = Bstr::from("/%u");
    let e = Bstr::from("/%u");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PRESERVE_PERCENT,
        );
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace17_UrlencodedNul() {
    let mut i = Bstr::from("/%00");
    let e = Bstr::from("/\0");
    unsafe {
        let test = DecodingTest::new();
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace18_UrlencodedNulTerminates() {
    let mut i = Bstr::from("/%00ABC");
    let e = Bstr::from("/");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_nul_encoded_terminates(true);
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace19_RawNulTerminates() {
    let mut i = Bstr::from("/\0ABC");
    let e = Bstr::from("/");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_nul_raw_terminates(true);
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTes_DecodeUrlencodedInplace20_UencodedBestFit() {
    let mut i = Bstr::from("/%u0107");
    let e = Bstr::from("/c");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        tx_urldecode_params_inplace(&mut *test.tx, &mut i).unwrap();
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodeUrlencodedInplace21_UencodedCaseInsensitive() {
    let mut i_lower = Bstr::from("/%u0064");
    let mut i_upper = Bstr::from("/%U0064");
    let e = Bstr::from("/d");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        tx_urldecode_params_inplace(&mut *test.tx, &mut i_lower).unwrap();
        tx_urldecode_params_inplace(&mut *test.tx, &mut i_upper).unwrap();
    }
    assert_eq!(i_upper, e);
    assert_eq!(i_lower, e);
}

#[test]
fn DecodingTest_DecodePathInplace1_UrlencodedInvalidNotEnoughBytes() {
    let mut i = Bstr::from("/%a");
    let e = Bstr::from("/%a");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PROCESS_INVALID,
        );
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_INVALID_ENCODING));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace2_UencodedInvalidNotEnoughBytes() {
    let mut i = Bstr::from("/%uX");
    let e = Bstr::from("/%uX");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PROCESS_INVALID,
        );
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_INVALID_ENCODING));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace3_UencodedValid() {
    let mut i = Bstr::from("/%u0107");
    let e = Bstr::from("/c");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PROCESS_INVALID,
        );
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace4_UencodedInvalidNotHexDigits_Remove() {
    let mut i = Bstr::from("/%uXXXX");
    let e = Bstr::from("/uXXXX");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_REMOVE_PERCENT,
        );
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_INVALID_ENCODING));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace5_UencodedInvalidNotHexDigits_Preserve() {
    let mut i = Bstr::from("/%uXXXX");
    let e = Bstr::from("/%uXXXX");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PRESERVE_PERCENT,
        );
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_INVALID_ENCODING));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace6_UencodedInvalidNotHexDigits_Process() {
    let mut i = Bstr::from("/%u00}9");
    let e = Bstr::from("/i");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PROCESS_INVALID,
        );
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_INVALID_ENCODING));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace7_UencodedNul() {
    let mut i = Bstr::from("/%u0000");
    let e = Bstr::from("/\0");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PROCESS_INVALID,
        );
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_ENCODED_NUL));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace8_UencodedNotEnough_Remove() {
    let mut i = Bstr::from("/%uXXX");
    let e = Bstr::from("/uXXX");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_REMOVE_PERCENT,
        );
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_INVALID_ENCODING));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace9_UencodedNotEnough_Preserve() {
    let mut i = Bstr::from("/%uXXX");
    let e = Bstr::from("/%uXXX");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PRESERVE_PERCENT,
        );
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_INVALID_ENCODING));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace10_UrlencodedNul() {
    let mut i = Bstr::from("/%00123");
    let e = Bstr::from("/\x00123");
    unsafe {
        let test = DecodingTest::new();
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_ENCODED_NUL));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace11_UrlencodedNul_Terminates() {
    let mut i = Bstr::from("/%00123");
    let e = Bstr::from("/");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_nul_encoded_terminates(true);
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_ENCODED_NUL));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace12_EncodedSlash() {
    let mut i = Bstr::from("/one%2ftwo");
    let e = Bstr::from("/one%2ftwo");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_path_separators_decode(false);
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_ENCODED_SEPARATOR));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace13_EncodedSlash_Decode() {
    let mut i = Bstr::from("/one%2ftwo");
    let e = Bstr::from("/one/two");
    unsafe {
        let test = DecodingTest::new();

        (*test.cfg).set_path_separators_decode(true);
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_ENCODED_SEPARATOR));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace14_Urlencoded_Invalid_Preserve() {
    let mut i = Bstr::from("/%HH");
    let e = Bstr::from("/%HH");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PRESERVE_PERCENT,
        );
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_INVALID_ENCODING));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace15_Urlencoded_Invalid_Remove() {
    let mut i = Bstr::from("/%HH");
    let e = Bstr::from("/HH");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_REMOVE_PERCENT,
        );
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_INVALID_ENCODING));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace16_Urlencoded_Invalid_Process() {
    let mut i = Bstr::from("/%}9");
    let e = Bstr::from("/i");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PROCESS_INVALID,
        );
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_INVALID_ENCODING));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace17_Urlencoded_NotEnough_Remove() {
    let mut i = Bstr::from("/%H");
    let e = Bstr::from("/H");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_REMOVE_PERCENT,
        );
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_INVALID_ENCODING));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace18_Urlencoded_NotEnough_Preserve() {
    let mut i = Bstr::from("/%H");
    let e = Bstr::from("/%H");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PRESERVE_PERCENT,
        );
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_INVALID_ENCODING));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace19_Urlencoded_NotEnough_Process() {
    let mut i = Bstr::from("/%H");
    let e = Bstr::from("/%H");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PROCESS_INVALID,
        );
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
        assert!((*test.tx).flags.contains(Flags::HTP_PATH_INVALID_ENCODING));
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace20_RawNul1() {
    let mut i = Bstr::from("/\x00123");
    let e = Bstr::from("/");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_nul_raw_terminates(true);
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace21_RawNul1() {
    let mut i = Bstr::from("/\x00123");
    let e = Bstr::from("/\x00123");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_nul_raw_terminates(false);
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace22_ConvertBackslash1() {
    let mut i = Bstr::from("/one\\two");
    let e = Bstr::from("/one/two");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_backslash_convert_slashes(true);
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace23_ConvertBackslash2() {
    let mut i = Bstr::from("/one\\two");
    let e = Bstr::from("/one\\two");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_backslash_convert_slashes(false);
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_DecodePathInplace24_CompressSeparators() {
    let mut i = Bstr::from("/one//two");
    let e = Bstr::from("/one/two");
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_path_separators_compress(true);
        decode_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
    }
    assert_eq!(i, e);
}

#[test]
fn DecodingTest_InvalidUtf8() {
    let mut i = Bstr::from(b"\xf1.\xf1\xef\xbd\x9dabcd".to_vec());
    unsafe {
        let test = DecodingTest::new();
        (*test.cfg).set_utf8_convert_bestfit(true);
        utf8_decode_and_validate_uri_path_inplace(
            &(*test.cfg).decoder_cfg,
            &mut (*test.tx).flags,
            &mut (*test.tx).response_status_expected_number,
            &mut i,
        );
    }
    assert!(i.eq("?.?}abcd"));
}

struct UrlEncodedParserTest {
    connp: *mut ConnectionParser,
    cfg: *mut config::Config,
    tx: *mut Transaction,
}

impl UrlEncodedParserTest {
    fn new() -> Self {
        let mut ret = Self {
            connp: std::ptr::null_mut(),
            cfg: std::ptr::null_mut(),
            tx: std::ptr::null_mut(),
        };
        unsafe {
            ret.cfg = config::create();
            ret.connp = htp_connp_create(ret.cfg);
            (*ret.connp).open(
                Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                Some(32768),
                Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                Some(80),
                None,
            );
            let tx_id = (*ret.connp).create_tx().unwrap();
            ret.tx = (*ret.connp).conn.tx_mut_ptr(tx_id);
            ret
        }
    }
}

impl Drop for UrlEncodedParserTest {
    fn drop(&mut self) {
        unsafe {
            htp_connp_destroy_all(self.connp);
            (*self.cfg).destroy();
        }
    }
}

// Start of Url Parser tests.
#[test]
fn UrlencodedParser_Empty() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_complete(&mut urlenp, b"");

    assert_eq!(0, urlenp.params.size());
}

#[test]
fn UrlencodedParser_EmptyKey1() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_complete(&mut urlenp, b"&");

    assert!(urlenp.params.get_nocase("").unwrap().1.eq(""));
    assert_eq!(1, urlenp.params.size());
}

#[test]
fn UrlencodedParser_EmptyKey2() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_complete(&mut urlenp, b"=&");

    assert!(urlenp.params.get_nocase("").unwrap().1.eq(""));
    assert_eq!(1, urlenp.params.size());
}

#[test]
fn UrlencodedParser_EmptyKey3() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_complete(&mut urlenp, b"=1&");

    assert!(urlenp.params.get_nocase("").unwrap().1.eq("1"));
    assert_eq!(1, urlenp.params.size());
}

#[test]
fn UrlencodedParser_EmptyKey4() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_complete(&mut urlenp, b"&=");

    assert!(urlenp.params.get_nocase("").unwrap().1.eq(""));
    assert_eq!(1, urlenp.params.size());
}

#[test]
fn UrlencodedParser_EmptyKey5() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_complete(&mut urlenp, b"&&");

    assert!(urlenp.params.get_nocase("").unwrap().1.eq(""));
    assert_eq!(1, urlenp.params.size());
}

#[test]
fn UrlencodedParser_EmptyKeyAndValue() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_complete(&mut urlenp, b"=");

    assert!(urlenp.params.get_nocase("").unwrap().1.eq(""));
    assert_eq!(1, urlenp.params.size());
}

#[test]
fn UrlencodedParser_OnePairEmptyValue() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_complete(&mut urlenp, b"p=");

    assert!(urlenp.params.get_nocase("p").unwrap().1.eq(""));
    assert_eq!(1, urlenp.params.size());
}

#[test]
fn UrlencodedParser_OnePairEmptyKey() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_complete(&mut urlenp, b"=p");

    assert!(urlenp.params.get_nocase("").unwrap().1.eq("p"));
    assert_eq!(1, urlenp.params.size());
}

#[test]
fn UrlencodedParser_OnePair() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_complete(&mut urlenp, b"p=1");

    assert!(urlenp.params.get_nocase("p").unwrap().1.eq("1"));
    assert_eq!(1, urlenp.params.size());
}

#[test]
fn UrlencodedParser_TwoPairs() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_complete(&mut urlenp, b"p=1&q=2");

    assert!(urlenp.params.get_nocase("p").unwrap().1.eq("1"));
    assert!(urlenp.params.get_nocase("q").unwrap().1.eq("2"));
    assert_eq!(2, urlenp.params.size());
}

#[test]
fn UrlencodedParser_KeyNoValue1() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_complete(&mut urlenp, b"p");

    assert!(urlenp.params.get_nocase("p").unwrap().1.eq(""));
    assert_eq!(1, urlenp.params.size());
}

#[test]
fn UrlencodedParser_KeyNoValue2() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_complete(&mut urlenp, b"p&");

    assert!(urlenp.params.get_nocase("p").unwrap().1.eq(""));
    assert_eq!(1, urlenp.params.size());
}

#[test]
fn UrlencodedParser_KeyNoValue3() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_complete(&mut urlenp, b"p&q");

    assert!(urlenp.params.get_nocase("p").unwrap().1.eq(""));
    assert!(urlenp.params.get_nocase("q").unwrap().1.eq(""));
    assert_eq!(2, urlenp.params.size());
}

#[test]
fn UrlencodedParser_KeyNoValue4() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_complete(&mut urlenp, b"p&q=2");

    assert!(urlenp.params.get_nocase("p").unwrap().1.eq(""));
    assert!(urlenp.params.get_nocase("q").unwrap().1.eq("2"));
    assert_eq!(2, urlenp.params.size());
}

#[test]
fn UrlencodedParser_Partial1() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_partial(&mut urlenp, b"p");
    urlenp_finalize(&mut urlenp);

    assert!(urlenp.params.get_nocase("p").unwrap().1.eq(""));
    assert_eq!(1, urlenp.params.size());
}

#[test]
fn UrlencodedParser_Partial2() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_partial(&mut urlenp, b"p");
    urlenp_parse_partial(&mut urlenp, b"x");
    urlenp_finalize(&mut urlenp);

    assert!(urlenp.params.get_nocase("px").unwrap().1.eq(""));
    assert_eq!(1, urlenp.params.size());
}

#[test]
fn UrlencodedParser_Partial3() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_partial(&mut urlenp, b"p");
    urlenp_parse_partial(&mut urlenp, b"x&");
    urlenp_finalize(&mut urlenp);

    assert!(urlenp.params.get_nocase("px").unwrap().1.eq(""));
    assert_eq!(1, urlenp.params.size());
}

#[test]
fn UrlencodedParser_Partial4() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_partial(&mut urlenp, b"p");
    urlenp_parse_partial(&mut urlenp, b"=");
    urlenp_finalize(&mut urlenp);

    assert!(urlenp.params.get_nocase("p").unwrap().1.eq(""));
    assert_eq!(1, urlenp.params.size());
}

#[test]
fn UrlencodedParser_Partial5() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_partial(&mut urlenp, b"p");
    urlenp_parse_partial(&mut urlenp, b"");
    urlenp_parse_partial(&mut urlenp, b"");
    urlenp_parse_partial(&mut urlenp, b"");
    urlenp_finalize(&mut urlenp);

    assert!(urlenp.params.get_nocase("p").unwrap().1.eq(""));
    assert_eq!(1, urlenp.params.size());
}

#[test]
fn UrlencodedParser_Partial6() {
    let test = UrlEncodedParserTest::new();
    let mut urlenp = Parser::new(test.tx);
    urlenp_parse_partial(&mut urlenp, b"px");
    urlenp_parse_partial(&mut urlenp, b"n");
    urlenp_parse_partial(&mut urlenp, b"");
    urlenp_parse_partial(&mut urlenp, b"=");
    urlenp_parse_partial(&mut urlenp, b"1");
    urlenp_parse_partial(&mut urlenp, b"2");
    urlenp_parse_partial(&mut urlenp, b"&");
    urlenp_parse_partial(&mut urlenp, b"qz");
    urlenp_parse_partial(&mut urlenp, b"n");
    urlenp_parse_partial(&mut urlenp, b"");
    urlenp_parse_partial(&mut urlenp, b"=");
    urlenp_parse_partial(&mut urlenp, b"2");
    urlenp_parse_partial(&mut urlenp, b"3");
    urlenp_parse_partial(&mut urlenp, b"&");
    urlenp_finalize(&mut urlenp);

    assert!(urlenp.params.get_nocase("pxn").unwrap().1.eq("12"));
    assert!(urlenp.params.get_nocase("qzn").unwrap().1.eq("23"));
    assert_eq!(2, urlenp.params.size());
}

#[test]
fn List_Misc() {
    unsafe {
        let mut l = List::with_capacity(16);

        l.push("1".as_ptr() as *mut core::ffi::c_void);
        l.push("2".as_ptr() as *mut core::ffi::c_void);
        l.push("3".as_ptr() as *mut core::ffi::c_void);

        assert_eq!(3, l.len());

        let p: *mut i8 = l.pop().unwrap() as *mut i8;
        assert!(!p.is_null());
        assert_eq!(0, libc::strcmp("3".as_ptr() as *mut i8, p));

        assert_eq!(2, l.len());

        let p = l.pop().unwrap() as *mut i8;
        assert!(!p.is_null());
        assert_eq!(0, libc::strcmp("2".as_ptr() as *mut i8, p));

        let p = l.pop().unwrap() as *mut i8;
        assert!(!p.is_null());
        assert_eq!(0, libc::strcmp("1".as_ptr() as *mut i8, p));

        let p = l.pop();
        assert!(p.is_none());

        drop(&l);
    }
}

#[test]
fn List_Misc2() {
    unsafe {
        let mut l = List::with_capacity(2);

        l.push("1".as_ptr() as *mut core::ffi::c_void);
        l.push("2".as_ptr() as *mut core::ffi::c_void);
        l.push("3".as_ptr() as *mut core::ffi::c_void);

        let p: *mut i8 = *l.get(2).unwrap() as *mut i8;
        assert!(!p.is_null());
        assert_eq!(0, libc::strcmp("3".as_ptr() as *mut i8, p));

        assert_eq!(3, l.len());

        let _ = l.replace(2, "4".as_ptr() as *mut core::ffi::c_void);

        let p = l.pop().unwrap() as *mut i8;
        assert!(!p.is_null());
        assert_eq!(0, libc::strcmp("4".as_ptr() as *mut i8, p));

        drop(&l);
    }
}

#[test]
fn List_Expand1() {
    unsafe {
        let mut l = List::with_capacity(2);

        l.push("1".as_ptr() as *mut core::ffi::c_void);
        l.push("2".as_ptr() as *mut core::ffi::c_void);

        assert_eq!(2, l.len());

        l.push("3".as_ptr() as *mut core::ffi::c_void);

        assert_eq!(3, l.len());

        let p: *mut i8 = *l.get(0).unwrap() as *mut i8;
        assert!(!p.is_null());
        assert_eq!(0, libc::strcmp("1".as_ptr() as *mut i8, p));

        let p = *l.get(1).unwrap() as *mut i8;
        assert!(!p.is_null());
        assert_eq!(0, libc::strcmp("2".as_ptr() as *mut i8, p));

        let p = *l.get(2).unwrap() as *mut i8;
        assert!(!p.is_null());
        assert_eq!(0, libc::strcmp("3".as_ptr() as *mut i8, p));

        drop(&l);
    }
}

#[test]
fn List_Expand2() {
    unsafe {
        let mut l = List::with_capacity(2);

        l.push("1".as_ptr() as *mut core::ffi::c_void);
        l.push("2".as_ptr() as *mut core::ffi::c_void);

        assert_eq!(2, l.len());

        l.push("3".as_ptr() as *mut core::ffi::c_void);
        l.push("4".as_ptr() as *mut core::ffi::c_void);

        assert_eq!(4, l.len());

        let p: *mut i8 = *l.get(0).unwrap() as *mut i8;
        assert!(!p.is_null());
        assert_eq!(0, libc::strcmp("1".as_ptr() as *mut i8, p));

        let p = *l.get(1).unwrap() as *mut i8;
        assert!(!p.is_null());
        assert_eq!(0, libc::strcmp("2".as_ptr() as *mut i8, p));

        let p = *l.get(2).unwrap() as *mut i8;
        assert!(!p.is_null());
        assert_eq!(0, libc::strcmp("3".as_ptr() as *mut i8, p));

        let p = l.pop().unwrap() as *mut i8;
        assert!(!p.is_null());
        assert_eq!(0, libc::strcmp("4".as_ptr() as *mut i8, p));

        drop(&l);
    }
}

#[test]
fn Table_Misc() {
    let mut t: Table<&str> = Table::with_capacity(2);

    let mut pkey = Bstr::with_capacity(1);
    pkey.add("p");

    let mut qkey = Bstr::with_capacity(1);
    qkey.add("q");

    t.add(pkey, "1");
    t.add(qkey, "2");

    assert!(t.get_nocase("z").is_none());
    assert_eq!("1", t.get_nocase("p").unwrap().1);
}

#[test]
fn Util_NormalizeUriPath() {
    let mut s = Bstr::from("/a/b/c/./../../g");
    normalize_uri_path_inplace(&mut s);
    assert!(s.eq("/a/g"));

    let mut s = Bstr::from("mid/content=5/../6");
    normalize_uri_path_inplace(&mut s);
    assert!(s.eq("mid/6"));

    let mut s = Bstr::from("./one");
    normalize_uri_path_inplace(&mut s);
    assert!(s.eq("one"));

    let mut s = Bstr::from("../one");
    normalize_uri_path_inplace(&mut s);
    assert!(s.eq("one"));

    let mut s = Bstr::from(".");
    normalize_uri_path_inplace(&mut s);
    assert!(s.eq(""));

    let mut s = Bstr::from("..");
    normalize_uri_path_inplace(&mut s);
    assert!(s.eq(""));

    let mut s = Bstr::from("one/.");
    normalize_uri_path_inplace(&mut s);
    assert!(s.eq("one"));

    let mut s = Bstr::from("one/..");
    normalize_uri_path_inplace(&mut s);
    assert!(s.eq(""));

    let mut s = Bstr::from("one/../");
    normalize_uri_path_inplace(&mut s);
    assert!(s.eq(""));

    let mut s = Bstr::from("/../../../images.gif");
    normalize_uri_path_inplace(&mut s);
    assert!(s.eq("/images.gif"));
}

#[test]
fn UrlencodedParser_UrlDecode1() {
    unsafe {
        let test = UrlEncodedParserTest::new();
        let mut flags: Flags = Flags::empty();

        let mut s = Bstr::from("/one/tw%u006f/three/%u123");
        let mut e = Bstr::from("/one/two/three/%u123");
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PRESERVE_PERCENT,
        );
        urldecode_inplace(&(*test.cfg).decoder_cfg, &mut s, &mut flags).unwrap();
        assert_eq!(e, s);

        s = Bstr::from("/one/tw%u006f/three/%uXXXX");
        e = Bstr::from("/one/two/three/%uXXXX");
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PRESERVE_PERCENT,
        );
        urldecode_inplace(&(*test.cfg).decoder_cfg, &mut s, &mut flags).unwrap();
        assert_eq!(e, s);

        s = Bstr::from("/one/tw%u006f/three/%u123");
        e = Bstr::from("/one/two/three/u123");
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_REMOVE_PERCENT,
        );
        urldecode_inplace(&(*test.cfg).decoder_cfg, &mut s, &mut flags).unwrap();
        assert_eq!(e, s);

        s = Bstr::from("/one/tw%u006f/three/%3");
        e = Bstr::from("/one/two/three/3");
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_REMOVE_PERCENT,
        );
        urldecode_inplace(&(*test.cfg).decoder_cfg, &mut s, &mut flags).unwrap();
        assert_eq!(e, s);

        s = Bstr::from("/one/tw%u006f/three/%3");
        e = Bstr::from("/one/two/three/%3");
        (*test.cfg).set_u_encoding_decode(true);
        (*test.cfg).set_url_encoding_invalid_handling(
            config::htp_url_encoding_handling_t::HTP_URL_DECODE_PROCESS_INVALID,
        );
        urldecode_inplace(&(*test.cfg).decoder_cfg, &mut s, &mut flags).unwrap();
        assert_eq!(e, s);
    }
}

#[test]
fn TakeUntilNull() {
    assert_eq!(
        Ok(("\0   ".as_bytes(), "hello_world  ".as_bytes())),
        take_until_null(b"hello_world  \0   ")
    );
    assert_eq!(
        Ok(("\0\0\0\0".as_bytes(), "hello".as_bytes())),
        take_until_null(b"hello\0\0\0\0")
    );
    assert_eq!(Ok(("\0".as_bytes(), "".as_bytes())), take_until_null(b"\0"));
}

#[test]
fn TakeIsSpaceTrailing() {
    assert_eq!(
        Ok(("w0rd".as_bytes(), "   ".as_bytes())),
        take_is_space_trailing(b"w0rd   ")
    );
    assert_eq!(
        Ok(("word".as_bytes(), "   \t".as_bytes())),
        take_is_space_trailing(b"word   \t")
    );
    assert_eq!(
        Ok(("w0rd".as_bytes(), "".as_bytes())),
        take_is_space_trailing(b"w0rd")
    );
    assert_eq!(
        Ok(("\t  w0rd".as_bytes(), "   ".as_bytes())),
        take_is_space_trailing(b"\t  w0rd   ")
    );
    assert_eq!(
        Ok(("".as_bytes(), "     ".as_bytes())),
        take_is_space_trailing(b"     ")
    );
}

fn TakeIsSpace() {
    assert_eq!(
        Ok(("hello".as_bytes(), "   ".as_bytes())),
        take_is_space(b"   hello")
    );
    assert_eq!(
        Ok(("hell o".as_bytes(), "   \t".as_bytes())),
        take_is_space(b"   \thell o")
    );
    assert_eq!(
        Ok(("hell o".as_bytes(), "".as_bytes())),
        take_is_space(b"hell o")
    );
    assert_eq!(
        Ok(("hell o".as_bytes(), "\r\x0b".as_bytes())),
        take_is_space(b"\r\x0bhell o")
    );
    assert_eq!(
        Ok(("hell \to".as_bytes(), "\r\x0b  \t".as_bytes())),
        take_is_space(b"\r\x0b  \thell \to")
    )
}

#[test]
fn TreatResponseLineAsBody() {
    assert_eq!(false, treat_response_line_as_body(b"   http 1.1"));
    assert_eq!(false, treat_response_line_as_body(b"http"));
    assert_eq!(false, treat_response_line_as_body(b"HTTP"));
    assert_eq!(false, treat_response_line_as_body(b"    HTTP"));
    assert_eq!(true, treat_response_line_as_body(b"test"));
    assert_eq!(true, treat_response_line_as_body(b"     test"));
    assert_eq!(true, treat_response_line_as_body(b""));
    assert_eq!(true, treat_response_line_as_body(b"kfgjl  hTtp "));
}

#[test]
fn RemoveLWS() {
    assert_eq!(
        Ok(("hello".as_bytes(), "   ".as_bytes())),
        take_is_space(b"   hello")
    );
    assert_eq!(
        Ok(("hell o".as_bytes(), "   \t".as_bytes())),
        take_is_space(b"   \thell o")
    );
    assert_eq!(
        Ok(("hell o".as_bytes(), "".as_bytes())),
        take_is_space(b"hell o")
    );
}

#[test]
fn SplitByColon() {
    assert_eq!(
        Ok(("Content-Length".as_bytes(), "230".as_bytes())),
        split_by_colon(b"Content-Length: 230")
    );
    assert_eq!(
        Ok(("".as_bytes(), "No header name".as_bytes())),
        split_by_colon(b":No header name")
    );
    assert_eq!(
        Ok(("Header@Name".as_bytes(), "Not Token".as_bytes())),
        split_by_colon(b"Header@Name: Not Token")
    );
    assert_eq!(
        Err(Error(("No colon".as_bytes(), TakeUntil))),
        split_by_colon(b"No colon")
    );
}

#[test]
fn IsWordToken() {
    assert_eq!(true, is_word_token(b"allalpha"));
    assert_eq!(true, is_word_token(b"alpha567numeric1234"));
    assert_eq!(false, is_word_token(b"alpha{}"));
    assert_eq!(false, is_word_token(b"\n"));
    assert_eq!(true, is_word_token(b"234543"));
    assert_eq!(false, is_word_token(b"abcdeg\t"));
    assert_eq!(true, is_word_token(b"content-length"));
}
