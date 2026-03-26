import { ChromaClient, Collection, OpenAIEmbeddingFunction } from 'chromadb';
import { readTextFile, readDir } from '@tauri-apps/plugin-fs';
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

class RAGService {
  private client: ChromaClient;
  private collections: Map<string, Collection> = new Map();
  private isInitialized = false;
  private embeddingFunction: OpenAIEmbeddingFunction;
  private logger = createServiceLogger('RAGService');

  constructor() {
    // Use environment variables with fallback to localhost for development
    const chromaHost = this.getEnvVar('CHROMA_HOST', 'localhost');
    const chromaPort = this.getEnvVar('CHROMA_PORT', '8000');
    
    this.client = new ChromaClient({
      path: `http://${chromaHost}:${chromaPort}`
    });
    
    this.embeddingFunction = new OpenAIEmbeddingFunction({
      openai_api_key: "not-needed",
      openai_model: "nomic-embed-text"
    });
  }

  /**
   * Get environment variable with fallback - works in both browser and Tauri
   */
  private getEnvVar(key: string, fallback: string): string {
    // Try process.env first (Node.js/Tauri context)
    if (typeof process !== 'undefined' && process.env && process.env[key]) {
      return process.env[key]!;
    }
    
    // Try window environment (browser context) 
    if (typeof window !== 'undefined' && (window as any).__TAURI__ && (window as any).__TAURI_ENV__) {
      const envValue = (window as any).__TAURI_ENV__[key];
      if (envValue) return envValue;
    }
    
    // Try reading from .env file via Tauri
    try {
      // This would need to be implemented via Tauri command if needed
      return fallback;
    } catch {
      return fallback;
    }
  }

  /**
   * Initialize RAG service and create collections
   */
  async initialize(): Promise<void> {
    try {
      // Check if ChromaDB is running
      await this.client.heartbeat();
      
      // Create collections for different document types
      const collections = [
        'codebase_files',
        'command_history', 
        'documentation',
        'ai_conversations',
        'project_knowledge'
      ];

      for (const collectionName of collections) {
        try {
          let collection = await this.client.getCollection({
            name: collectionName,
            embeddingFunction: this.embeddingFunction
          });
          this.collections.set(collectionName, collection);
        } catch (error) {
          // Collection doesn't exist, create it
          const collection = await this.client.createCollection({
            name: collectionName,
            embeddingFunction: this.embeddingFunction,
            metadata: {
              description: `${collectionName} knowledge base`,
              created: new Date().toISOString()
            }
          });
          this.collections.set(collectionName, collection);
        }
      }

      this.isInitialized = true;
      this.logger.info('RAG Service initialized successfully');
    } catch (error) {
      this.logger.error('Failed to initialize RAG service:', error as Error);
      throw error;
    }
  }

  /**
   * Index the entire codebase for semantic search
   */
  async indexCodebase(projectPath: string): Promise<void> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    const codebaseCollection = this.collections.get('codebase_files')!;
    const documents: RAGDocument[] = [];

