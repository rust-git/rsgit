pub fn do_nothing() {
    println!("Nothing");
}

pub fn silently_do_nothing() {
    println!("Even more nothing");
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
