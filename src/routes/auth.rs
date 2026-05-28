use crate::{error::AppResult, services, state::AppState};
use askama::Template;
use axum::{
    body::Body,
    extract::{Form, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::Next,
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate<'a> {
    error: Option<&'a str>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    password: String,
}

pub async fn require_auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Response {
    if state.inner.config.password.is_none() && state.inner.config.allow_empty_password {
        return next.run(request).await;
    }
    let session = services::auth::parse_session(
        &state.inner.config,
        headers.get(header::COOKIE).and_then(|v| v.to_str().ok()),
    );
    if session.is_some() {
        next.run(request).await
    } else {
        Redirect::to("/login").into_response()
    }
}

pub async fn login_page() -> AppResult<Html<String>> {
    Ok(Html(LoginTemplate { error: None }.render()?))
}

pub async fn login(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> AppResult<Response> {
    if services::auth::verify_password(&state.inner.config, &form.password) {
        let mut res = Redirect::to("/").into_response();
        let cookie = services::auth::make_session_cookie(&state.inner.config);
        res.headers_mut()
            .insert(header::SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
        Ok(res)
    } else {
        Ok((
            StatusCode::UNAUTHORIZED,
            Html(
                LoginTemplate {
                    error: Some("Invalid password"),
                }
                .render()?,
            ),
        )
            .into_response())
    }
}

pub async fn logout() -> Response {
    let mut res = Redirect::to("/login").into_response();
    res.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&services::auth::expire_session_cookie()).unwrap(),
    );
    res
}
