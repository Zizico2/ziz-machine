mod ziz_machine;

pub struct State {
    pub on_enter: Box<dyn FnMut() -> ()>,
    pub name: String,
    pub on_exit: Box<dyn FnMut() -> ()>,
}

pub struct Transition {
    pub previous_state: String,
    pub symbol: Option<String>,
    pub output: Box<dyn FnMut() -> ()>,
    pub following_state: String,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2, 2);
    }
}
