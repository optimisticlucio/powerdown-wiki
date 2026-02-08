use crate::art::structs::BaseArt;
use crate::{RootErrors, ServerState, User};
use axum::extract::{OriginalUri, Path, State};
use axum::http;
use axum::response::{IntoResponse, Response};
use http::StatusCode;

/// Add comment under a given post.
#[axum::debug_handler]
pub async fn add_comment(
    Path(art_slug): Path<String>,
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
    body: String,
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_| RootErrors::InternalServerError)?;

    // Who's trying to do this?
    let requesting_user = match User::get_from_cookie_jar(&db_connection, &cookie_jar).await {
        Some(user) => user,
        None => {
            return Err(RootErrors::Unauthorized);
        }
    };

    // What post is this on?
    let requested_post = match BaseArt::get_by_slug(&db_connection, &art_slug).await {
        Some(art) => art,
        None => {
            return Err(RootErrors::NotFound(
                original_uri,
                cookie_jar,
                Some(requesting_user),
            ))
        }
    };

    let sanitized_comment = sanitize_comment_content(&body);

    if !comment_content_is_valid(&sanitized_comment) {
        return Err(RootErrors::BadRequest(
            "Content of comment is invalid.".to_string(),
        ));
    }

    // Lovely! A new comment! Let's post it.
    const POST_COMMENT_QUERY: &str =
        "INSERT INTO art_comment (under_post, poster_id, contents) VALUES ($1,$2,$3);";

    db_connection
        .execute(
            POST_COMMENT_QUERY,
            &[&requested_post.id, &requesting_user.id, &sanitized_comment],
        )
        .await
        .map_err(|err| {
            eprintln!(
                "[POST ART COMMENT] Posting a comment on art ID {} failed. {:?}",
                requested_post.id, err
            );
            RootErrors::InternalServerError
        })?;

    Ok((StatusCode::CREATED, "").into_response())
}

/// Given the textual content of a given comment, cleans up anything that may cause issues for the code.
fn sanitize_comment_content(original_comment: &str) -> String {
    // TODO: Properly sanitize. This is the MINIMUM
    original_comment.trim().to_string()
}

/// Given the textual content of a given comment, returns whether it has anything contentwise that may cause problems.
/// Probably best not to look for specific words, cunthrope problem and all.
fn comment_content_is_valid(comment: &str) -> bool {
    !comment.is_empty()
}
