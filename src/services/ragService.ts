import { invoke } from '@tauri-apps/api/core';
import { createServiceLogger } from '../utils/logger';

export interface RAGDocument {
  id: string;
  content: string;
  metadata: {
    type: 'file' | 'command' | 'documentation' | 'conversation';
    path?: string;
    language?: string;
    timestamp: string;
    project?: string;
    tags: string[];
  };
}

export interface RAGSearchResult {
  document: RAGDocument;
  similarity: number;
  relevance: string;
}

export interface RAGQuery {
  query: string;
  filters?: Record<string, any>;
  maxResults?: number;
  threshold?: number;
}

export interface Collection {
  name: string;
  description?: string;
  document_count: number;
  created_at: string;
}

export interface SearchResponse {
  results: Array<{
    content: string;
    metadata: Record<string, any>;
    score: number;
  }>;
  total_results: number;
  query_time_ms: number;
}

/**
 * RAG Service that uses the Tauri backend LocalRecall implementation
 * This provides semantic search and knowledge management capabilities
 */
class RAGServiceTauri {
  private isInitialized = false;
  private logger = createServiceLogger('RAGServiceTauri');
  private defaultCollection = 'general';

  constructor() {
    // Auto-initialize on first use
    this.ensureInitialized();
  }

  /**
   * Ensure the RAG service is initialized and running
   */
  private async ensureInitialized(): Promise<void> {
    if (this.isInitialized) return;

    try {
      // Ensure LocalRecall is running
      await invoke('local_recall_ensure_running');
      
      // Initialize the service
      await invoke('local_recall_initialize');
      
      // Get or create default collection
      try {
        const collections = await this.listCollections();
        if (!collections.find(c => c.name === this.defaultCollection)) {
          await this.createCollection(this.defaultCollection, 'Default knowledge base');
        }
      } catch (error) {
        this.logger.warn('Failed to setup default collection:', error as Error);
      }

      this.isInitialized = true;
      this.logger.info('RAG Service (Tauri) initialized successfully');
    } catch (error) {
      this.logger.error('Failed to initialize RAG service:', error as Error);
      throw error;
    }
  }

  /**
   * Check health of the LocalRecall service
   */
  async healthCheck(): Promise<boolean> {
    try {
      await invoke('local_recall_health_check');
      return true;
    } catch (error) {
      this.logger.error('Health check failed:', error as Error);
      return false;
    }
  }

  /**
   * Initialize RAG service and ensure it's running
   */
  async initialize(): Promise<void> {
    await this.ensureInitialized();
  }

  /**
   * List all available collections
   */
  async listCollections(): Promise<Collection[]> {
    await this.ensureInitialized();
    try {
      return await invoke<Collection[]>('local_recall_list_collections');
    } catch (error) {
      this.logger.error('Failed to list collections:', error as Error);
      throw error;
    }
  }

  /**
   * Create a new collection
   */
  async createCollection(name: string, description?: string): Promise<void> {
    await this.ensureInitialized();
    try {
      await invoke('local_recall_create_collection', { name, description });
      this.logger.info(`Created collection: ${name}`);
    } catch (error) {
      this.logger.error(`Failed to create collection ${name}:`, error as Error);
      throw error;
    }
  }

  /**
   * Add text content to a collection
   */
  async addText(
    collection: string,
    content: string,
    metadata?: Record<string, any>,
    source?: string
  ): Promise<void> {
    await this.ensureInitialized();
    try {
      await invoke('local_recall_add_text', {
        collection,
        content,
        metadata,
        source
      });
    } catch (error) {
      this.logger.error(`Failed to add text to collection ${collection}:`, error as Error);
      throw error;
    }
  }

  /**
   * Search across collections
   */
  async search(query: RAGQuery): Promise<RAGSearchResult[]> {
    await this.ensureInitialized();
    try {
      const collection = query.filters?.collection || this.defaultCollection;
      const maxResults = query.maxResults || 10;
      const threshold = query.threshold || 0.7;

      const response = await invoke<SearchResponse>('local_recall_search', {
        collection,
        query: query.query,
        maxResults,
        threshold
      });

      return response.results.map((result, index) => ({
        document: {
          id: `result_${index}`,
          content: result.content,
          metadata: {
            type: result.metadata.type || 'unknown',
            path: result.metadata.path,
            language: result.metadata.language,
            timestamp: result.metadata.timestamp || new Date().toISOString(),
            project: result.metadata.project,
            tags: result.metadata.tags ? result.metadata.tags.split(',') : []
          } as RAGDocument['metadata']
        },
        similarity: result.score,
        relevance: this.calculateRelevance(result.score)
      }));
    } catch (error) {
      this.logger.error('Search failed:', error as Error);
      throw error;
    }
  }

  /**
   * Index command execution history
   */
  async indexCommand(
    command: string,
    output: string,
    workingDir: string,
    exitCode: number,
    durationMs: number = 0
  ): Promise<void> {
    await this.ensureInitialized();
    try {
      await invoke('local_recall_index_command', {
        command,
        output,
        workingDir,
        exitCode,
        durationMs
      });
    } catch (error) {
      this.logger.error('Failed to index command:', error as Error);
      throw error;
    }
  }

