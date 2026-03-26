import { invoke } from '@tauri-apps/api/core';
// import { writeText } from '@tauri-apps/api/clipboard';

export interface ScrapingOptions {
  url: string;
  depth: number;
  maxPages: number;
  followExternalLinks: boolean;
  downloadImages: boolean;
  downloadCSS: boolean;
  downloadJS: boolean;
  customUserAgent?: string;
  delayBetweenRequests: number;
  respectRobotsTxt: boolean;
  includePatterns: string[];
  excludePatterns: string[];
  outputDirectory: string;
  structureMirror: boolean;
  convertLinks: boolean;
  timeout: number;
}

export interface ScrapingResult {
  id: string;
  status: 'running' | 'completed' | 'failed' | 'paused';
  startTime: Date;
  endTime?: Date;
  totalPages: number;
  scrapedPages: number;
  totalSize: number;
  errors: ScrapingError[];
  downloadedFiles: DownloadedFile[];
  currentUrl?: string;
  estimatedTimeRemaining?: number;
}

export interface ScrapingError {
  url: string;
  error: string;
  timestamp: Date;
  statusCode?: number;
}

export interface DownloadedFile {
  url: string;
  localPath: string;
  size: number;
  mimeType: string;
  timestamp: Date;
}

export interface SiteMap {
  url: string;
  title?: string;
  depth: number;
  children: SiteMap[];
  links: string[];
  images: string[];
  assets: string[];
  lastModified?: Date;
  size?: number;
}

export class WebScrapingService {
  private activeJobs = new Map<string, ScrapingResult>();
  
  async startScraping(options: ScrapingOptions): Promise<string> {
    const jobId = this.generateJobId();
    
    // Validate options
    this.validateOptions(options);
    
    // Initialize job
    const result: ScrapingResult = {
      id: jobId,
      status: 'running',
      startTime: new Date(),
      totalPages: 0,
      scrapedPages: 0,
      totalSize: 0,
      errors: [],
      downloadedFiles: []
    };
    
    this.activeJobs.set(jobId, result);
    
    try {
      // Start scraping via backend
      await invoke('start_web_scraping', { jobId, options });
      
      // Start progress monitoring
      this.monitorProgress(jobId);
      
      return jobId;
    } catch (error) {
      result.status = 'failed';
      result.errors.push({
        url: options.url,
        error: error as string,
        timestamp: new Date()
      });
      throw error;
    }
  }
  
  async pauseScraping(jobId: string): Promise<void> {
    const job = this.activeJobs.get(jobId);
    if (!job) throw new Error('Job not found');
    
    await invoke('pause_scraping', { jobId });
    job.status = 'paused';
  }
  
  async resumeScraping(jobId: string): Promise<void> {
    const job = this.activeJobs.get(jobId);
    if (!job) throw new Error('Job not found');
    
    await invoke('resume_scraping', { jobId });
    job.status = 'running';
    this.monitorProgress(jobId);
  }
  
  async stopScraping(jobId: string): Promise<void> {
    const job = this.activeJobs.get(jobId);
    if (!job) throw new Error('Job not found');
    
    await invoke('stop_scraping', { jobId });
    job.status = 'failed';
    job.endTime = new Date();
  }
  
  getJobStatus(jobId: string): ScrapingResult | null {
    return this.activeJobs.get(jobId) || null;
  }
  
  getAllJobs(): ScrapingResult[] {
    return Array.from(this.activeJobs.values());
  }
  
  private async monitorProgress(jobId: string): Promise<void> {
    const job = this.activeJobs.get(jobId);
    if (!job) return;
    
    const interval = setInterval(async () => {
      try {
        const progress = await invoke<ScrapingResult>('get_scraping_progress', { jobId });
        
        // Update job with progress
        Object.assign(job, progress);
        
        if (progress.status === 'completed' || progress.status === 'failed') {
          clearInterval(interval);
          job.endTime = new Date();
        }
      } catch (error) {
        clearInterval(interval);
        job.status = 'failed';
        job.endTime = new Date();
        job.errors.push({
          url: job.currentUrl || 'unknown',
          error: error as string,
          timestamp: new Date()
        });
      }
    }, 1000);
  }
  
  private validateOptions(options: ScrapingOptions): void {
    if (!options.url) {
      throw new Error('URL is required');
    }
    
    if (!this.isValidUrl(options.url)) {
      throw new Error('Invalid URL format');
    }
    
    if (options.depth < 0 || options.depth > 10) {
      throw new Error('Depth must be between 0 and 10');
    }
    
    if (options.maxPages < 1 || options.maxPages > 10000) {
      throw new Error('Max pages must be between 1 and 10000');
    }
    
    if (options.delayBetweenRequests < 0) {
      throw new Error('Delay must be non-negative');
    }
    
    if (!options.outputDirectory) {
      throw new Error('Output directory is required');
    }
  }
  
