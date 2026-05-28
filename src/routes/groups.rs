use crate::{
    db::group_repository,
    domain::group::Group,
    error::{AppError, AppResult},
    state::AppState,
};
use askama::Template;
use axum::{
    extract::{Form, Path, State},
    response::{Html, Redirect},
};
use serde::Deserialize;

#[derive(Template)]
#[template(path = "groups.html")]
struct GroupsTemplate {
    groups: Vec<Group>,
}

#[derive(Deserialize)]
pub struct GroupForm {
    name: String,
}

pub async fn index(State(state): State<AppState>) -> AppResult<Html<String>> {
    Ok(Html(
        GroupsTemplate {
            groups: group_repository::list(&state.inner.db).await?,
        }
        .render()?,
    ))
}

pub async fn create(
    State(state): State<AppState>,
    Form(form): Form<GroupForm>,
) -> AppResult<Redirect> {
    validate_name(&form.name)?;
    group_repository::create(&state.inner.db, &form.name).await?;
    Ok(Redirect::to("/groups"))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<GroupForm>,
) -> AppResult<Redirect> {
    validate_name(&form.name)?;
    group_repository::update(&state.inner.db, id, &form.name).await?;
    Ok(Redirect::to("/groups"))
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Redirect> {
    group_repository::delete(&state.inner.db, id).await?;
    Ok(Redirect::to("/groups"))
}

fn validate_name(name: &str) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::BadRequest("group name is required".to_string()));
    }
    Ok(())
}