    try {
      const files = await this.scanDirectory(projectPath);
      
      for (const file of files) {
        if (this.isTextFile(file.path)) {
          try {
            const content = await readTextFile(file.path);
            const language = this.detectLanguage(file.path);
            
            // Split large files into chunks
            const chunks = this.chunkContent(content, 1000, 200);
            
            for (let i = 0; i < chunks.length; i++) {
              const doc: RAGDocument = {
                id: `${file.path}_chunk_${i}`,
                content: chunks[i],
                metadata: {
                  type: 'file',
                  path: file.path,
                  language,
                  timestamp: new Date().toISOString(),
                  project: projectPath,
                  tags: [language, 'codebase', 'source']
                }
              };
              documents.push(doc);
            }
          } catch (error) {
            this.logger.warn(`Failed to read file ${file.path}:`, error as Error);
          }
        }
      }

      // Batch insert documents
      await this.batchAddDocuments(codebaseCollection, documents);
      this.logger.info(`Indexed ${documents.length} document chunks from codebase`);
      
    } catch (error) {
      this.logger.error('Failed to index codebase:', error as Error);
      throw error;
    }
  }

  /**
   * Add command history to RAG knowledge base
   */
  async indexCommand(command: string, output: string, workingDir: string, exitCode: number): Promise<void> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    const commandCollection = this.collections.get('command_history')!;
    const doc: RAGDocument = {
      id: `cmd_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      content: `Command: ${command}\nOutput: ${output}\nExit Code: ${exitCode}`,
      metadata: {
        type: 'command',
        path: workingDir,
        timestamp: new Date().toISOString(),
        tags: ['command', 'terminal', exitCode === 0 ? 'success' : 'error']
      }
    };

    await this.addDocument(commandCollection, doc);
  }

  /**
   * Add AI conversation to knowledge base
   */
  async indexConversation(messages: Array<{role: string, content: string}>, context: any): Promise<void> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    const conversationCollection = this.collections.get('ai_conversations')!;
    const content = messages.map(m => `${m.role}: ${m.content}`).join('\n');
    
    const doc: RAGDocument = {
      id: `conv_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      content,
      metadata: {
        type: 'conversation',
        timestamp: new Date().toISOString(),
        project: context.workingDirectory,
        tags: ['conversation', 'ai', 'context']
      }
    };

    await this.addDocument(conversationCollection, doc);
  }

  /**
   * Search across all knowledge bases
   */
  async search(query: RAGQuery): Promise<RAGSearchResult[]> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    const results: RAGSearchResult[] = [];
    const maxResults = query.maxResults || 10;
    const threshold = query.threshold || 0.7;

    try {
      // Search across all collections
      for (const [, collection] of this.collections) {
        const searchResults = await collection.query({
          queryTexts: [query.query],
          nResults: Math.ceil(maxResults / this.collections.size),
          where: query.filters
        });

        if (searchResults.documents?.[0] && searchResults.distances?.[0]) {
          for (let i = 0; i < searchResults.documents[0].length; i++) {
            const similarity = 1 - (searchResults.distances[0][i] || 1);
            
            if (similarity >= threshold) {
              const doc: RAGDocument = {
                id: searchResults.ids![0][i] as string,
                content: searchResults.documents[0][i] as string,
                metadata: searchResults.metadatas![0][i] as any
              };

              results.push({
                document: doc,
                similarity,
                relevance: this.calculateRelevance(doc, query.query)
              });
            }
          }
        }
      }

      // Sort by similarity and relevance
      results.sort((a, b) => (b.similarity + this.relevanceScore(b.relevance)) - 
                             (a.similarity + this.relevanceScore(a.relevance)));

      return results.slice(0, maxResults);
      
    } catch (error) {
      this.logger.error('RAG search failed:', error as Error);
      throw error;
    }
  }

  /**
   * Get contextual information for AI prompts
   */
  async getContextForPrompt(query: string, workingDir?: string): Promise<string> {
    const searchQuery: RAGQuery = {
      query,
      maxResults: 5,
      filters: workingDir ? { path: workingDir } : undefined
    };

    const results = await this.search(searchQuery);
    
    if (results.length === 0) {
      return '';
    }

    const contextSections = results.map(result => {
      const { document, similarity } = result;
      const relevanceIndicator = similarity > 0.9 ? '🎯' : similarity > 0.8 ? '📌' : '💡';
      
      return `${relevanceIndicator} **${document.metadata.type.toUpperCase()}** (${Math.round(similarity * 100)}% relevant)
${document.metadata.path ? `Path: ${document.metadata.path}` : ''}
${document.metadata.language ? `Language: ${document.metadata.language}` : ''}
Content: ${document.content.substring(0, 500)}${document.content.length > 500 ? '...' : ''}
---`;
    });

    return `## 🧠 RAG Context Retrieved:
${contextSections.join('\n\n')}

Based on this context, here's my response:`;
  }

  /**
   * Batch add documents to a collection
   */
  private async batchAddDocuments(collection: Collection, documents: RAGDocument[]): Promise<void> {
    const batchSize = 100;
    
    for (let i = 0; i < documents.length; i += batchSize) {
      const batch = documents.slice(i, i + batchSize);
      
      await collection.add({
        ids: batch.map(doc => doc.id),
        documents: batch.map(doc => doc.content),
        metadatas: batch.map(doc => ({
          type: doc.metadata.type,
          path: doc.metadata.path || '',
          language: doc.metadata.language || '',
          timestamp: doc.metadata.timestamp,
          project: doc.metadata.project || '',
          tags: doc.metadata.tags.join(',') // Convert array to string
        }))
      });
    }
  }

  /**
   * Add single document to collection
   */
  private async addDocument(collection: Collection, document: RAGDocument): Promise<void> {
    await collection.add({
      ids: [document.id],
      documents: [document.content],
      metadatas: [{
        type: document.metadata.type,
        path: document.metadata.path || '',
        language: document.metadata.language || '',
        timestamp: document.metadata.timestamp,
        project: document.metadata.project || '',
        tags: document.metadata.tags.join(',') // Convert array to string
      }]
    });
  }

  /**
   * Recursively scan directory for files
   */
  private async scanDirectory(path: string): Promise<Array<{path: string, isFile: boolean}>> {
    const files: Array<{path: string, isFile: boolean}> = [];
    const ignoreDirs = ['node_modules', '.git', 'target', 'build', 'dist', '.next'];
    
    try {
      const entries = await readDir(path);
      
      for (const entry of entries) {
        if (entry.isDirectory) {
          // It's a directory
          if (!ignoreDirs.some(ignore => entry.name.includes(ignore))) {
            files.push({ path: entry.name, isFile: false });
            // Recursively scan subdirectory
            const subFiles = await this.scanDirectory(entry.name);
            files.push(...subFiles);
          }
        } else {
          // It's a file
          files.push({ path: entry.name, isFile: true });
        }
      }
    } catch (error) {
      this.logger.warn(`Failed to scan directory ${path}:`, error as Error);
    }
    
    return files;
  }

  /**
   * Check if file is a text file that should be indexed
   */
  private isTextFile(path: string): boolean {
    const textExtensions = [
      '.ts', '.js', '.jsx', '.tsx', '.py', '.rs', '.go', '.java', '.c', '.cpp', '.h',
      '.css', '.scss', '.html', '.xml', '.json', '.yaml', '.yml', '.toml', '.ini',
      '.md', '.txt', '.sql', '.sh', '.bash', '.zsh', '.fish', '.ps1', '.bat',
      '.dockerfile', '.gitignore', '.env', '.config'
    ];

    const extension = path.substring(path.lastIndexOf('.')).toLowerCase();
    return textExtensions.includes(extension) || 
           path.includes('README') || 
           path.includes('LICENSE') ||
           path.includes('Dockerfile');
  }

  /**
   * Detect programming language from file extension
   */
  private detectLanguage(path: string): string {
    const extension = path.substring(path.lastIndexOf('.')).toLowerCase();
    const languageMap: Record<string, string> = {
      '.ts': 'typescript',
      '.js': 'javascript',
      '.jsx': 'javascript',
      '.tsx': 'typescript',
      '.py': 'python',
      '.rs': 'rust',
      '.go': 'go',
      '.java': 'java',
      '.c': 'c',
      '.cpp': 'cpp',
      '.h': 'c',
      '.css': 'css',
      '.scss': 'scss',
      '.html': 'html',
      '.xml': 'xml',
      '.json': 'json',
      '.yaml': 'yaml',
      '.yml': 'yaml',
      '.toml': 'toml',
      '.md': 'markdown',
      '.sql': 'sql',
      '.sh': 'bash',
      '.bash': 'bash',
      '.zsh': 'zsh'
    };

    return languageMap[extension] || 'text';
  }

  /**
   * Split content into overlapping chunks
   */
  private chunkContent(content: string, chunkSize: number, overlap: number): string[] {
    const chunks: string[] = [];
    const lines = content.split('\n');
    
    let currentChunk = '';
    let currentSize = 0;
    
    for (const line of lines) {
      if (currentSize + line.length > chunkSize && currentChunk) {
        chunks.push(currentChunk.trim());
        
        // Create overlap by keeping last few lines
        const overlapLines = currentChunk.split('\n').slice(-Math.floor(overlap / 50));
        currentChunk = overlapLines.join('\n') + '\n' + line;
        currentSize = currentChunk.length;
      } else {
        currentChunk += line + '\n';
        currentSize += line.length + 1;
      }
    }
    
    if (currentChunk.trim()) {
      chunks.push(currentChunk.trim());
    }
    
    return chunks;
  }

  /**
   * Calculate relevance based on document type and content
   */
  private calculateRelevance(document: RAGDocument, query: string): string {
    const queryLower = query.toLowerCase();
    const contentLower = document.content.toLowerCase();
    
    if (document.metadata.type === 'command' && queryLower.includes('command')) {
      return 'high';
    }
    
    if (document.metadata.type === 'file' && document.metadata.language) {
      if (queryLower.includes(document.metadata.language)) {
        return 'high';
      }
    }
    
    if (contentLower.includes(queryLower)) {
      return 'direct_match';
    }
    
    return 'medium';
  }

  /**
   * Convert relevance string to numeric score
   */
  private relevanceScore(relevance: string): number {
    const scores = {
      'direct_match': 0.3,
      'high': 0.2,
      'medium': 0.1,
      'low': 0.05
    };
    
    return scores[relevance as keyof typeof scores] || 0;
  }

  /**
   * Clean up and optimize the knowledge base
   */
  async optimize(): Promise<void> {
    if (!this.isInitialized) return;

    try {
      // Remove old documents (older than 30 days)
      const thirtyDaysAgo = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString();
      
      for (const [name, collection] of this.collections) {
        if (name === 'command_history' || name === 'ai_conversations') {
          // These collections can be cleaned up more aggressively
          const results = await collection.get({
            where: { timestamp: { "$lt": thirtyDaysAgo } }
          });
          
          if (results.ids && results.ids.length > 0) {
            await collection.delete({
              ids: results.ids as string[]
            });
            this.logger.info(`Cleaned up ${results.ids.length} old documents from ${name}`);
          }
        }
      }
    } catch (error) {
      this.logger.error('Failed to optimize RAG database:', error as Error);
    }
  }
}

// Singleton instance
export const ragService = new RAGService();
