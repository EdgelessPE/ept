pub trait Interpretable {
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String;
}