  /**
   * Index AI conversation
   */
  async indexConversation(
    messages: Array<{ role: string; content: string }>,
    context?: string
  ): Promise<void> {
    await this.ensureInitialized();
    try {
      const messagesPairs = messages.map(m => [m.role, m.content] as [string, string]);
      await invoke('local_recall_index_conversation', {
        messages: messagesPairs,
        context
      });
    } catch (error) {
      this.logger.error('Failed to index conversation:', error as Error);
      throw error;
    }
  }

  /**
   * Index entire codebase
   */
  async indexCodebase(projectPath: string): Promise<void> {
    await this.ensureInitialized();
    try {
      // Let the backend handle file discovery and indexing
      await invoke('local_recall_auto_index_project', {
        workingDir: projectPath
      });
      
      this.logger.info(`Successfully indexed codebase at: ${projectPath}`);
    } catch (error) {
      this.logger.error('Failed to index codebase:', error as Error);
      throw error;
    }
  }

  /**
   * Index specific files
   */
  async indexFiles(projectPath: string, files: string[]): Promise<void> {
    await this.ensureInitialized();
    try {
      await invoke('local_recall_index_codebase', {
        projectPath,
        files
      });
      this.logger.info(`Indexed ${files.length} files from ${projectPath}`);
    } catch (error) {
      this.logger.error('Failed to index files:', error as Error);
      throw error;
    }
  }

  /**
   * Add external source (URL, API, etc.)
   */
  async addExternalSource(
    collection: string,
    url: string,
    updateInterval?: number,
    metadata?: Record<string, any>
  ): Promise<void> {
    await this.ensureInitialized();
    try {
      await invoke('local_recall_add_external_source', {
        collection,
        url,
        updateInterval,
        metadata
      });
    } catch (error) {
      this.logger.error('Failed to add external source:', error as Error);
      throw error;
    }
  }

  /**
   * Upload and index a file
   */
  async uploadFile(
    collection: string,
    filePath: string,
    metadata?: Record<string, any>
  ): Promise<void> {
    await this.ensureInitialized();
    try {
      await invoke('local_recall_upload_file', {
        collection,
        filePath,
        metadata
      });
    } catch (error) {
      this.logger.error('Failed to upload file:', error as Error);
      throw error;
    }
  }

  /**
   * Reset/clear a collection
   */
  async resetCollection(collection: string): Promise<void> {
    await this.ensureInitialized();
    try {
      await invoke('local_recall_reset_collection', { collection });
      this.logger.info(`Reset collection: ${collection}`);
    } catch (error) {
      this.logger.error(`Failed to reset collection ${collection}:`, error as Error);
      throw error;
    }
  }

  /**
   * Get contextual information for AI prompts
   */
  async getContextForPrompt(query: string, maxResults?: number): Promise<string> {
    await this.ensureInitialized();
    try {
      return await invoke<string>('local_recall_get_context_for_prompt', {
        query,
        maxResults: maxResults || 5
      });
    } catch (error) {
      this.logger.error('Failed to get context for prompt:', error as Error);
      return ''; // Don't fail AI requests if RAG is unavailable
    }
  }

  /**
   * Get service statistics
   */
  async getStats(): Promise<Record<string, any>> {
    await this.ensureInitialized();
    try {
      return await invoke<Record<string, any>>('local_recall_get_stats');
    } catch (error) {
      this.logger.error('Failed to get stats:', error as Error);
      throw error;
    }
  }

  /**
   * Get default collection name
   */
  async getDefaultCollection(): Promise<string> {
    try {
      return await invoke<string>('local_recall_get_default_collection');
    } catch (error) {
      this.logger.warn('Failed to get default collection:', error as Error);
      return this.defaultCollection;
    }
  }

  /**
   * Set default collection
   */
  async setDefaultCollection(collection: string): Promise<void> {
    try {
      await invoke('local_recall_set_default_collection', { collection });
      this.defaultCollection = collection;
    } catch (error) {
      this.logger.error('Failed to set default collection:', error as Error);
      throw error;
    }
  }

  /**
   * Auto-index the current project/working directory
   */
  async autoIndexProject(workingDir: string): Promise<void> {
    await this.ensureInitialized();
    try {
      await invoke('local_recall_auto_index_project', { workingDir });
      this.logger.info(`Auto-indexed project: ${workingDir}`);
    } catch (error) {
      this.logger.error('Failed to auto-index project:', error as Error);
      throw error;
    }
  }

  /**
   * Calculate relevance score string from numeric similarity
   */
  private calculateRelevance(similarity: number): string {
    if (similarity >= 0.9) return 'direct_match';
    if (similarity >= 0.8) return 'high';
    if (similarity >= 0.6) return 'medium';
    return 'low';
  }

  /**
   * Optimize and clean up the knowledge base
   */
  async optimize(): Promise<void> {
    try {
      // The backend handles optimization automatically
      await this.getStats();
      this.logger.info('RAG service stats successfully retrieved');
    } catch (error) {
      this.logger.error('Failed to optimize RAG service:', error as Error);
    }
  }
}

// Export singleton instance
export const ragService = new RAGServiceTauri();
export default ragService;
