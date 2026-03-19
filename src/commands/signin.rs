use crate::utils;
use crate::utils::tilt_dir;
use anyhow::{Context, Result, anyhow, bail};
use axum::{Router, extract::Query, routing::get};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use clap::Args;
use rand::{Rng, distributions::Alphanumeric};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, oneshot};
use uuid::Uuid;

#[derive(Debug, Args)]
pub struct Signin {}

#[derive(Debug, Deserialize)]
struct CallbackParams {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OrganizationId {
    id: String,
}

#[derive(Debug, Deserialize)]
struct SignInResponse {
    token: String,
    organization: OrganizationId,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    id_token: Option<String>,
    token_type: Option<String>,
    expires_in: Option<u64>,
    scope: Option<String>,
    organization_id: Option<String>,
}

impl Signin {
    pub async fn run(&self) -> Result<()> {
        let state = Uuid::new_v4().to_string();
        let code_verifier = generate_code_verifier();
        let code_challenge = pkce_s256_challenge(&code_verifier);

        // Local callback server
        let listener = tokio::net::TcpListener::bind("127.0.0.1:56161")
            .await
            .context("failed to bind local callback port")?;
        let local_port = listener
            .local_addr()
            .context("failed to get local callback address")?
            .port();

        let redirect_uri = format!("http://127.0.0.1:{local_port}/callback");
        let (tx, rx) = oneshot::channel::<CallbackParams>();
        let tx = Arc::new(Mutex::new(Some(tx)));

        let app = Router::new().route(
            "/callback",
            get({
                let tx = tx.clone();
                move |Query(params): Query<CallbackParams>| {
                    let tx = tx.clone();
                    async move {
                        if let Some(sender) = tx.lock().await.take() {
                            let _ = sender.send(params);
                        }
                        "Login concluído. Pode fechar esta aba."
                    }
                }
            }),
        );

        let server = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        let base_url = utils::dashboard_url_from_env();
        let authorize_url = format!(
            "{}/oauth/consent?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
            base_url,
            urlencoding("tilt-cli"),
            urlencoding(&redirect_uri),
            urlencoding("openid profile email"),
            urlencoding(&state),
            urlencoding(&code_challenge),
        );

        println!("Opening browser for authentication...");
        open::that(&authorize_url).context("failed to open browser for authentication")?;

        let callback = tokio::time::timeout(Duration::from_secs(180), rx)
            .await
            .context("timed out waiting for OAuth callback")?
            .context("failed to receive OAuth callback")?;

        server.abort();

        if let Some(err) = callback.error {
            return Err(anyhow!(
                "oauth error: {} ({})",
                err,
                callback.error_description.unwrap_or_default()
            ));
        }

        let code = callback
            .code
            .ok_or_else(|| anyhow!("callback did not include authorization code"))?;

        let returned_state = callback.state.unwrap_or_default();
        if returned_state != state {
            bail!("invalid OAuth state");
        }

        let token = exchange_code_for_token(
            utils::url_from_env(),
            &code,
            &redirect_uri,
            &code_verifier,
            "tilt-cli",
            None,
        )
        .await?;

        utils::save_auth_token(&token.access_token)?;
        if let Some(refresh_token) = token.refresh_token {
            utils::save_refresh_token(&refresh_token)?;
        }
        if let Some(org_id) = token.organization_id {
            save_selected_organization_id(&org_id)?;
        }

        println!("Authenticated successfully.");
        Ok(())
    }
}

async fn exchange_code_for_token(
    base_url: &str,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
    client_id: &str,
    client_secret: Option<&str>,
) -> Result<TokenResponse> {
    #[derive(Deserialize)]
    struct ErrBody {
        message: Option<String>,
        error: Option<String>,
        error_description: Option<String>,
    }

    let mut payload = std::collections::HashMap::new();
    payload.insert("grant_type", "authorization_code");
    payload.insert("code", code);
    payload.insert("redirect_uri", redirect_uri);
    payload.insert("client_id", client_id);
    payload.insert("code_verifier", code_verifier);
    payload.insert("client_secret", client_secret.unwrap_or(""));

    let client = Client::builder().timeout(Duration::from_secs(15)).build()?;
    let res = client
        .post(format!("{}/oauth/token", base_url))
        .form(&payload)
        .send()
        .await
        .context("failed to request OAuth token exchange")?;

    if res.status().is_success() {
        let token_response = res
            .json::<TokenResponse>()
            .await
            .context("failed to parse token response")?;
        return Ok(token_response);
    }

    let status = res.status();
    let body = res
        .json::<ErrBody>()
        .await
        .ok()
        .map(|e| {
            e.message
                .or(e.error_description)
                .or(e.error)
                .unwrap_or_else(|| status.to_string())
        })
        .unwrap_or_else(|| status.to_string());

    Err(anyhow!("token exchange failed: {}", body))
}

/// Saves the selected organization ID to a file in the Tilt directory.
///
/// Only the sign-in command should be able to call this function, as it is responsible for
/// authenticating the user and obtaining the organization ID.
fn save_selected_organization_id(id: &str) -> Result<()> {
    let path = tilt_dir()?.join("organization_id_selected");
    fs::write(path, id)?;
    Ok(())
}

fn generate_code_verifier() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

fn pkce_s256_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

fn urlencoding(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
