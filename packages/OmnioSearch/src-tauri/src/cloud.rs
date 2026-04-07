use std::collections::HashMap;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn, error};
use reqwest::Client;
use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    TokenResponse, AccessToken, RefreshToken,
};
use chrono::{DateTime, Utc};

use crate::config::{CloudProvider, Config};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFile {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: DateTime<Utc>,
    pub mime_type: String,
    pub provider: CloudProvider,
    pub download_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub is_folder: bool,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub provider: CloudProvider,
}

pub struct CloudManager {
    client: Client,
    credentials: HashMap<CloudProvider, CloudCredentials>,
    config: Config,
}

impl CloudManager {
    pub fn new(config: Config) -> Self {
        let client = Client::new();
        
        Self {
            client,
            credentials: HashMap::new(),
            config,
        }
    }

    pub async fn authenticate_provider(&mut self, provider: &CloudProvider) -> Result<String> {
        info!("☁️ Starting authentication for {:?}", provider);

        match provider {
            CloudProvider::GoogleDrive => self.authenticate_google_drive().await,
            CloudProvider::Dropbox => self.authenticate_dropbox().await,
            CloudProvider::OneDrive => self.authenticate_onedrive().await,
            CloudProvider::NextCloud => self.authenticate_nextcloud().await,
            CloudProvider::TeraBox => self.authenticate_terabox().await,
        }
    }

    async fn authenticate_google_drive(&self) -> Result<String> {
        info!("📱 Authenticating with Google Drive...");

        // Google Drive OAuth2 configuration
        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .unwrap_or_else(|_| "your-google-client-id".to_string());
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .unwrap_or_else(|_| "your-google-client-secret".to_string());

        let auth_url = format!(
            "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}",
            client_id,
            "http://localhost:8080/auth/google/callback",
            "https://www.googleapis.com/auth/drive.readonly"
        );

        debug!("🔗 Google Drive auth URL generated");
        Ok(auth_url)
    }

    async fn authenticate_dropbox(&self) -> Result<String> {
        info!("📦 Authenticating with Dropbox...");

        let client_id = std::env::var("DROPBOX_CLIENT_ID")
            .unwrap_or_else(|_| "your-dropbox-client-id".to_string());

        let auth_url = format!(
            "https://www.dropbox.com/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code",
            client_id,
            "http://localhost:8080/auth/dropbox/callback"
        );

        debug!("🔗 Dropbox auth URL generated");
        Ok(auth_url)
    }

    async fn authenticate_onedrive(&self) -> Result<String> {
        info!("🔵 Authenticating with OneDrive...");

        let client_id = std::env::var("ONEDRIVE_CLIENT_ID")
            .unwrap_or_else(|_| "your-onedrive-client-id".to_string());

        let auth_url = format!(
            "https://login.microsoftonline.com/common/oauth2/v2.0/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}",
            client_id,
            "http://localhost:8080/auth/onedrive/callback",
            "https://graph.microsoft.com/Files.Read"
        );

        debug!("🔗 OneDrive auth URL generated");
        Ok(auth_url)
    }

    async fn authenticate_nextcloud(&self) -> Result<String> {
        info!("☁️ Authenticating with NextCloud...");

        // NextCloud requires server URL configuration
        let server_url = std::env::var("NEXTCLOUD_SERVER_URL")
            .unwrap_or_else(|_| "https://your-nextcloud-server.com".to_string());
        let client_id = std::env::var("NEXTCLOUD_CLIENT_ID")
            .unwrap_or_else(|_| "your-nextcloud-client-id".to_string());

        let auth_url = format!(
            "{}/apps/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=files",
            server_url,
            client_id,
            "http://localhost:8080/auth/nextcloud/callback"
        );

        debug!("🔗 NextCloud auth URL generated for server: {}", server_url);
        Ok(auth_url)
    }

    async fn authenticate_terabox(&self) -> Result<String> {
        info!("📦 Authenticating with TeraBox...");

        let client_id = std::env::var("TERABOX_CLIENT_ID")
            .unwrap_or_else(|_| "your-terabox-client-id".to_string());

        // TeraBox OAuth2 flow (similar to Baidu PCS API)
        let auth_url = format!(
            "https://openapi.baidu.com/oauth/2.0/authorize?client_id={}&redirect_uri={}&response_type=code&scope=netdisk",
            client_id,
            "http://localhost:8080/auth/terabox/callback"
        );

        debug!("🔗 TeraBox auth URL generated");
        Ok(auth_url)
    }

