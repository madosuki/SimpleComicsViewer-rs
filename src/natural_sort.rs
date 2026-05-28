pub fn compare_by_natural(a: &str, b : &str) -> std::cmp::Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    loop {
        match (a_chars.peek(), b_chars.peek()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(a_c), Some(b_c)) => {
                if a_c.is_ascii_digit() && b_c.is_ascii_digit() {
                    let target_a = get_digits(&mut a_chars);
                    let target_b = get_digits(&mut b_chars);
                    let ord = cmp_digits(&target_a, &target_b);
                    if ord != std::cmp::Ordering::Equal {
                        return ord;
                    }
                }  else {
                    let tmp_a = a_chars.next();
                    let tmp_b = b_chars.next();

                    if tmp_a.is_some() && tmp_b.is_some() {
                        let x = tmp_a.unwrap();
                        let y = tmp_b.unwrap();
                        let ord = x.cmp(&y);

                        if ord != std::cmp::Ordering::Equal {
                            return ord;
                        }
                    }
                }
            },
        }
    }
    
}

fn get_digits<T>(iter: &mut std::iter::Peekable<T>) -> String
where T: Iterator<Item = char> {
    let mut tmp = String::new();

    while let Some(&c) = iter.peek() {
        if c.is_ascii_digit() {
            tmp.push(c);
        } else {
            break;
        }

        iter.next();
    }

    tmp
}

fn cmp_digits(a: &str, b: &str) -> std::cmp::Ordering {
    let trimed_a = a.trim_start_matches('0');
    let trimed_b = b.trim_start_matches('0');

    let alt_a = if trimed_a.is_empty() { "0" } else { trimed_a };
    let alt_b = if trimed_b.is_empty() { "0" } else { trimed_b };

    match alt_a.len().cmp(&alt_b.len()) {
        std::cmp::Ordering::Equal => match alt_a.cmp(&alt_b) {
            std::cmp::Ordering::Equal => {
              a.len().cmp(&b.len())
            },
            ord => ord,
        },
        ord => ord,
    }
}
