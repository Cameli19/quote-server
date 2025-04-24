pub struct Quote {
    pub text: &'static str,
    pub author: &'static str,
}

pub const THE_QUOTE: Quote = Quote {
    text: "Without effort, your talent is nothing more than unmet potential",
    author: "Angela Duckworth",
};