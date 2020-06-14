pub fn do_nothing() {
    println!("Nothing");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn does_nothing() {
        super::do_nothing();
    }
}
