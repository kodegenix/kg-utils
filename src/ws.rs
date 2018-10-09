#[macro_export]
macro_rules! strip_ws_json {
    ($input: expr) => {
        $crate::strip_whitespace($input, &[("\"", "\"")], &["\\\\", "\\\""])
    }
}

#[macro_export]
macro_rules! json_eq {
    ($a: expr, $b: expr) => {
        strip_ws_json!($a.as_ref()) == strip_ws_json!($b.as_ref())
    }
}

pub fn strip_whitespace(input: &str, quotes: &[(&str, &str)], ignored: &[&str]) -> String {
    let mut out = String::with_capacity(input.len());

    let mut inp = input.chars();
    let mut quote: Option<&(&str, &str)> = None;

    loop {
        if let Some(q) = quote {
            let mut ignore = false;
            for i in ignored.iter() {
                if inp.as_str().starts_with(i) {
                    out.push_str(i);
                    inp = inp.as_str()[i.len()..].chars();
                    ignore = true;
                    break;
                }
            }
            if !ignore {
                if inp.as_str().starts_with(q.1) {
                    out.push_str(q.1);
                    inp = inp.as_str()[q.1.len()..].chars();
                    quote = None;
                } else {
                    if let Some(c) = inp.next() {
                        out.push(c);
                    } else {
                        break;
                    }
                }
            }
        } else {
            for q in quotes.iter() {
                if inp.as_str().starts_with(q.0) {
                    out.push_str(q.0);
                    inp = inp.as_str()[q.0.len()..].chars();
                    quote = Some(q);
                    break;
                }
            }
            if quote.is_none() {
                if let Some(c) = inp.next() {
                    if !c.is_whitespace() {
                        out.push(c);
                    }
                } else {
                    break;
                }
            }
        }
    }
    out
}
