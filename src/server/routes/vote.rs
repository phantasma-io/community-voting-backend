use anyhow::anyhow;
use axum::{Extension, Json, extract::Query, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{IntoParams, ToSchema};

use crate::{
    data::{Candidate, Vote, verify_vote},
    error::ApiErr,
    server::ApiCtx,
};

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/vote/candidates", axum::routing::get(get_candidates))
        .route("/vote/categories", axum::routing::get(get_categories))
        .route("/vote/check", axum::routing::get(vote_check))
        .route("/vote/submit", axum::routing::post(submit_vote))
}

#[utoipa::path(get, path = "/candidates")]
async fn get_candidates(ctx: Extension<ApiCtx>) -> Result<impl IntoResponse, ApiErr> {
    Ok(Json(ctx.data.candidates.clone()))
}

#[utoipa::path(get, path = "/categories")]
async fn get_categories(ctx: Extension<ApiCtx>) -> Result<impl IntoResponse, ApiErr> {
    Ok(Json(ctx.data.categories.clone()))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
struct VoteCheckParams {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct VoteCheckResult {
    pub votes: Vec<Vote>,
}

#[utoipa::path(get, path = "/check", params(VoteCheckParams))]
async fn vote_check(
    ctx: Extension<ApiCtx>,
    Query(query): Query<VoteCheckParams>,
) -> Result<impl IntoResponse, ApiErr> {
    let votes = ctx.data.get_addr_votes(&query.address);

    Ok(Json(VoteCheckResult { votes }))
}

#[utoipa::path(post, path = "/submit", request_body=Vote)]
async fn submit_vote(
    ctx: Extension<ApiCtx>,
    Json(vote): Json<Vote>,
) -> Result<impl IntoResponse, ApiErr> {
    if ctx.data.vote_exist(&vote) {
        return Err(anyhow!("{} ({}) Already voted.", vote.addr, &vote.category_slug).into());
    }

    // check vote candidate exist
    if !ctx
        .data
        .candidates
        .iter()
        .any(|v| v.slug.eq(&vote.candidate_slug))
    {
        return Err(anyhow!("Vote candidate '{}' does not exist", &vote.candidate_slug).into());
    }

    // check vote category exist
    if !ctx
        .data
        .categories
        .iter()
        .any(|v| v.slug.eq(&vote.category_slug))
    {
        return Err(anyhow!("Vote category '{}' does not exist", &vote.category_slug).into());
    }

    // check if signature valid
    let is_valid = verify_vote(&ctx.config.explorer_api_url, &vote).await?;

    if !is_valid {
        return Err(anyhow!("Signature is not valid").into());
    }

    ctx.data.persist_vote(vote)?;

    Ok((Json(json!({
        "result": "ok"
    }))))
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(get_candidates, get_categories, submit_vote, vote_check),
    components(schemas(Vote, Candidate, VoteCheckParams, VoteCheckResult))
)]
pub struct ApiDoc;
