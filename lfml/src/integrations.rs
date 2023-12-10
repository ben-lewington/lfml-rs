#[cfg(feature = "axum")]
mod axum {
    extern crate alloc;
    use crate::Markup;

    use axum_core::{body::Body, response::IntoResponse};
    use http::{header, Response};

    impl IntoResponse for Markup {
        fn into_response(self) -> Response<Body> {
            ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], self.0).into_response()
        }
    }
}
