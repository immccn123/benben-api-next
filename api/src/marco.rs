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

#[macro_export]
macro_rules! export_routes_group {
    () => {
        #[inline]
        pub fn routes() -> Vec<rocket::Route> {
            vec![]
        }
    };

    ($($x:ident),+ $(,)?) => {
        #[inline]
        pub fn routes() -> Vec<rocket::Route> {
            const COUNT: usize = [$(stringify!($x)),+].len();
            let mut routes = Vec::with_capacity(8 * COUNT);

            $(
                routes.extend($x::routes());
            )+

            routes
        }
    };
}
