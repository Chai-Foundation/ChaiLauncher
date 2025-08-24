use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::command;
use anyhow::Result;
use oauth2::{
    AuthUrl, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope,
    TokenUrl,
};
use oauth2::basic::BasicClient;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::storage::StorageManager;
use tokio::sync::oneshot;
use warp::Filter;

// Microsoft OAuth2 endpoints
const MICROSOFT_AUTH_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const MICROSOFT_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const MINECRAFT_AUTH_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MINECRAFT_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";
const XBOX_LIVE_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XBOX_XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

// Microsoft Azure app registration
// To set up your own app:
// 1. Go to https://portal.azure.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade
// 2. Click "New registration"
// 3. Name: "ChaiLauncher" or your preferred name
// 4. Supported account types: "Personal Microsoft accounts only"
// 5. Redirect URI: "Public client/native (mobile & desktop)" -> http://localhost:7931/auth/callback
// 6. After creation, note the "Application (client) ID"
// 7. Go to "Authentication" tab, enable "Allow public client flows"
// 8. Replace CLIENT_ID below with your Application (client) ID
const CLIENT_ID: &str = "cbd5ce66-bb68-4a36-bb3a-6c489107e8e5"; // Replace with your Azure app client ID
const REDIRECT_URI: &str = "http://localhost:7931/auth/callback";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinecraftAccount {
    pub id: String,
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
    pub skin_url: Option<String>,
    pub cape_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OAuthSession {
    pub csrf_token: String,
    pub pkce_verifier: String,
    pub auth_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MicrosoftTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
    token_type: String,
    scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct XboxLiveAuthRequest {
    #[serde(rename = "Properties")]
    properties: XboxLiveAuthProperties,
    #[serde(rename = "RelyingParty")]
    relying_party: String,
    #[serde(rename = "TokenType")]
    token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct XboxLiveAuthProperties {
    #[serde(rename = "AuthMethod")]
    auth_method: String,
    #[serde(rename = "SiteName")]
    site_name: String,
    #[serde(rename = "RpsTicket")]
    rps_ticket: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct XboxLiveAuthResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: XboxDisplayClaims,
}

#[derive(Debug, Serialize, Deserialize)]
struct XboxDisplayClaims {
    xui: Vec<XboxUserInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct XboxUserInfo {
    uhs: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct XSTSAuthRequest {
    #[serde(rename = "Properties")]
    properties: XSTSAuthProperties,
    #[serde(rename = "RelyingParty")]
    relying_party: String,
    #[serde(rename = "TokenType")]
    token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct XSTSAuthProperties {
    #[serde(rename = "SandboxId")]
    sandbox_id: String,
    #[serde(rename = "UserTokens")]
    user_tokens: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MinecraftAuthRequest {
    #[serde(rename = "identityToken")]
    identity_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MinecraftAuthResponse {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MinecraftProfile {
    id: String,
    name: String,
    skins: Option<Vec<MinecraftSkin>>,
    capes: Option<Vec<MinecraftCape>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MinecraftSkin {
    id: String,
    state: String,
    url: String,
    variant: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MinecraftCape {
    id: String,
    state: String,
    url: String,
}

// Global state for OAuth sessions
use lazy_static::lazy_static;
lazy_static! {
    static ref OAUTH_SESSIONS: Mutex<HashMap<String, OAuthSession>> = Mutex::new(HashMap::new());
}

#[command]
pub async fn start_microsoft_oauth() -> Result<String, String> {
    let client = BasicClient::new(
        ClientId::new(CLIENT_ID.to_string()),
        None, // ClientSecret
        AuthUrl::new(MICROSOFT_AUTH_URL.to_string()).unwrap(),
        Some(TokenUrl::new(MICROSOFT_TOKEN_URL.to_string()).unwrap())
    )
    .set_redirect_uri(RedirectUrl::new(REDIRECT_URI.to_string()).unwrap());

    // Generate PKCE challenge
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate CSRF token
    let (auth_url, csrf_token) = client
        .authorize_url(|| CsrfToken::new_random())
        .add_scope(Scope::new("XboxLive.signin".to_string()))
        .add_scope(Scope::new("offline_access".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    let session = OAuthSession {
        csrf_token: csrf_token.secret().clone(),
        pkce_verifier: pkce_verifier.secret().clone(),
        auth_url: auth_url.to_string(),
    };

    // Store session
    let session_id = uuid::Uuid::new_v4().to_string();
    OAUTH_SESSIONS
        .lock()
        .unwrap()
        .insert(session_id.clone(), session.clone());

    Ok(auth_url.to_string())
}

#[command]
pub async fn start_oauth_with_server() -> Result<MinecraftAccount, String> {
    let client = BasicClient::new(
        ClientId::new(CLIENT_ID.to_string()),
        None, // ClientSecret
        AuthUrl::new(MICROSOFT_AUTH_URL.to_string()).unwrap(),
        Some(TokenUrl::new(MICROSOFT_TOKEN_URL.to_string()).unwrap())
    )
    .set_redirect_uri(RedirectUrl::new(REDIRECT_URI.to_string()).unwrap());

    // Generate PKCE challenge
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate CSRF token
    let (auth_url, csrf_token) = client
        .authorize_url(|| CsrfToken::new_random())
        .add_scope(Scope::new("XboxLive.signin".to_string()))
        .add_scope(Scope::new("offline_access".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // Start local server
    let server_future = start_callback_server();
    
    // Open auth URL in browser
    let _ = open::that(auth_url.to_string());
    
    // Wait for callback
    let (code, state) = server_future.await
        .map_err(|e| format!("Failed to get callback: {}", e))?;

    // Verify CSRF token
    if csrf_token.secret() != &state {
        return Err("Invalid CSRF token".to_string());
    }

    // Exchange authorization code for tokens
    let microsoft_token = exchange_code_for_token(&code, pkce_verifier.secret())
        .await
        .map_err(|e| format!("Failed to exchange code: {}", e))?;

    // Complete authentication flow
    complete_authentication_flow(microsoft_token).await
}

#[command]
pub async fn complete_microsoft_oauth(
    session_id: String,
    authorization_code: String,
    csrf_token: String,
) -> Result<MinecraftAccount, String> {
    // Retrieve session
    let session = {
        let sessions = OAUTH_SESSIONS.lock().unwrap();
        sessions.get(&session_id).cloned()
    };

    let session = session.ok_or("Invalid session ID")?;

    // Verify CSRF token
    if session.csrf_token != csrf_token {
        return Err("Invalid CSRF token".to_string());
    }

    // Exchange authorization code for tokens
    let microsoft_token = exchange_code_for_token(&authorization_code, &session.pkce_verifier)
        .await
        .map_err(|e| format!("Failed to exchange code: {}", e))?;

    // Authenticate with Xbox Live
    let xbox_token = authenticate_xbox_live(&microsoft_token.access_token)
        .await
        .map_err(|e| format!("Xbox Live auth failed: {}", e))?;

    // Get XSTS token
    let (xsts_token, user_hash) = get_xsts_token(&xbox_token)
        .await
        .map_err(|e| format!("XSTS auth failed: {}", e))?;

    // Authenticate with Minecraft
    let minecraft_token = authenticate_minecraft(&xsts_token, &user_hash)
        .await
        .map_err(|e| format!("Minecraft auth failed: {}", e))?;

    // Get Minecraft profile
    let profile = get_minecraft_profile(&minecraft_token.access_token)
        .await
        .map_err(|e| format!("Failed to get profile: {}", e))?;

    let account = MinecraftAccount {
        id: profile.id.clone(),
        username: profile.name,
        uuid: profile.id,
        access_token: minecraft_token.access_token,
        refresh_token: microsoft_token.refresh_token,
        expires_at: current_timestamp() + minecraft_token.expires_in,
        skin_url: profile.skins.and_then(|skins| {
            skins.into_iter().find(|s| s.state == "ACTIVE").map(|s| s.url)
        }),
        cape_url: profile.capes.and_then(|capes| {
            capes.into_iter().find(|c| c.state == "ACTIVE").map(|c| c.url)
        }),
    };

    // Clean up session
    OAUTH_SESSIONS.lock().unwrap().remove(&session_id);

    // Store account
    store_minecraft_account(&account)
        .await
        .map_err(|e| format!("Failed to store account: {}", e))?;

    Ok(account)
}

#[command]
pub async fn get_stored_accounts() -> Result<Vec<MinecraftAccount>, String> {
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;

    // Load accounts from storage
    load_minecraft_accounts(&storage)
        .await
        .map_err(|e| format!("Failed to load accounts: {}", e))
}

#[command]
pub async fn refresh_minecraft_token(account_id: String) -> Result<MinecraftAccount, String> {
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;

    let mut accounts = load_minecraft_accounts(&storage)
        .await
        .map_err(|e| format!("Failed to load accounts: {}", e))?;

    let account = accounts
        .iter_mut()
        .find(|a| a.id == account_id)
        .ok_or("Account not found")?;

    // Check if token needs refresh
    if account.expires_at > current_timestamp() + 300 {
        // Token still valid for at least 5 minutes
        return Ok(account.clone());
    }

    // Refresh Microsoft token
    let microsoft_token = refresh_microsoft_token(&account.refresh_token)
        .await
        .map_err(|e| format!("Failed to refresh Microsoft token: {}", e))?;

    // Re-authenticate with Xbox Live and Minecraft
    let xbox_token = authenticate_xbox_live(&microsoft_token.access_token)
        .await
        .map_err(|e| format!("Xbox Live auth failed: {}", e))?;

    let (xsts_token, user_hash) = get_xsts_token(&xbox_token)
        .await
        .map_err(|e| format!("XSTS auth failed: {}", e))?;

    let minecraft_token = authenticate_minecraft(&xsts_token, &user_hash)
        .await
        .map_err(|e| format!("Minecraft auth failed: {}", e))?;

    // Update account
    account.access_token = minecraft_token.access_token;
    account.refresh_token = microsoft_token.refresh_token;
    account.expires_at = current_timestamp() + minecraft_token.expires_in;

    // Clone account before saving to avoid borrow issues
    let updated_account = account.clone();

    // Save updated accounts
    save_minecraft_accounts(&storage, &accounts)
        .await
        .map_err(|e| format!("Failed to save accounts: {}", e))?;

    Ok(updated_account)
}

#[command]
pub async fn remove_minecraft_account(account_id: String) -> Result<(), String> {
    let storage = StorageManager::new().await
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;

    let mut accounts = load_minecraft_accounts(&storage)
        .await
        .map_err(|e| format!("Failed to load accounts: {}", e))?;

    accounts.retain(|a| a.id != account_id);

    save_minecraft_accounts(&storage, &accounts)
        .await
        .map_err(|e| format!("Failed to save accounts: {}", e))?;

    Ok(())
}

// Helper functions

async fn exchange_code_for_token(
    authorization_code: &str,
    pkce_verifier: &str,
) -> Result<MicrosoftTokenResponse> {
    let client = reqwest::Client::new();
    
    let params = [
        ("client_id", CLIENT_ID),
        ("code", authorization_code),
        ("redirect_uri", REDIRECT_URI),
        ("grant_type", "authorization_code"),
        ("code_verifier", pkce_verifier),
    ];

    let response = client
        .post(MICROSOFT_TOKEN_URL)
        .form(&params)
        .send()
        .await?;

    if !response.status().is_success() {
        let text = response.text().await?;
        return Err(anyhow::anyhow!("Token exchange failed: {}", text));
    }

    let token_response: MicrosoftTokenResponse = response.json().await?;
    Ok(token_response)
}

async fn refresh_microsoft_token(refresh_token: &str) -> Result<MicrosoftTokenResponse> {
    let client = reqwest::Client::new();
    
    let params = [
        ("client_id", CLIENT_ID),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
        ("scope", "XboxLive.signin offline_access"),
    ];

    let response = client
        .post(MICROSOFT_TOKEN_URL)
        .form(&params)
        .send()
        .await?;

    if !response.status().is_success() {
        let text = response.text().await?;
        return Err(anyhow::anyhow!("Token refresh failed: {}", text));
    }

    let token_response: MicrosoftTokenResponse = response.json().await?;
    Ok(token_response)
}

async fn authenticate_xbox_live(microsoft_token: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let auth_request = XboxLiveAuthRequest {
        properties: XboxLiveAuthProperties {
            auth_method: "RPS".to_string(),
            site_name: "user.auth.xboxlive.com".to_string(),
            rps_ticket: format!("d={}", microsoft_token),
        },
        relying_party: "http://auth.xboxlive.com".to_string(),
        token_type: "JWT".to_string(),
    };

    let response = client
        .post(XBOX_LIVE_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&auth_request)
        .send()
        .await?;

    if !response.status().is_success() {
        let text = response.text().await?;
        return Err(anyhow::anyhow!("Xbox Live auth failed: {}", text));
    }

    let auth_response: XboxLiveAuthResponse = response.json().await?;
    Ok(auth_response.token)
}

async fn get_xsts_token(xbox_token: &str) -> Result<(String, String)> {
    let client = reqwest::Client::new();

    let auth_request = XSTSAuthRequest {
        properties: XSTSAuthProperties {
            sandbox_id: "RETAIL".to_string(),
            user_tokens: vec![xbox_token.to_string()],
        },
        relying_party: "rp://api.minecraftservices.com/".to_string(),
        token_type: "JWT".to_string(),
    };

    let response = client
        .post(XBOX_XSTS_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&auth_request)
        .send()
        .await?;

    if !response.status().is_success() {
        let text = response.text().await?;
        return Err(anyhow::anyhow!("XSTS auth failed: {}", text));
    }

    let auth_response: XboxLiveAuthResponse = response.json().await?;
    let user_hash = auth_response
        .display_claims
        .xui
        .first()
        .ok_or_else(|| anyhow::anyhow!("No user hash in XSTS response"))?
        .uhs
        .clone();

    Ok((auth_response.token, user_hash))
}

async fn authenticate_minecraft(xsts_token: &str, user_hash: &str) -> Result<MinecraftAuthResponse> {
    let client = reqwest::Client::new();

    let auth_request = MinecraftAuthRequest {
        identity_token: format!("XBL3.0 x={};{}", user_hash, xsts_token),
    };

    let response = client
        .post(MINECRAFT_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&auth_request)
        .send()
        .await?;

    if !response.status().is_success() {
        let text = response.text().await?;
        return Err(anyhow::anyhow!("Minecraft auth failed: {}", text));
    }

    let auth_response: MinecraftAuthResponse = response.json().await?;
    Ok(auth_response)
}

async fn get_minecraft_profile(minecraft_token: &str) -> Result<MinecraftProfile> {
    let client = reqwest::Client::new();

    let response = client
        .get(MINECRAFT_PROFILE_URL)
        .header("Authorization", format!("Bearer {}", minecraft_token))
        .send()
        .await?;

    if !response.status().is_success() {
        let text = response.text().await?;
        return Err(anyhow::anyhow!("Failed to get profile: {}", text));
    }

    let profile: MinecraftProfile = response.json().await?;
    Ok(profile)
}

async fn store_minecraft_account(account: &MinecraftAccount) -> Result<()> {
    let storage = StorageManager::new().await?;
    let mut accounts = load_minecraft_accounts(&storage).await.unwrap_or_default();
    
    // Remove existing account with same ID
    accounts.retain(|a| a.id != account.id);
    
    // Add new account
    let account_to_save = account.clone();
    accounts.push(account_to_save);
    
    save_minecraft_accounts(&storage, &accounts).await
}

async fn load_minecraft_accounts(_storage: &StorageManager) -> Result<Vec<MinecraftAccount>> {
    let accounts_path = crate::storage::get_launcher_dir().join("accounts.json");
    
    if !accounts_path.exists() {
        return Ok(Vec::new());
    }
    
    let content = tokio::fs::read_to_string(&accounts_path).await?;
    let accounts: Vec<MinecraftAccount> = serde_json::from_str(&content)?;
    Ok(accounts)
}

async fn save_minecraft_accounts(
    _storage: &StorageManager,
    accounts: &[MinecraftAccount],
) -> Result<()> {
    let accounts_path = crate::storage::get_launcher_dir().join("accounts.json");
    
    // Ensure parent directory exists
    if let Some(parent) = accounts_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    
    let content = serde_json::to_string_pretty(accounts)?;
    tokio::fs::write(&accounts_path, content).await?;
    Ok(())
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub async fn get_active_account_token() -> Result<Option<String>> {
    let storage = StorageManager::new().await?;
    let accounts = load_minecraft_accounts(&storage).await?;
    
    if let Some(account) = accounts.first() {
        // Check if token is still valid
        if account.expires_at > current_timestamp() + 300 {
            // Token valid for at least 5 minutes
            Ok(Some(account.access_token.clone()))
        } else {
            // Token expired, try to refresh
            match refresh_account_token(account).await {
                Ok(refreshed_account) => Ok(Some(refreshed_account.access_token)),
                Err(e) => {
                    println!("Failed to refresh token: {}", e);
                    Ok(None)
                }
            }
        }
    } else {
        Ok(None)
    }
}

async fn refresh_account_token(account: &MinecraftAccount) -> Result<MinecraftAccount> {
    // Refresh Microsoft token
    let microsoft_token = refresh_microsoft_token(&account.refresh_token).await?;

    // Re-authenticate with Xbox Live and Minecraft
    let xbox_token = authenticate_xbox_live(&microsoft_token.access_token).await?;
    let (xsts_token, user_hash) = get_xsts_token(&xbox_token).await?;
    let minecraft_token = authenticate_minecraft(&xsts_token, &user_hash).await?;

    let refreshed_account = MinecraftAccount {
        id: account.id.clone(),
        username: account.username.clone(),
        uuid: account.uuid.clone(),
        access_token: minecraft_token.access_token,
        refresh_token: microsoft_token.refresh_token,
        expires_at: current_timestamp() + minecraft_token.expires_in,
        skin_url: account.skin_url.clone(),
        cape_url: account.cape_url.clone(),
    };

    // Update stored account
    let storage = StorageManager::new().await?;
    let mut accounts = load_minecraft_accounts(&storage).await?;
    if let Some(stored_account) = accounts.iter_mut().find(|a| a.id == account.id) {
        *stored_account = refreshed_account.clone();
        save_minecraft_accounts(&storage, &accounts).await?;
        Ok(refreshed_account)
    } else {
        Err(anyhow::anyhow!("Account not found in storage"))
    }
}

async fn start_callback_server() -> Result<(String, String)> {
    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));

    let callback = warp::path!("auth" / "callback")
        .and(warp::query::<HashMap<String, String>>())
        .map(move |params: HashMap<String, String>| {
            let code = params.get("code").cloned().unwrap_or_default();
            let state = params.get("state").cloned().unwrap_or_default();
            
            // Send the result through the channel
            if let Some(sender) = tx.lock().unwrap().take() {
                let _ = sender.send((code.clone(), state.clone()));
            }

            warp::reply::html(r#"
                <html>
                <head><title>Authentication Complete</title></head>
                <body>
                    <h1>Authentication Successful!</h1>
                    <p>You can now close this browser window and return to ChaiLauncher.</p>
                    <script>
                        setTimeout(() => window.close(), 3000);
                    </script>
                </body>
                </html>
            "#)
        });

    let routes = callback;
    
    // Start server on port 7931
    let server = warp::serve(routes).run(([127, 0, 0, 1], 7931));
    
    // Run server in background
    tokio::spawn(server);
    
    // Wait for callback
    match tokio::time::timeout(std::time::Duration::from_secs(300), rx).await {
        Ok(Ok((code, state))) => Ok((code, state)),
        Ok(Err(_)) => Err(anyhow::anyhow!("Channel error")),
        Err(_) => Err(anyhow::anyhow!("OAuth timeout")),
    }
}

async fn complete_authentication_flow(microsoft_token: MicrosoftTokenResponse) -> Result<MinecraftAccount, String> {
    // Authenticate with Xbox Live
    let xbox_token = authenticate_xbox_live(&microsoft_token.access_token)
        .await
        .map_err(|e| format!("Xbox Live auth failed: {}", e))?;

    // Get XSTS token
    let (xsts_token, user_hash) = get_xsts_token(&xbox_token)
        .await
        .map_err(|e| format!("XSTS auth failed: {}", e))?;

    // Authenticate with Minecraft
    let minecraft_token = authenticate_minecraft(&xsts_token, &user_hash)
        .await
        .map_err(|e| format!("Minecraft auth failed: {}", e))?;

    // Get Minecraft profile
    let profile = get_minecraft_profile(&minecraft_token.access_token)
        .await
        .map_err(|e| format!("Failed to get profile: {}", e))?;

    let account = MinecraftAccount {
        id: profile.id.clone(),
        username: profile.name,
        uuid: profile.id,
        access_token: minecraft_token.access_token,
        refresh_token: microsoft_token.refresh_token,
        expires_at: current_timestamp() + minecraft_token.expires_in,
        skin_url: profile.skins.and_then(|skins| {
            skins.into_iter().find(|s| s.state == "ACTIVE").map(|s| s.url)
        }),
        cape_url: profile.capes.and_then(|capes| {
            capes.into_iter().find(|c| c.state == "ACTIVE").map(|c| c.url)
        }),
    };

    // Store account
    store_minecraft_account(&account)
        .await
        .map_err(|e| format!("Failed to store account: {}", e))?;

    Ok(account)
}