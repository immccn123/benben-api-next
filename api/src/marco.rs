#[macro_export]
macro_rules! export_routes {
    () => {
        #[inline]
        pub fn routes() -> Vec<rocket::Route> {
            vec![]
        }
    };

    ($($x:expr),+ $(,)?) => {
        #[inline]
        pub fn routes() -> Vec<rocket::Route> {
            routes![$($x),+]
        }
    };
}
