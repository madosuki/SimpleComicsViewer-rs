pub fn get_value_with_option_from_ref_cell_option<T, R, F>(data: &std::cell::RefCell<Option<T>>, f: F) -> Option<R>
where F: Fn(&T) -> R {
    match data.borrow().as_ref() {
        Some(v) => Some(f(v)),
        None => Option::None
    }
}