    pub async fn handle_oauth_callback(
        &mut self, 
        provider: &CloudProvider, 
        code: &str
    ) -> Result<()> {
        info!("🔑 Handling OAuth callback for {:?}", provider);

        match provider {
            CloudProvider::GoogleDrive => self.handle_google_drive_callback(code).await,
            CloudProvider::Dropbox => self.handle_dropbox_callback(code).await,
            CloudProvider::OneDrive => self.handle_onedrive_callback(code).await,
            CloudProvider::NextCloud => self.handle_nextcloud_callback(code).await,
            CloudProvider::TeraBox => self.handle_terabox_callback(code).await,
        }
    }

    async fn handle_google_drive_callback(&mut self, code: &str) -> Result<()> {
        debug!("🔑 Processing Google Drive OAuth callback...");

        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .unwrap_or_else(|_| "your-google-client-id".to_string());
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .unwrap_or_else(|_| "your-google-client-secret".to_string());

        let token_request = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", "http://localhost:8080/auth/google/callback"),
        ];

        let response = self.client
            .post("https://oauth2.googleapis.com/token")
            .form(&token_request)
            .send()
            .await?;

        let token_data: serde_json::Value = response.json().await?;
        
        if let Some(access_token) = token_data["access_token"].as_str() {
            let credentials = CloudCredentials {
                access_token: access_token.to_string(),
                refresh_token: token_data["refresh_token"].as_str().map(|s| s.to_string()),
                expires_at: token_data["expires_in"].as_u64().map(|secs| {
                    Utc::now() + chrono::Duration::seconds(secs as i64)
                }),
                provider: CloudProvider::GoogleDrive,
            };

            self.credentials.insert(CloudProvider::GoogleDrive, credentials);
            info!("✅ Google Drive authentication successful");
        } else {
            return Err(anyhow::anyhow!("Failed to get access token from Google Drive"));
        }

