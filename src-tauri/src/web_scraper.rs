use serde::{Deserialize, Serialize};
use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;
use anyhow::{Result, anyhow, Context};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingOptions {
    pub url: String,
    pub depth: u32,
    pub max_pages: u32,
    pub follow_external_links: bool,
    pub download_images: bool,
    pub download_css: bool,
    pub download_js: bool,
    pub delay_between_requests: u64,
    pub respect_robots_txt: bool,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub output_directory: String,
    pub structure_mirror: bool,
    pub convert_links: bool,
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingResult {
    pub id: String,
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub total_pages: u32,
    pub scraped_pages: u32,
    pub total_size: u64,
    pub errors: Vec<ScrapingError>,
    pub downloaded_files: Vec<DownloadedFile>,
    pub current_url: Option<String>,
    pub estimated_time_remaining: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingError {
    pub url: String,
    pub error: String,
    pub timestamp: DateTime<Utc>,
    pub status_code: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadedFile {
    pub url: String,
    pub local_path: String,
    pub size: u64,
    pub mime_type: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteMap {
    pub url: String,
    pub title: Option<String>,
    pub depth: u32,
    pub children: Vec<SiteMap>,
    pub links: Vec<String>,
    pub images: Vec<String>,
    pub assets: Vec<String>,
    pub last_modified: Option<DateTime<Utc>>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub author: Option<String>,
    pub language: Option<String>,
    pub charset: Option<String>,
    pub favicon: Option<String>,
    pub open_graph: HashMap<String, String>,
    pub twitter_card: HashMap<String, String>,
    pub canonical_url: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotsTxtInfo {
    pub exists: bool,
    pub content: Option<String>,
    pub allowed_paths: Vec<String>,
    pub disallowed_paths: Vec<String>,
    pub crawl_delay: Option<u32>,
    pub sitemap_urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: u64,
    pub total_bytes_downloaded: u64,
    pub pages_per_second: f64,
    pub most_common_errors: Vec<ErrorCount>,
    pub downloads_by_type: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCount {
    pub error: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingEstimate {
    pub estimated_pages: u64,
    pub estimated_size: String,
    pub estimated_time: String,
    pub bandwidth_required: String,
    pub storage_required: String,
    pub warnings: Vec<String>,
}

pub struct WebScraper {
    client: Client,
    active_jobs: HashMap<String, ScrapingResult>,
}

impl WebScraper {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Nexus Terminal Web Scraper 1.0")
            .build()
            .map_err(|e| anyhow!("Failed to build HTTP client: {}", e))?;

        Ok(Self {
            client,
            active_jobs: HashMap::new(),
        })
    }

    /// Start web scraping job
    pub async fn start_scraping(&mut self, options: ScrapingOptions) -> Result<String> {
        let job_id = Uuid::new_v4().to_string();
        
        // Validate options
        self.validate_options(&options)?;
        
        // Check robots.txt if required
        if options.respect_robots_txt {
            let robots_info = self.check_robots_txt(&options.url).await?;
            if !robots_info.allowed_paths.is_empty() {
                // Check if our URL is allowed
                // This is a simplified check - proper implementation would be more comprehensive
            }
        }

        let result = ScrapingResult {
            id: job_id.clone(),
            status: "running".to_string(),
            start_time: Utc::now(),
            end_time: None,
            total_pages: 0,
            scraped_pages: 0,
            total_size: 0,
            errors: Vec::new(),
            downloaded_files: Vec::new(),
            current_url: Some(options.url.clone()),
            estimated_time_remaining: None,
        };

        self.active_jobs.insert(job_id.clone(), result);
        
        // Start scraping in background task
        let scraper = self.clone();
        let job_id_clone = job_id.clone();
        tokio::spawn(async move {
            if let Err(e) = scraper.run_scraping_job(job_id_clone, options).await {
                eprintln!("Scraping job failed: {}", e);
            }
        });

        Ok(job_id)
    }

    /// Get scraping job progress
    pub fn get_scraping_progress(&self, job_id: &str) -> Result<ScrapingResult> {
        self.active_jobs.get(job_id)
            .cloned()
            .ok_or_else(|| anyhow!("Job not found"))
    }

    /// Scrape a single page
    pub async fn scrape_single_page(&self, url: &str, output_path: Option<String>) -> Result<DownloadedFile> {
        let _parsed_url = Url::parse(url)?;
        let response = self.client.get(url).send().await?;
        
        let content = response.text().await?;
        let output_path = output_path.unwrap_or_else(|| {
            let temp_dir = std::env::var("TEMP_DIR")
                .unwrap_or_else(|_| "./temp".to_string());
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            format!("{}/scraped_{}.html", temp_dir, timestamp)
        });
        
        fs::write(&output_path, &content).await?;
        
        Ok(DownloadedFile {
            url: url.to_string(),
            local_path: output_path,
            size: content.len() as u64,
            mime_type: "text/html".to_string(),
            timestamp: Utc::now(),
        })
    }

    /// Extract links from a page
    pub async fn extract_links(&self, url: &str) -> Result<Vec<String>> {
        let response = self.client.get(url).send().await?;
        let content = response.text().await?;
        let document = Html::parse_document(&content);
        
        let link_selector = Selector::parse("a[href]").map_err(|e| anyhow!("Failed to parse selector: {}", e))?;
        let base_url = Url::parse(url)?;
        
        let mut links = Vec::new();
        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                match base_url.join(href) {
                    Ok(absolute_url) => links.push(absolute_url.to_string()),
                    Err(_) => continue, // Skip invalid URLs
                }
            }
        }
        
        Ok(links)
    }

    /// Generate site map
    pub async fn generate_site_map(&self, url: &str, max_depth: u32) -> Result<SiteMap> {
        self.build_site_map(url, 0, max_depth, &mut HashSet::new()).await
    }

    /// Check robots.txt
    pub async fn check_robots_txt(&self, url: &str) -> Result<RobotsTxtInfo> {
        let base_url = Url::parse(url)?;
        let robots_url = base_url.join("/robots.txt")?;
        
        match self.client.get(robots_url.as_str()).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let content = response.text().await?;
                    Ok(self.parse_robots_txt(&content))
                } else {
                    Ok(RobotsTxtInfo {
                        exists: false,
                        content: None,
                        allowed_paths: Vec::new(),
                        disallowed_paths: Vec::new(),
                        crawl_delay: None,
                        sitemap_urls: Vec::new(),
                    })
                }
            }
            Err(_) => {
                Ok(RobotsTxtInfo {
                    exists: false,
                    content: None,
                    allowed_paths: Vec::new(),
                    disallowed_paths: Vec::new(),
                    crawl_delay: None,
                    sitemap_urls: Vec::new(),
                })
            }
        }
    }

    /// Get website metadata
    pub async fn get_website_metadata(&self, url: &str) -> Result<WebsiteMetadata> {
        let response = self.client.get(url).send().await?;
        let content = response.text().await?;
        let document = Html::parse_document(&content);
        
        let mut metadata = WebsiteMetadata {
            title: None,
            description: None,
            keywords: Vec::new(),
            author: None,
            language: None,
            charset: None,
            favicon: None,
            open_graph: HashMap::new(),
            twitter_card: HashMap::new(),
            canonical_url: None,
            last_modified: None,
        };

        // Extract title
        if let Ok(title_selector) = Selector::parse("title") {
            if let Some(title_element) = document.select(&title_selector).next() {
                metadata.title = Some(title_element.text().collect::<String>().trim().to_string());
            }
        }

        // Extract meta tags
        let meta_selector = Selector::parse("meta").map_err(|e| anyhow!("Failed to parse meta selector: {}", e))?;
        for meta in document.select(&meta_selector) {
            let attrs = meta.value();
            
            if let Some(name) = attrs.attr("name") {
                if let Some(content) = attrs.attr("content") {
                    match name.to_lowercase().as_str() {
                        "description" => metadata.description = Some(content.to_string()),
                        "keywords" => metadata.keywords = content.split(',').map(|s| s.trim().to_string()).collect(),
                        "author" => metadata.author = Some(content.to_string()),
                        _ => {}
                    }
                }
            }
            
            // Open Graph tags
            if let Some(property) = attrs.attr("property") {
                if property.starts_with("og:") {
                    if let Some(content) = attrs.attr("content") {
                        metadata.open_graph.insert(property.to_string(), content.to_string());
                    }
                }
            }
            
            // Twitter Card tags
            if let Some(name) = attrs.attr("name") {
                if name.starts_with("twitter:") {
                    if let Some(content) = attrs.attr("content") {
                        metadata.twitter_card.insert(name.to_string(), content.to_string());
                    }
                }
            }
        }

        Ok(metadata)
    }

    /// Estimate scraping job requirements
    pub async fn estimate_scraping(&self, options: &ScrapingOptions) -> Result<ScrapingEstimate> {
        // This is a simplified estimation - real implementation would be more sophisticated
        let estimated_pages = std::cmp::min(options.max_pages as u64, 1000);
        let estimated_size_bytes = estimated_pages * 50_000; // Assume ~50KB per page
        
        let estimated_time_seconds = estimated_pages * (options.delay_between_requests + 1000) / 1000;
        
        Ok(ScrapingEstimate {
            estimated_pages,
            estimated_size: format!("{:.2} MB", estimated_size_bytes as f64 / 1_000_000.0),
            estimated_time: format!("{} minutes", estimated_time_seconds / 60),
            bandwidth_required: format!("{:.2} MB", estimated_size_bytes as f64 / 1_000_000.0),
            storage_required: format!("{:.2} MB", (estimated_size_bytes as f64 * 1.2) / 1_000_000.0),
            warnings: vec![
                "Estimates are approximate and may vary significantly".to_string(),
                "Large sites may take much longer than estimated".to_string(),
            ],
        })
    }

    // Private helper methods

    async fn run_scraping_job(&self, job_id: String, options: ScrapingOptions) -> Result<()> {
        let mut visited_urls = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut total_size = 0u64;
        let mut scraped_pages = 0u32;
        let mut errors = Vec::new();
        let mut downloaded_files = Vec::new();
        
        // Initialize with starting URL
        queue.push_back((options.url.clone(), 0u32));
        
        // Create output directory
        if let Err(e) = fs::create_dir_all(&options.output_directory).await {
            errors.push(ScrapingError {
                url: options.url.clone(),
                error: format!("Failed to create output directory: {}", e),
                timestamp: Utc::now(),
                status_code: None,
            });
            return Ok(());
        }
        
        while let Some((url, depth)) = queue.pop_front() {
            // Check limits
            if scraped_pages >= options.max_pages {
                break;
            }
            
            if depth >= options.depth {
                continue;
            }
            
            if visited_urls.contains(&url) {
                continue;
            }
            
            visited_urls.insert(url.clone());
            
            // Apply delay between requests
            if scraped_pages > 0 && options.delay_between_requests > 0 {
                tokio::time::sleep(Duration::from_millis(options.delay_between_requests)).await;
            }
            
            // Download page
            match self.download_page(&url, &options, depth).await {
                Ok((file, links)) => {
                    total_size += file.size;
                    scraped_pages += 1;
                    downloaded_files.push(file);
                    
                    // Add links to queue for next depth
                    if depth < options.depth - 1 {
                        for link in links {
                            // Check if we should follow this link
                            if self.should_follow_link(&link, &url, &options) {
                                queue.push_back((link, depth + 1));
                            }
                        }
                    }
                }
                Err(e) => {
                    errors.push(ScrapingError {
                        url: url.clone(),
                        error: e.to_string(),
                        timestamp: Utc::now(),
                        status_code: None,
                    });
                }
            }
        }
        
        // Final update (in a real implementation, this would update shared state)
        info!("Scraping job {} completed: {} pages, {} bytes, {} errors", 
              job_id, scraped_pages, total_size, errors.len());
        
        Ok(())
    }
    
    async fn download_page(
        &self, 
        url: &str, 
        options: &ScrapingOptions, 
        depth: u32
    ) -> Result<(DownloadedFile, Vec<String>)> {
        let response = self.client
            .get(url)
            .timeout(Duration::from_secs(options.timeout))
            .send()
            .await
            .context("Failed to fetch page")?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP {} for {}", response.status(), url));
        }
        
        let content = response.text().await
            .context("Failed to read page content")?;
        
        // Generate local file path
        let _parsed_url = Url::parse(url)?;
        let local_path = self.generate_local_path(url, &options.output_directory, depth, options)?;
        
        // Save content to file
        if let Some(parent) = std::path::Path::new(&local_path).parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // Process content based on options
        let processed_content = if options.convert_links {
            self.convert_links_to_local(&content, url, &options.output_directory)
        } else {
            content.clone()
        };
        
        fs::write(&local_path, &processed_content).await?;
        
        // Extract links for crawling
        let links = if depth < options.depth - 1 {
            self.extract_links(url).await.unwrap_or_default()
        } else {
            Vec::new()
        };
        
        // Download assets if requested
        if options.download_images || options.download_css || options.download_js {
            self.download_assets(&content, url, &options.output_directory, options).await?;
        }
        
        let downloaded_file = DownloadedFile {
            url: url.to_string(),
            local_path,
            size: processed_content.len() as u64,
            mime_type: self.detect_mime_type(url),
            timestamp: Utc::now(),
        };
        
        Ok((downloaded_file, links))
    }
    
    fn generate_local_path(&self, url: &str, base_dir: &str, depth: u32, options: &ScrapingOptions) -> Result<String> {
        let parsed_url = Url::parse(url)?;
        let host = parsed_url.host_str().unwrap_or("unknown_host");
        let path = parsed_url.path();
        
        let mut local_path = if options.structure_mirror {
            format!("{}/{}{}", base_dir, host, path)
        } else {
            format!("{}/page_{}_{}.html", base_dir, depth, 
                   url.chars().filter(|c| c.is_alphanumeric()).collect::<String>())
        };
        
        // Ensure it ends with .html if it's a page
        if local_path.ends_with('/') {
            local_path.push_str("index.html");
        } else if !local_path.contains('.') {
            local_path.push_str(".html");
        }
        
        Ok(local_path)
    }
    
    fn should_follow_link(&self, link: &str, base_url: &str, options: &ScrapingOptions) -> bool {
        // Check include/exclude patterns
        if !options.include_patterns.is_empty() {
            let matches_include = options.include_patterns.iter()
                .any(|pattern| link.contains(pattern));
            if !matches_include {
                return false;
            }
        }
        
        if !options.exclude_patterns.is_empty() {
            let matches_exclude = options.exclude_patterns.iter()
                .any(|pattern| link.contains(pattern));
            if matches_exclude {
                return false;
            }
        }
        
        // Check external links
        if !options.follow_external_links {
            if let (Ok(base), Ok(link_url)) = (Url::parse(base_url), Url::parse(link)) {
                if base.host() != link_url.host() {
                    return false;
                }
            }
        }
        
        true
    }
    
    async fn download_assets(
        &self, 
        html_content: &str, 
        base_url: &str, 
        output_dir: &str,
        options: &ScrapingOptions
    ) -> Result<()> {
        let base_url_parsed = Url::parse(base_url)?;
        let mut all_asset_urls = Vec::new();
        
        // Collect all asset URLs without holding document across await points
        {
            let document = Html::parse_document(html_content);
            
            // Collect image URLs
            if options.download_images {
                if let Ok(img_selector) = Selector::parse("img[src]") {
                    let img_urls: Vec<String> = document.select(&img_selector)
                        .filter_map(|img| img.value().attr("src"))
                        .filter_map(|src| base_url_parsed.join(src).ok())
                        .map(|url| url.to_string())
                        .collect();
                    all_asset_urls.extend(img_urls);
                }
            }
            
            // Collect CSS URLs
            if options.download_css {
                if let Ok(css_selector) = Selector::parse("link[rel=stylesheet][href]") {
                    let css_urls: Vec<String> = document.select(&css_selector)
                        .filter_map(|css| css.value().attr("href"))
                        .filter_map(|href| base_url_parsed.join(href).ok())
                        .map(|url| url.to_string())
                        .collect();
                    all_asset_urls.extend(css_urls);
                }
            }
            
            // Collect JavaScript URLs
            if options.download_js {
                if let Ok(js_selector) = Selector::parse("script[src]") {
                    let js_urls: Vec<String> = document.select(&js_selector)
                        .filter_map(|script| script.value().attr("src"))
                        .filter_map(|src| base_url_parsed.join(src).ok())
                        .map(|url| url.to_string())
                        .collect();
                    all_asset_urls.extend(js_urls);
                }
            }
        } // document is dropped here
        
        // Now download all assets without holding any document references
        for asset_url in all_asset_urls {
            let _ = self.download_asset(&asset_url, output_dir).await;
        }
        
        Ok(())
    }
    
    async fn download_asset(&self, url: &str, output_dir: &str) -> Result<()> {
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to download asset: {}", url));
        }
        
        let content = response.bytes().await?;
        let parsed_url = Url::parse(url)?;
        let filename = parsed_url.path_segments()
            .and_then(|segments| segments.last())
            .unwrap_or("asset");
        
        let local_path = format!("{}/assets/{}", output_dir, filename);
        if let Some(parent) = std::path::Path::new(&local_path).parent() {
            fs::create_dir_all(parent).await?;
        }
        
        fs::write(&local_path, &content).await?;
        Ok(())
    }
    
    fn convert_links_to_local(&self, html: &str, base_url: &str, _output_dir: &str) -> String {
        // Simple link conversion - in a real implementation this would be more sophisticated
        let mut converted = html.to_string();
        
        // Convert absolute URLs to relative paths
        if let Ok(base) = Url::parse(base_url) {
            if let Some(host) = base.host_str() {
                let pattern = format!("https://{}", host);
                converted = converted.replace(&pattern, ".");
                
                let pattern = format!("http://{}", host);
                converted = converted.replace(&pattern, ".");
            }
        }
        
        converted
    }
    
    fn detect_mime_type(&self, url: &str) -> String {
        if url.ends_with(".html") || url.ends_with(".htm") {
            "text/html".to_string()
        } else if url.ends_with(".css") {
            "text/css".to_string()
        } else if url.ends_with(".js") {
            "application/javascript".to_string()
        } else if url.ends_with(".json") {
            "application/json".to_string()
        } else if url.ends_with(".png") {
            "image/png".to_string()
        } else if url.ends_with(".jpg") || url.ends_with(".jpeg") {
            "image/jpeg".to_string()
        } else {
            "application/octet-stream".to_string()
        }
    }

    async fn build_site_map(&self, url: &str, current_depth: u32, max_depth: u32, visited: &mut HashSet<String>) -> Result<SiteMap> {
        if current_depth >= max_depth || visited.contains(url) {
            return Ok(SiteMap {
                url: url.to_string(),
                title: None,
                depth: current_depth,
                children: Vec::new(),
                links: Vec::new(),
                images: Vec::new(),
                assets: Vec::new(),
                last_modified: None,
                size: None,
            });
        }
        
        visited.insert(url.to_string());
        
        let response = self.client.get(url).send().await?;
        let content = response.text().await?;
        
        // Extract title from content
        let title = {
            let document = Html::parse_document(&content);
            if let Ok(title_selector) = Selector::parse("title") {
                document.select(&title_selector)
                    .next()
                    .map(|el| el.text().collect::<String>())
            } else {
                None
            }
        };
        
        // Extract links
        let links = self.extract_links(url).await.unwrap_or_default();
        
        Ok(SiteMap {
            url: url.to_string(),
            title,
            depth: current_depth,
            children: Vec::new(), // Would build recursively in full implementation
            links,
            images: Vec::new(),
            assets: Vec::new(),
            last_modified: Some(Utc::now()),
            size: Some(content.len() as u64),
        })
    }

    fn parse_robots_txt(&self, content: &str) -> RobotsTxtInfo {
        let mut allowed_paths = Vec::new();
        let mut disallowed_paths = Vec::new();
        let mut sitemap_urls = Vec::new();
        let mut crawl_delay = None;
        
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("Allow:") {
                allowed_paths.push(line["Allow:".len()..].trim().to_string());
            } else if line.starts_with("Disallow:") {
                disallowed_paths.push(line["Disallow:".len()..].trim().to_string());
            } else if line.starts_with("Sitemap:") {
                sitemap_urls.push(line["Sitemap:".len()..].trim().to_string());
            } else if line.starts_with("Crawl-delay:") {
                if let Ok(delay) = line["Crawl-delay:".len()..].trim().parse() {
                    crawl_delay = Some(delay);
                }
            }
        }
        
        RobotsTxtInfo {
            exists: true,
            content: Some(content.to_string()),
            allowed_paths,
            disallowed_paths,
            crawl_delay,
            sitemap_urls,
        }
    }

    fn validate_options(&self, options: &ScrapingOptions) -> Result<()> {
        Url::parse(&options.url)?;
        
        if options.depth > 10 {
            return Err(anyhow!("Maximum depth is 10"));
        }
        
        if options.max_pages > 10000 {
            return Err(anyhow!("Maximum pages is 10000"));
        }
        
        Ok(())
    }
}

impl Clone for WebScraper {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            active_jobs: HashMap::new(), // Don't clone active jobs
        }
    }
}

/// Global web scraper instance
static WEB_SCRAPER: once_cell::sync::Lazy<std::sync::Mutex<WebScraper>> = 
    once_cell::sync::Lazy::new(|| {
        std::sync::Mutex::new(WebScraper::new().expect("Failed to create WebScraper"))
    });

pub fn get_web_scraper() -> &'static std::sync::Mutex<WebScraper> {
    &WEB_SCRAPER
}
