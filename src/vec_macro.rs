#[macro_export]
macro_rules! new_vec {
    () => { Vec::new() };

    ($($element:expr),*) => {{
        const COUNTER: usize = $crate::count![$($element),*];
        let mut new_vec = Vec::with_capacity(COUNTER);
        $(new_vec.push($element);)*
        new_vec
    }};

    ($($element:expr,)*) => { $crate::new_vec![$($element),*] };

    ($element:expr; $count:expr) => {{
        let mut new_vec = Vec::new();
        new_vec.resize($count, $element);
        new_vec
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! count {
    ($($element:expr),*) => { <[()]>::len(&[$($crate::substitute![$element]),*]) };
}

#[macro_export]
#[doc(hidden)]
macro_rules! substitute {
    ($_element:expr) => {
        ()
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_create_empty_vec() {
        let empty_vec: Vec<u64> = new_vec![];
        assert!(empty_vec.is_empty());
    }

    #[test]
    fn test_create_vec_with_single_value() {
        let new_vec: Vec<u64> = new_vec![37];
        assert!(!new_vec.is_empty());
        assert_eq!(new_vec.len(), 1);
        assert_eq!(new_vec[0], 37);
    }

    #[test]
    fn test_create_vec_with_three_values() {
        let new_vec: Vec<u64> = new_vec![37, 73, 137];
        assert_eq!(new_vec.len(), 3);
        assert_eq!(new_vec, [37, 73, 137]);
    }

    #[test]
    fn test_create_vec_with_trailing_comma() {
        let new_vec = new_vec![
            "adding",
            "trailing",
            "comma",
            "to",
            "make",
            "sure",
            "it",
            "is",
            "processed",
            "correctly",
        ];
        assert_eq!(new_vec.len(), 10);
        assert_eq!(new_vec.last(), Some(&"correctly"));
    }

    #[test]
    fn test_create_vec_with_the_same_element_occurring_n_times() {
        let new_vec = new_vec![0_u64; 1024];
        assert!(!new_vec.is_empty());
        assert_eq!(new_vec.len(), 1024);
        assert_eq!(new_vec[0], 0);
        assert_eq!(new_vec.last(), Some(&0_u64));
    }
}