        Ok(())
    }

    async fn handle_dropbox_callback(&mut self, code: &str) -> Result<()> {
        debug!("🔑 Processing Dropbox OAuth callback...");

        let client_id = std::env::var("DROPBOX_CLIENT_ID")
            .unwrap_or_else(|_| "your-dropbox-client-id".to_string());
        let client_secret = std::env::var("DROPBOX_CLIENT_SECRET")
            .unwrap_or_else(|_| "your-dropbox-client-secret".to_string());

        let token_request = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", "http://localhost:8080/auth/dropbox/callback"),
        ];

        let response = self.client
            .post("https://api.dropboxapi.com/oauth2/token")
            .form(&token_request)
            .send()
            .await?;

        let token_data: serde_json::Value = response.json().await?;
        
        if let Some(access_token) = token_data["access_token"].as_str() {
            let credentials = CloudCredentials {
                access_token: access_token.to_string(),
                refresh_token: None, // Dropbox uses long-lived tokens
                expires_at: None,
                provider: CloudProvider::Dropbox,
            };

            self.credentials.insert(CloudProvider::Dropbox, credentials);
            info!("✅ Dropbox authentication successful");
        } else {
            return Err(anyhow::anyhow!("Failed to get access token from Dropbox"));
        }

        Ok(())
    }

    async fn handle_onedrive_callback(&mut self, code: &str) -> Result<()> {
        debug!("🔑 Processing OneDrive OAuth callback...");

        let client_id = std::env::var("ONEDRIVE_CLIENT_ID")
            .unwrap_or_else(|_| "your-onedrive-client-id".to_string());
        let client_secret = std::env::var("ONEDRIVE_CLIENT_SECRET")
            .unwrap_or_else(|_| "your-onedrive-client-secret".to_string());

        let token_request = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", "http://localhost:8080/auth/onedrive/callback"),
        ];

        let response = self.client
            .post("https://login.microsoftonline.com/common/oauth2/v2.0/token")
            .form(&token_request)
            .send()
            .await?;

        let token_data: serde_json::Value = response.json().await?;
        
        if let Some(access_token) = token_data["access_token"].as_str() {
            let credentials = CloudCredentials {
                access_token: access_token.to_string(),
                refresh_token: token_data["refresh_token"].as_str().map(|s| s.to_string()),
                expires_at: token_data["expires_in"].as_u64().map(|secs| {
                    Utc::now() + chrono::Duration::seconds(secs as i64)
                }),
                provider: CloudProvider::OneDrive,
            };

            self.credentials.insert(CloudProvider::OneDrive, credentials);
            info!("✅ OneDrive authentication successful");
        } else {
            return Err(anyhow::anyhow!("Failed to get access token from OneDrive"));
        }

        Ok(())
    }

    async fn handle_nextcloud_callback(&mut self, code: &str) -> Result<()> {
        debug!("🔑 Processing NextCloud OAuth callback...");

        let server_url = std::env::var("NEXTCLOUD_SERVER_URL")
            .unwrap_or_else(|_| "https://your-nextcloud-server.com".to_string());
        let client_id = std::env::var("NEXTCLOUD_CLIENT_ID")
            .unwrap_or_else(|_| "your-nextcloud-client-id".to_string());
        let client_secret = std::env::var("NEXTCLOUD_CLIENT_SECRET")
            .unwrap_or_else(|_| "your-nextcloud-client-secret".to_string());

        let token_request = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", "http://localhost:8080/auth/nextcloud/callback"),
        ];

        let token_url = format!("{}/apps/oauth2/api/v1/token", server_url);
        let response = self.client
            .post(&token_url)
            .form(&token_request)
            .send()
            .await?;

        let token_data: serde_json::Value = response.json().await?;
        
        if let Some(access_token) = token_data["access_token"].as_str() {
            let credentials = CloudCredentials {
                access_token: access_token.to_string(),
                refresh_token: token_data["refresh_token"].as_str().map(|s| s.to_string()),
                expires_at: token_data["expires_in"].as_u64().map(|secs| {
                    Utc::now() + chrono::Duration::seconds(secs as i64)
                }),
                provider: CloudProvider::NextCloud,
            };

            self.credentials.insert(CloudProvider::NextCloud, credentials);
            info!("✅ NextCloud authentication successful for server: {}", server_url);
        } else {
            return Err(anyhow::anyhow!("Failed to get access token from NextCloud"));
        }
        
        Ok(())
    }

    async fn handle_terabox_callback(&mut self, code: &str) -> Result<()> {
        debug!("🔑 Processing TeraBox OAuth callback...");

        let client_id = std::env::var("TERABOX_CLIENT_ID")
            .unwrap_or_else(|_| "your-terabox-client-id".to_string());
        let client_secret = std::env::var("TERABOX_CLIENT_SECRET")
            .unwrap_or_else(|_| "your-terabox-client-secret".to_string());

        let token_request = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", "http://localhost:8080/auth/terabox/callback"),
        ];

        let response = self.client
            .post("https://openapi.baidu.com/oauth/2.0/token")
            .form(&token_request)
            .send()
            .await?;

        let token_data: serde_json::Value = response.json().await?;
        
        if let Some(access_token) = token_data["access_token"].as_str() {
            let credentials = CloudCredentials {
                access_token: access_token.to_string(),
                refresh_token: token_data["refresh_token"].as_str().map(|s| s.to_string()),
                expires_at: token_data["expires_in"].as_u64().map(|secs| {
                    Utc::now() + chrono::Duration::seconds(secs as i64)
                }),
                provider: CloudProvider::TeraBox,
            };

            self.credentials.insert(CloudProvider::TeraBox, credentials);
            info!("✅ TeraBox authentication successful");
        } else {
            return Err(anyhow::anyhow!("Failed to get access token from TeraBox"));
        }
        
        Ok(())
    }

    pub async fn search_files(&self, provider: &CloudProvider, query: &str) -> Result<Vec<CloudFile>> {
        debug!("🔍 Searching files in {:?} for: {}", provider, query);

        if !self.credentials.contains_key(provider) {
            return Err(anyhow::anyhow!("Not authenticated with {:?}", provider));
        }

        match provider {
            CloudProvider::GoogleDrive => self.search_google_drive(query).await,
            CloudProvider::Dropbox => self.search_dropbox(query).await,
            CloudProvider::OneDrive => self.search_onedrive(query).await,
            CloudProvider::NextCloud => self.search_nextcloud(query).await,
            CloudProvider::TeraBox => self.search_terabox(query).await,
        }
    }

    async fn search_google_drive(&self, query: &str) -> Result<Vec<CloudFile>> {
        debug!("📱 Searching Google Drive for: {}", query);

        let credentials = self.credentials.get(&CloudProvider::GoogleDrive)
            .ok_or_else(|| anyhow::anyhow!("Google Drive not authenticated"))?;

        let search_query = format!("name contains '{}'", query);
        let url = format!(
            "https://www.googleapis.com/drive/v3/files?q={}&fields=files(id,name,size,modifiedTime,mimeType,webContentLink,thumbnailLink)",
            urlencoding::encode(&search_query)
        );

        let response = self.client
            .get(&url)
            .bearer_auth(&credentials.access_token)
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        
        let mut files = Vec::new();
        
        if let Some(file_list) = data["files"].as_array() {
            for file in file_list {
                if let (Some(id), Some(name)) = (file["id"].as_str(), file["name"].as_str()) {
                    files.push(CloudFile {
                        id: id.to_string(),
                        name: name.to_string(),
                        path: format!("/google_drive/{}", name),
                        size: file["size"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0),
                        modified: file["modifiedTime"].as_str()
                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|| Utc::now()),
                        mime_type: file["mimeType"].as_str().unwrap_or("application/octet-stream").to_string(),
                        provider: CloudProvider::GoogleDrive,
                        download_url: file["webContentLink"].as_str().map(|s| s.to_string()),
                        thumbnail_url: file["thumbnailLink"].as_str().map(|s| s.to_string()),
                        is_folder: file["mimeType"].as_str() == Some("application/vnd.google-apps.folder"),
                        parent_id: None,
                    });
                }
            }
        }

        debug!("📱 Found {} Google Drive files", files.len());
        Ok(files)
    }

    async fn search_dropbox(&self, query: &str) -> Result<Vec<CloudFile>> {
        debug!("📦 Searching Dropbox for: {}", query);

        let credentials = self.credentials.get(&CloudProvider::Dropbox)
            .ok_or_else(|| anyhow::anyhow!("Dropbox not authenticated"))?;

        let search_request = serde_json::json!({
            "query": query,
            "options": {
                "path": "",
                "max_results": 100,
                "file_status": "active"
            }
        });

        let response = self.client
            .post("https://api.dropboxapi.com/2/files/search_v2")
            .bearer_auth(&credentials.access_token)
            .header("Content-Type", "application/json")
            .json(&search_request)
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        
        let mut files = Vec::new();
        
        if let Some(matches) = data["matches"].as_array() {
            for match_item in matches {
                if let Some(metadata) = match_item["metadata"]["metadata"].as_object() {
                    if let (Some(name), Some(path_lower)) = (
                        metadata["name"].as_str(),
                        metadata["path_lower"].as_str()
                    ) {
                        files.push(CloudFile {
                            id: metadata["id"].as_str().unwrap_or("").to_string(),
                            name: name.to_string(),
                            path: path_lower.to_string(),
                            size: metadata["size"].as_u64().unwrap_or(0),
                            modified: metadata["client_modified"].as_str()
                                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                                .map(|dt| dt.with_timezone(&Utc))
                                .unwrap_or_else(|| Utc::now()),
                            mime_type: "application/octet-stream".to_string(), // Dropbox doesn't provide MIME types
                            provider: CloudProvider::Dropbox,
                            download_url: None, // Would need separate API call
                            thumbnail_url: None,
                            is_folder: metadata[".tag"].as_str() == Some("folder"),
                            parent_id: None,
                        });
                    }
                }
            }
        }

        debug!("📦 Found {} Dropbox files", files.len());
        Ok(files)
    }

    async fn search_onedrive(&self, query: &str) -> Result<Vec<CloudFile>> {
        debug!("🔵 Searching OneDrive for: {}", query);

        let credentials = self.credentials.get(&CloudProvider::OneDrive)
            .ok_or_else(|| anyhow::anyhow!("OneDrive not authenticated"))?;

        let url = format!(
            "https://graph.microsoft.com/v1.0/me/drive/root/search(q='{}')",
            urlencoding::encode(query)
        );

        let response = self.client
            .get(&url)
            .bearer_auth(&credentials.access_token)
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        
        let mut files = Vec::new();
        
        if let Some(value) = data["value"].as_array() {
            for item in value {
                if let (Some(id), Some(name)) = (item["id"].as_str(), item["name"].as_str()) {
                    files.push(CloudFile {
                        id: id.to_string(),
                        name: name.to_string(),
                        path: item["parentReference"]["path"].as_str()
                            .map(|p| format!("{}/{}", p, name))
                            .unwrap_or_else(|| format!("/{}", name)),
                        size: item["size"].as_u64().unwrap_or(0),
                        modified: item["lastModifiedDateTime"].as_str()
                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|| Utc::now()),
                        mime_type: item["file"]["mimeType"].as_str()
                            .unwrap_or("application/octet-stream").to_string(),
                        provider: CloudProvider::OneDrive,
                        download_url: item["@microsoft.graph.downloadUrl"].as_str().map(|s| s.to_string()),
                        thumbnail_url: None,
                        is_folder: item["folder"].is_object(),
                        parent_id: item["parentReference"]["id"].as_str().map(|s| s.to_string()),
                    });
                }
            }
        }

        debug!("🔵 Found {} OneDrive files", files.len());
        Ok(files)
    }

    async fn search_nextcloud(&self, query: &str) -> Result<Vec<CloudFile>> {
        debug!("☁️ Searching NextCloud for: {}", query);
        
        let credentials = self.credentials.get(&CloudProvider::NextCloud)
            .ok_or_else(|| anyhow::anyhow!("NextCloud not authenticated"))?;

        let server_url = std::env::var("NEXTCLOUD_SERVER_URL")
            .unwrap_or_else(|_| "https://your-nextcloud-server.com".to_string());

        // NextCloud uses WebDAV SEARCH method with basic query
        let search_url = format!(
            "{}/remote.php/dav/files/username/?search={}",
            server_url,
            urlencoding::encode(query)
        );

        let response = self.client
            .get(&search_url)
            .bearer_auth(&credentials.access_token)
            .header("Depth", "infinity")
            .header("Content-Type", "application/xml")
            .send()
            .await?;

        let mut files = Vec::new();
        
        // Parse WebDAV XML response (simplified)
        if response.status().is_success() {
            let body = response.text().await?;
            // In a real implementation, we'd parse the XML properly
            // For now, we'll create a placeholder result
            files.push(CloudFile {
                id: "nextcloud_search_result".to_string(),
                name: format!("NextCloud search results for: {}", query),
                path: "/nextcloud/search_results".to_string(),
                size: 0,
                modified: Utc::now(),
                mime_type: "text/plain".to_string(),
                provider: CloudProvider::NextCloud,
                download_url: None,
                thumbnail_url: None,
                is_folder: false,
                parent_id: None,
            });
        }

        debug!("☁️ Found {} NextCloud files", files.len());
        Ok(files)
    }

    async fn search_terabox(&self, query: &str) -> Result<Vec<CloudFile>> {
        debug!("📦 Searching TeraBox for: {}", query);
        
        let credentials = self.credentials.get(&CloudProvider::TeraBox)
            .ok_or_else(|| anyhow::anyhow!("TeraBox not authenticated"))?;

        // TeraBox uses Baidu PCS API for file operations
        let url = format!(
            "https://pan.baidu.com/rest/2.0/xpan/file?method=search&access_token={}&query={}&recursion=1",
            credentials.access_token,
            urlencoding::encode(query)
        );

        let response = self.client
            .get(&url)
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        
        let mut files = Vec::new();
        
        if let Some(file_list) = data["list"].as_array() {
            for file in file_list {
                if let (Some(fs_id), Some(filename)) = (file["fs_id"].as_u64(), file["filename"].as_str()) {
                    files.push(CloudFile {
                        id: fs_id.to_string(),
                        name: filename.to_string(),
                        path: file["path"].as_str().unwrap_or("/").to_string(),
                        size: file["size"].as_u64().unwrap_or(0),
                        modified: file["mtime"].as_u64()
                            .and_then(|timestamp| {
                                DateTime::from_timestamp(timestamp as i64, 0)
                            })
                            .unwrap_or_else(|| Utc::now()),
                        mime_type: "application/octet-stream".to_string(), // TeraBox doesn't provide MIME types directly
                        provider: CloudProvider::TeraBox,
                        download_url: None, // Would need separate API call for download link
                        thumbnail_url: file["thumbs"].as_object().and_then(|thumbs| {
                            thumbs["url1"].as_str().map(|s| s.to_string())
                        }),
                        is_folder: file["isdir"].as_u64().unwrap_or(0) == 1,
                        parent_id: file["parent_path"].as_str().map(|s| s.to_string()),
                    });
                }
            }
        }

        debug!("📦 Found {} TeraBox files", files.len());
        Ok(files)
    }

    pub async fn refresh_token(&mut self, provider: &CloudProvider) -> Result<()> {
        debug!("🔄 Refreshing token for {:?}", provider);

        if let Some(credentials) = self.credentials.get(provider) {
            if let Some(refresh_token) = &credentials.refresh_token {
                match provider {
                    CloudProvider::GoogleDrive => {
                        self.refresh_google_drive_token(refresh_token).await?;
                    }
                    CloudProvider::OneDrive => {
                        self.refresh_onedrive_token(refresh_token).await?;
                    }
                    _ => {
                        warn!("⚠️ Token refresh not implemented for {:?}", provider);
                    }
                }
            }
        }

        Ok(())
    }

    async fn refresh_google_drive_token(&mut self, refresh_token: &str) -> Result<()> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .unwrap_or_else(|_| "your-google-client-id".to_string());
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .unwrap_or_else(|_| "your-google-client-secret".to_string());

        let token_request = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let response = self.client
            .post("https://oauth2.googleapis.com/token")
            .form(&token_request)
            .send()
            .await?;

        let token_data: serde_json::Value = response.json().await?;
        
        if let Some(access_token) = token_data["access_token"].as_str() {
            if let Some(credentials) = self.credentials.get_mut(&CloudProvider::GoogleDrive) {
                credentials.access_token = access_token.to_string();
                credentials.expires_at = token_data["expires_in"].as_u64().map(|secs| {
                    Utc::now() + chrono::Duration::seconds(secs as i64)
                });
            }
            debug!("✅ Google Drive token refreshed");
        }

        Ok(())
    }

    async fn refresh_onedrive_token(&mut self, refresh_token: &str) -> Result<()> {
        let client_id = std::env::var("ONEDRIVE_CLIENT_ID")
            .unwrap_or_else(|_| "your-onedrive-client-id".to_string());
        let client_secret = std::env::var("ONEDRIVE_CLIENT_SECRET")
            .unwrap_or_else(|_| "your-onedrive-client-secret".to_string());

        let token_request = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let response = self.client
            .post("https://login.microsoftonline.com/common/oauth2/v2.0/token")
            .form(&token_request)
            .send()
            .await?;

        let token_data: serde_json::Value = response.json().await?;
        
        if let Some(access_token) = token_data["access_token"].as_str() {
            if let Some(credentials) = self.credentials.get_mut(&CloudProvider::OneDrive) {
                credentials.access_token = access_token.to_string();
                credentials.expires_at = token_data["expires_in"].as_u64().map(|secs| {
                    Utc::now() + chrono::Duration::seconds(secs as i64)
                });
            }
            debug!("✅ OneDrive token refreshed");
        }

        Ok(())
    }

    pub fn is_authenticated(&self, provider: &CloudProvider) -> bool {
        self.credentials.contains_key(provider)
    }

    pub fn get_providers(&self) -> Vec<CloudProvider> {
        self.credentials.keys().cloned().collect()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CloudError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Authentication failed: {0}")]
    Auth(String),
    
    #[error("Provider not supported: {0:?}")]
    UnsupportedProvider(CloudProvider),
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Generic error: {0}")]
    Generic(String),
}