  private isValidUrl(url: string): boolean {
    try {
      new URL(url);
      return true;
    } catch {
      return false;
    }
  }
  
  private generateJobId(): string {
    return `scrape_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
  
  /**
   * Quick scrape single page
   */
  async scrapePage(url: string, outputPath?: string): Promise<DownloadedFile> {
    try {
      const result = await invoke<DownloadedFile>('scrape_single_page', {
        url,
        outputPath: outputPath || `./temp/scraped_${Date.now()}.html`
      });
      
      return result;
    } catch (error) {
      throw new Error(`Failed to scrape page: ${error}`);
    }
  }
  
  /**
   * Extract links from a page
   */
  async extractLinks(url: string): Promise<string[]> {
    try {
      const links = await invoke<string[]>('extract_links', { url });
      return links;
    } catch (error) {
      throw new Error(`Failed to extract links: ${error}`);
    }
  }
  
  /**
   * Generate site map
   */
  async generateSiteMap(url: string, maxDepth: number = 2): Promise<SiteMap> {
    try {
      const siteMap = await invoke<SiteMap>('generate_site_map', { url, maxDepth });
      return siteMap;
    } catch (error) {
      throw new Error(`Failed to generate site map: ${error}`);
    }
  }
  
  /**
   * Download specific file types
   */
  async downloadAssets(url: string, assetTypes: string[], outputDir: string): Promise<DownloadedFile[]> {
    try {
      const files = await invoke<DownloadedFile[]>('download_assets', {
        url,
        assetTypes,
        outputDir
      });
      
      return files;
    } catch (error) {
      throw new Error(`Failed to download assets: ${error}`);
    }
  }
  
  /**
   * Convert scraped site for offline viewing
   */
  async convertForOfflineViewing(inputDir: string, outputDir: string): Promise<void> {
    try {
      await invoke('convert_offline', { inputDir, outputDir });
    } catch (error) {
      throw new Error(`Failed to convert for offline viewing: ${error}`);
    }
  }
  
  /**
   * Get website metadata
   */
  async getWebsiteMetadata(url: string): Promise<WebsiteMetadata> {
    try {
      const metadata = await invoke<WebsiteMetadata>('get_website_metadata', { url });
      return metadata;
    } catch (error) {
      throw new Error(`Failed to get metadata: ${error}`);
    }
  }
  
  /**
   * Check robots.txt
   */
  async checkRobotsTxt(url: string): Promise<RobotsTxtInfo> {
    try {
      const robotsInfo = await invoke<RobotsTxtInfo>('check_robots_txt', { url });
      return robotsInfo;
    } catch (error) {
      throw new Error(`Failed to check robots.txt: ${error}`);
    }
  }
  
  /**
   * Get scraping statistics
   */
  async getScrapingStats(jobId: string): Promise<ScrapingStats> {
    try {
      const stats = await invoke<ScrapingStats>('get_scraping_stats', { jobId });
      return stats;
    } catch (error) {
      throw new Error(`Failed to get stats: ${error}`);
    }
  }
  
  /**
   * Export scraped data
   */
  async exportData(jobId: string, format: 'json' | 'csv' | 'xml', outputPath: string): Promise<void> {
    try {
      await invoke('export_scraped_data', { jobId, format, outputPath });
    } catch (error) {
      throw new Error(`Failed to export data: ${error}`);
    }
  }
  
  /**
   * Import scraping configuration
   */
  async importConfig(configPath: string): Promise<ScrapingOptions> {
    try {
      const config = await invoke<ScrapingOptions>('import_scraping_config', { configPath });
      return config;
    } catch (error) {
      throw new Error(`Failed to import config: ${error}`);
    }
  }
  
  /**
   * Save scraping configuration
   */
  async saveConfig(options: ScrapingOptions, configPath: string): Promise<void> {
    try {
      await invoke('save_scraping_config', { options, configPath });
    } catch (error) {
      throw new Error(`Failed to save config: ${error}`);
    }
  }
  
  /**
   * Get preset configurations
   */
  getPresetConfigs(): Record<string, Partial<ScrapingOptions>> {
    return {
      'quick-mirror': {
        depth: 2,
        maxPages: 100,
        followExternalLinks: false,
        downloadImages: true,
        downloadCSS: true,
        downloadJS: false,
        delayBetweenRequests: 1000,
        respectRobotsTxt: true,
        structureMirror: true,
        convertLinks: true,
        timeout: 30000
      },
      
      'deep-archive': {
        depth: 5,
        maxPages: 1000,
        followExternalLinks: false,
        downloadImages: true,
        downloadCSS: true,
        downloadJS: true,
        delayBetweenRequests: 2000,
        respectRobotsTxt: true,
        structureMirror: true,
        convertLinks: true,
        timeout: 60000
      },
      
      'images-only': {
        depth: 3,
        maxPages: 500,
        followExternalLinks: false,
        downloadImages: true,
        downloadCSS: false,
        downloadJS: false,
        delayBetweenRequests: 500,
        respectRobotsTxt: true,
        includePatterns: ['*.jpg', '*.png', '*.gif', '*.webp', '*.svg'],
        timeout: 30000
      },
      
      'documentation': {
        depth: 10,
        maxPages: 2000,
        followExternalLinks: false,
        downloadImages: true,
        downloadCSS: true,
        downloadJS: false,
        delayBetweenRequests: 1500,
        respectRobotsTxt: true,
        includePatterns: ['*/docs/*', '*/documentation/*', '*/api/*'],
        structureMirror: true,
        convertLinks: true,
        timeout: 45000
      },
      
      'social-media': {
        depth: 1,
        maxPages: 200,
        followExternalLinks: false,
        downloadImages: true,
        downloadCSS: false,
        downloadJS: false,
        delayBetweenRequests: 3000,
        respectRobotsTxt: true,
        timeout: 30000
      }
    };
  }
  
  /**
   * Estimate scraping time and resources
   */
  async estimateScrapingJob(options: ScrapingOptions): Promise<ScrapingEstimate> {
    try {
      const estimate = await invoke<ScrapingEstimate>('estimate_scraping', { options });
      return estimate;
    } catch (error) {
      throw new Error(`Failed to estimate scraping job: ${error}`);
    }
  }
  
  /**
   * Clean up completed jobs
   */
  cleanupJobs(olderThanHours: number = 24): void {
    const cutoff = new Date(Date.now() - olderThanHours * 60 * 60 * 1000);
    
    for (const [jobId, job] of this.activeJobs.entries()) {
      if (job.endTime && job.endTime < cutoff) {
        this.activeJobs.delete(jobId);
      }
    }
  }
  
  /**
   * Get scraping templates for common sites
   */
  getSiteTemplates(): Record<string, Partial<ScrapingOptions>> {
    return {
      'github-repo': {
        includePatterns: ['*/blob/*', '*/tree/*', '*/releases/*'],
        excludePatterns: ['*/commits/*', '*/pull/*', '*/issues/*'],
        depth: 3,
        delayBetweenRequests: 1000
      },
      
      'wikipedia': {
        includePatterns: ['*/wiki/*'],
        excludePatterns: ['*/Special:*', '*/User:*', '*/Talk:*'],
        depth: 2,
        delayBetweenRequests: 2000,
        respectRobotsTxt: true
      },
      
      'stackoverflow': {
        includePatterns: ['*/questions/*', '*/tags/*'],
        excludePatterns: ['*/users/*', '*/jobs/*'],
        depth: 2,
        delayBetweenRequests: 1500
      },
      
      'news-site': {
        includePatterns: ['*/article/*', '*/news/*', '*/story/*'],
        excludePatterns: ['*/comment*', '*/user*'],
        depth: 2,
        delayBetweenRequests: 2000,
        downloadImages: true
      }
    };
  }
}

export interface WebsiteMetadata {
  title?: string;
  description?: string;
  keywords?: string[];
  author?: string;
  language?: string;
  charset?: string;
  favicon?: string;
  openGraph?: Record<string, string>;
  twitterCard?: Record<string, string>;
  canonicalUrl?: string;
  lastModified?: Date;
}

export interface RobotsTxtInfo {
  exists: boolean;
  content?: string;
  allowedPaths: string[];
  disallowedPaths: string[];
  crawlDelay?: number;
  sitemapUrls: string[];
}

export interface ScrapingStats {
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  averageResponseTime: number;
  totalBytesDownloaded: number;
  pagesPerSecond: number;
  mostCommonErrors: Array<{ error: string; count: number }>;
  downloadsByType: Record<string, number>;
}

export interface ScrapingEstimate {
  estimatedPages: number;
  estimatedSize: string;
  estimatedTime: string;
  bandwidthRequired: string;
  storageRequired: string;
  warnings: string[];
}

export const webScrapingService = new WebScrapingService();
