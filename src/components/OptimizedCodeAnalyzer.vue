<template>
  <div class="optimized-code-analyzer">
    <!-- Enhanced Header with Real-time Status -->
    <div class="analyzer-header">
      <div class="header-left">
        <h3>🚀 AI Code Analysis</h3>
        <div class="performance-metrics" v-if="lastAnalysis">
          <span class="metric">⚡ {{ lastAnalysis.processing_time_ms }}ms</span>
          <span class="metric" v-if="lastAnalysis.tokens_used">📊 {{ lastAnalysis.tokens_used }} tokens</span>
          <span class="metric" v-if="lastAnalysis.confidence_score">
            🎯 {{ Math.round(lastAnalysis.confidence_score * 100) }}% confidence
          </span>
        </div>
      </div>
      <div class="header-controls">
        <select v-model="selectedLanguage" class="form-input language-select" @change="onLanguageChange">
          <option value="auto">🔍 Auto Detect</option>
          <option value="python">🐍 Python</option>
          <option value="javascript">📜 JavaScript</option>
          <option value="typescript">📘 TypeScript</option>
          <option value="rust">🦀 Rust</option>
          <option value="go">🐹 Go</option>
          <option value="java">☕ Java</option>
          <option value="c++">⚙️ C++</option>
          <option value="c">🔧 C</option>
          <option value="csharp">🎯 C#</option>
          <option value="php">🐘 PHP</option>
          <option value="ruby">💎 Ruby</option>
          <option value="swift">🐦 Swift</option>
          <option value="kotlin">🤖 Kotlin</option>
        </select>
        <button class="btn btn-secondary" @click="toggleAdvancedMode">
          {{ advancedMode ? '📊' : '⚙️' }} {{ advancedMode ? 'Simple' : 'Advanced' }}
        </button>
      </div>
    </div>

    <!-- Advanced Configuration Panel -->
    <div v-if="advancedMode" class="advanced-panel">
      <div class="advanced-controls">
        <div class="control-group">
          <label class="form-label">Model Selection</label>
          <select v-model="selectedModel" class="form-input">
            <option value="auto">🤖 Auto Select</option>
            <option v-for="model in availableModels" :key="model.name" :value="model.name">
              {{ model.name }} {{ getModelRecommendation(model) }}
            </option>
          </select>
        </div>
        <div class="control-group">
          <label class="form-label">Context</label>
          <input v-model="analysisContext" class="form-input" placeholder="Additional context (e.g., 'Web API endpoint')">
        </div>
        <div class="control-group">
          <label class="form-label">File Path</label>
          <input v-model="filePath" class="form-input" placeholder="File path (optional)">
        </div>
      </div>
      <div class="cache-controls">
        <button class="btn btn-secondary" @click="clearCache" :disabled="isClearing">
          {{ isClearing ? '🔄' : '🗑️' }} Clear Cache
        </button>
        <label class="checkbox-label">
          <input type="checkbox" v-model="enableCaching">
          📦 Enable Caching
        </label>
      </div>
    </div>

    <div class="analyzer-body">
      <!-- Enhanced Code Input Section -->
      <div class="code-input-section">
        <div class="section-header">
          <h4>Code Input</h4>
          <div class="input-controls">
            <div class="code-stats">
              <span class="stat">{{ codeInput.length }} chars</span>
              <span class="stat">{{ codeInput.split('\n').length }} lines</span>
              <span class="stat" :class="{ warning: codeInput.length > 40000 }">
                {{ getCodeSizeStatus() }}
              </span>
            </div>
          </div>
        </div>
        
        <!-- Enhanced Operation Buttons -->
        <div class="operation-section">
          <div class="operation-buttons">
            <button 
              v-for="operation in enhancedOperations" 
              :key="operation.id"
              :class="['btn', 'operation-btn', { active: selectedOperation === operation.id }]"
              @click="selectedOperation = operation.id"
              :title="operation.description"
            >
              {{ operation.icon }} {{ operation.name }}
            </button>
          </div>
        </div>

        <!-- Code Input Area with Enhancements -->
        <div class="code-input-container">
          <textarea 
            v-model="codeInput" 
            class="code-input"
            :placeholder="getPlaceholderText()"
            rows="18"
            @input="onCodeInput"
            @paste="onCodePaste"
          ></textarea>
          <div class="input-overlay">
            <div class="detected-language" v-if="detectedLanguage && selectedLanguage === 'auto'">
              Detected: {{ detectedLanguage }}
            </div>
          </div>
        </div>
      </div>

      <!-- Enhanced Analysis Results Section -->
      <div class="analysis-section">
        <div class="section-header">
          <h4>AI Analysis Results</h4>
          <div class="analysis-controls">
            <button 
              class="btn btn-primary analyze-btn" 
              @click="analyzeCode" 
              :disabled="!canAnalyze"
              :class="{ loading: isAnalyzing }"
            >
              <span v-if="isAnalyzing" class="loading-spinner"></span>
              <span v-else-if="isRetrying">🔄</span>
              <span v-else>🚀</span>
              {{ getAnalyzeButtonText() }}
            </button>
            <button 
              v-if="analysisResult" 
              class="btn btn-secondary" 
              @click="exportResults"
            >
              📄 Export
            </button>
          </div>
        </div>
        
        <!-- Analysis Results Display -->
        <div v-if="analysisResult" class="analysis-result">
          <div class="result-header">
            <div class="result-meta">
              <span class="model-info">🤖 {{ analysisResult.model_used }}</span>
              <span class="operation-info">⚙️ {{ getOperationName(selectedOperation) }}</span>
              <span class="timing-info">⏱️ {{ analysisResult.processing_time_ms }}ms</span>
              <span v-if="analysisResult.confidence_score" class="confidence-info">
                🎯 {{ Math.round(analysisResult.confidence_score * 100) }}%
              </span>
            </div>
            <div class="result-actions">
              <button class="btn-icon" @click="copyToClipboard(analysisResult.result)" title="Copy to clipboard">
                📋
              </button>
              <button class="btn-icon" @click="toggleResultFormat" title="Toggle format">
                {{ resultFormatMode === 'formatted' ? '📝' : '🔧' }}
              </button>
            </div>
          </div>
          
          <div class="result-content-wrapper">
            <pre 
              v-if="resultFormatMode === 'raw'" 
              class="result-content raw"
            >{{ analysisResult.result }}</pre>
            <div 
              v-else 
              class="result-content formatted"
              v-html="formatAnalysisResult(analysisResult.result)"
            ></div>
          </div>
        </div>
        
        <!-- Enhanced Empty State -->
        <div v-else-if="!isAnalyzing" class="no-result">
          <div class="empty-state">
            <div class="placeholder-animation">
              <div class="placeholder-icon">🤖</div>
              <div class="placeholder-waves">
                <div class="wave"></div>
                <div class="wave"></div>
                <div class="wave"></div>
              </div>
            </div>
            <h4>Ready for AI Analysis</h4>
            <p>{{ getEmptyStateMessage() }}</p>
            <div class="quick-actions">
              <button class="btn btn-outline" @click="loadSampleCode">
                📝 Load Sample Code
              </button>
              <button class="btn btn-outline" @click="pasteFromClipboard">
                📋 Paste from Clipboard
              </button>
            </div>
          </div>
        </div>
        
        <!-- Enhanced Loading State -->
        <div v-if="isAnalyzing" class="analyzing-state">
          <div class="analysis-animation">
            <div class="brain-icon">🧠</div>
            <div class="thinking-dots">
              <div class="dot"></div>
              <div class="dot"></div>
              <div class="dot"></div>
            </div>
          </div>
          <h4>AI is analyzing your code...</h4>
          <p>{{ getAnalysisProgressMessage() }}</p>
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: analysisProgress + '%' }"></div>
          </div>
          <button class="btn btn-secondary" @click="cancelAnalysis" v-if="canCancelAnalysis">
            ❌ Cancel Analysis
          </button>
        </div>
        
        <!-- Enhanced Error Display -->
        <div v-if="error" class="error-message enhanced">
          <div class="error-header">
            <span class="error-icon">⚠️</span>
            <strong>{{ error.code || 'Analysis Error' }}</strong>
            <button class="btn-icon" @click="clearError">✖️</button>
          </div>
          <div class="error-content">
            <p>{{ error.message }}</p>
            <div v-if="error.details" class="error-details">
              <details>
                <summary>Show details</summary>
                <pre>{{ error.details }}</pre>
              </details>
            </div>
            <div class="error-actions" v-if="error.retry_after">
              <button class="btn btn-primary" @click="retryAnalysis" :disabled="retryCountdown > 0">
                {{ retryCountdown > 0 ? `Retry in ${retryCountdown}s` : '🔄 Retry' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

export default {
  name: 'OptimizedCodeAnalyzer',
  setup() {
    // Reactive state
    const codeInput = ref('')
    const selectedLanguage = ref('auto')
    const selectedOperation = ref('analyze')
    const selectedModel = ref('auto')
    const isAnalyzing = ref(false)
    const analysisResult = ref(null)
    const lastAnalysis = ref(null)
    const error = ref(null)
    const advancedMode = ref(false)
    const analysisContext = ref('')
    const filePath = ref('')
    const enableCaching = ref(true)
    const availableModels = ref([])
    const detectedLanguage = ref('')
    const resultFormatMode = ref('formatted')
    const isClearing = ref(false)
    const isRetrying = ref(false)
    const retryCountdown = ref(0)
    const analysisProgress = ref(0)
    const canCancelAnalysis = ref(false)

    // Enhanced operations with descriptions
    const enhancedOperations = [
      { 
        id: 'analyze', 
        name: 'Analyze', 
        icon: '🔍', 
        description: 'Comprehensive code analysis for issues and improvements' 
      },
      { 
        id: 'fix_bugs', 
        name: 'Fix Bugs', 
        icon: '🐛', 
        description: 'Identify and provide fixes for bugs in the code' 
      },
      { 
        id: 'optimize', 
        name: 'Optimize', 
        icon: '⚡', 
        description: 'Optimize code for better performance and efficiency' 
      },
      { 
        id: 'document', 
        name: 'Document', 
        icon: '📚', 
        description: 'Generate comprehensive documentation' 
      },
      { 
        id: 'test', 
        name: 'Test', 
        icon: '🧪', 
        description: 'Generate unit tests for the code' 
      },
      { 
        id: 'security', 
        name: 'Security', 
        icon: '🔒', 
        description: 'Analyze code for security vulnerabilities' 
      },
      { 
        id: 'refactor', 
        name: 'Refactor', 
        icon: '🔧', 
        description: 'Refactor code for better maintainability' 
      }
    ]

    // Sample code snippets
    const sampleCode = {
      python: `def fibonacci(n):
    if n <= 1:
        return n
    else:
        return fibonacci(n-1) + fibonacci(n-2)

# Calculate the 10th Fibonacci number
result = fibonacci(10)
print(f"The 10th Fibonacci number is: {result}")`,
      javascript: `function quickSort(arr) {
    if (arr.length <= 1) {
        return arr;
    }
    
    const pivot = arr[Math.floor(arr.length / 2)];
    const left = arr.filter(x => x < pivot);
    const middle = arr.filter(x => x === pivot);
    const right = arr.filter(x => x > pivot);
    
    return [...quickSort(left), ...middle, ...quickSort(right)];
}

const numbers = [3, 6, 8, 10, 1, 2, 1];
console.log(quickSort(numbers));`,
      rust: `use std::collections::HashMap;

fn word_count(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    
    for word in text.split_whitespace() {
        let word = word.to_lowercase();
        *counts.entry(word).or_insert(0) += 1;
    }
    
    counts
}

fn main() {
    let text = "hello world hello rust world";
    let counts = word_count(text);
    println!("{:?}", counts);
}`
    }

    // Computed properties
    const canAnalyze = computed(() => {
      return codeInput.value.trim().length > 0 && !isAnalyzing.value
    })

    const getOperationName = (operationId) => {
      const operation = enhancedOperations.find(op => op.id === operationId)
      return operation ? operation.name : operationId
    }

    const getModelRecommendation = (model) => {
      if (model.recommended_for) {
        const recommendations = model.recommended_for.slice(0, 2).join(', ')
        return `(${recommendations})`
      }
      return ''
    }

    const getCodeSizeStatus = () => {
      const length = codeInput.value.length
      if (length > 40000) return '⚠️ Large'
      if (length > 20000) return '📊 Medium'
      return '✅ OK'
    }

    const getPlaceholderText = () => {
      const operation = enhancedOperations.find(op => op.id === selectedOperation.value)
      return `Paste your ${selectedLanguage.value === 'auto' ? '' : selectedLanguage.value + ' '}code here for ${operation?.name.toLowerCase() || 'analysis'}...`
    }

    const getAnalyzeButtonText = () => {
      if (isAnalyzing.value) return 'Analyzing...'
      if (isRetrying.value) return 'Retrying...'
      return `Analyze Code`
    }

    const getEmptyStateMessage = () => {
      const operation = enhancedOperations.find(op => op.id === selectedOperation.value)
      return `${operation?.description || 'Ready to analyze your code'} - paste your code or load a sample to get started.`
    }

    const getAnalysisProgressMessage = () => {
      const messages = [
        'Processing your code...',
        'Running AI analysis...',
        'Generating insights...',
        'Finalizing results...'
      ]
      const index = Math.floor(analysisProgress.value / 25)
      return messages[Math.min(index, messages.length - 1)]
    }

    // Language detection
    const detectLanguage = (code) => {
      if (selectedLanguage.value !== 'auto') return selectedLanguage.value

      const patterns = {
        python: [/def\s+\w+\s*\(/, /import\s+\w+/, /from\s+\w+\s+import/, /class\s+\w+/, /if\s+__name__\s*==\s*['""]__main__['""]/, /print\s*\(/],
        javascript: [/function\s+\w+\s*\(/, /const\s+\w+\s*=/, /let\s+\w+\s*=/, /var\s+\w+\s*=/, /=>\s*{/, /require\s*\(/, /console\.log\s*\(/],
        typescript: [/interface\s+\w+/, /type\s+\w+\s*=/, /enum\s+\w+/, /:\s*string/, /:\s*number/, /:\s*boolean/, /export\s+type/],
        rust: [/fn\s+\w+\s*\(/, /let\s+mut\s+\w+/, /use\s+std::/, /impl\s+\w+/, /struct\s+\w+/, /enum\s+\w+/, /match\s+\w+/, /Ok\s*\(/, /Err\s*\(/],
        go: [/func\s+\w+\s*\(/, /package\s+\w+/, /import\s+"/, /type\s+\w+\s+struct/, /var\s+\w+/, /fmt\.Print/, /go\s+func\s*\(/, /defer\s+/],
        java: [/public\s+class\s+\w+/, /private\s+\w+/, /protected\s+\w+/, /import\s+java\./, /System\.out/, /public\s+static\s+void\s+main/],
        'c++': [/#include\s*</, /using\s+namespace/, /int\s+main\s*\(/, /std::/, /class\s+\w+/, /template\s*</, /cout\s*<</, /cin\s*>>/],
        c: [/#include\s*</, /int\s+main\s*\(/, /printf\s*\(/, /scanf\s*\(/, /malloc\s*\(/, /free\s*\(/, /struct\s+\w+/, /typedef/]
      }

      let maxScore = 0
      let detectedLang = 'text'

      for (const [lang, langPatterns] of Object.entries(patterns)) {
        let score = 0
        for (const pattern of langPatterns) {
          if (pattern.test(code)) {
            score += 2
          }
        }
        if (score > maxScore) {
          maxScore = score
          detectedLang = lang
        }
      }

      return maxScore > 0 ? detectedLang : 'text'
    }

    // Format analysis result with syntax highlighting and structure
    const formatAnalysisResult = (result) => {
      if (!result) return ''
      
      // Basic markdown-like formatting
      let formatted = result
        .replace(/^### (.*$)/gm, '<h3 class="analysis-heading">$1</h3>')
        .replace(/^## (.*$)/gm, '<h2 class="analysis-heading">$1</h2>')
        .replace(/^# (.*$)/gm, '<h1 class="analysis-heading">$1</h1>')
        .replace(/\*\*(.*?)\*\*/g, '<strong class="highlight">$1</strong>')
        .replace(/\*(.*?)\*/g, '<em>$1</em>')
        .replace(/`([^`]+)`/g, '<code class="inline-code">$1</code>')
        .replace(/```(\w+)?\n([\s\S]*?)```/g, '<pre class="code-block"><code>$2</code></pre>')
        .replace(/^- (.*$)/gm, '<li class="bullet-point">$1</li>')
        .replace(/\n\n/g, '</p><p>')

      // Wrap in paragraphs
      formatted = '<p>' + formatted + '</p>'

      // Fix list formatting
      formatted = formatted.replace(/(<li class="bullet-point">.*<\/li>)/gs, '<ul>$1</ul>')

      return formatted
    }

    // Event handlers
    const onLanguageChange = () => {
      if (codeInput.value && selectedLanguage.value === 'auto') {
        detectedLanguage.value = detectLanguage(codeInput.value)
      }
    }

    const onCodeInput = () => {
      if (selectedLanguage.value === 'auto') {
        detectedLanguage.value = detectLanguage(codeInput.value)
      }
    }

    const onCodePaste = async (event) => {
      await nextTick()
      if (selectedLanguage.value === 'auto') {
        detectedLanguage.value = detectLanguage(codeInput.value)
      }
    }

    const toggleAdvancedMode = () => {
      advancedMode.value = !advancedMode.value
    }

    const toggleResultFormat = () => {
      resultFormatMode.value = resultFormatMode.value === 'formatted' ? 'raw' : 'formatted'
    }

    const clearError = () => {
      error.value = null
    }

    const loadSampleCode = () => {
      const lang = selectedLanguage.value === 'auto' ? 'python' : selectedLanguage.value
      codeInput.value = sampleCode[lang] || sampleCode.python
      onCodeInput()
    }

    const pasteFromClipboard = async () => {
      try {
        const text = await navigator.clipboard.readText()
        codeInput.value = text
        onCodeInput()
      } catch (err) {
        console.error('Failed to read clipboard:', err)
      }
    }

    const copyToClipboard = async (text) => {
      try {
        await navigator.clipboard.writeText(text)
        // Show temporary success feedback
      } catch (err) {
        console.error('Failed to copy to clipboard:', err)
      }
    }

    const exportResults = () => {
      if (!analysisResult.value) return
      
      const exportData = {
        timestamp: new Date().toISOString(),
        language: detectedLanguage.value || selectedLanguage.value,
        operation: selectedOperation.value,
        model: analysisResult.value.model_used,
        processing_time: analysisResult.value.processing_time_ms,
        code: codeInput.value,
        result: analysisResult.value.result
      }
      
      const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `analysis-${Date.now()}.json`
      a.click()
      URL.revokeObjectURL(url)
    }

    // Main analysis function
    const analyzeCode = async () => {
      if (!canAnalyze.value) return

      isAnalyzing.value = true
      error.value = null
      analysisResult.value = null
      analysisProgress.value = 0
      canCancelAnalysis.value = true

      // Simulate progress
      const progressInterval = setInterval(() => {
        if (analysisProgress.value < 90) {
          analysisProgress.value += Math.random() * 15
        }
      }, 500)

      try {
        const request = {
          code: codeInput.value,
          language: detectedLanguage.value || selectedLanguage.value,
          operation: selectedOperation.value,
          context: analysisContext.value || null,
          file_path: filePath.value || null
        }

        const result = await invoke('analyze_code_optimized', { request })
        
        analysisProgress.value = 100
        setTimeout(() => {
          analysisResult.value = result
          lastAnalysis.value = result
        }, 200)

      } catch (err) {
        console.error('Analysis failed:', err)
        error.value = typeof err === 'string' ? { message: err } : err
        
        if (error.value.retry_after) {
          startRetryCountdown(error.value.retry_after)
        }
      } finally {
        clearInterval(progressInterval)
        setTimeout(() => {
          isAnalyzing.value = false
          canCancelAnalysis.value = false
          analysisProgress.value = 0
        }, 300)
      }
    }

    const cancelAnalysis = () => {
      // In a real implementation, this would cancel the ongoing request
      isAnalyzing.value = false
      canCancelAnalysis.value = false
      analysisProgress.value = 0
    }

    const retryAnalysis = async () => {
      if (retryCountdown.value > 0) return
      
      isRetrying.value = true
      await analyzeCode()
      isRetrying.value = false
    }

    const startRetryCountdown = (seconds) => {
      retryCountdown.value = seconds
      const interval = setInterval(() => {
        retryCountdown.value--
        if (retryCountdown.value <= 0) {
          clearInterval(interval)
        }
      }, 1000)
    }

    const clearCache = async () => {
      isClearing.value = true
      try {
        await invoke('clear_analysis_cache')
      } catch (err) {
        console.error('Failed to clear cache:', err)
      } finally {
        isClearing.value = false
      }
    }

    const loadAvailableModels = async () => {
      try {
        const models = await invoke('get_available_models_enhanced')
        availableModels.value = models
      } catch (err) {
        console.error('Failed to load models:', err)
        availableModels.value = []
      }
    }

    // Lifecycle
    onMounted(() => {
      loadAvailableModels()
    })

    // Watchers
    watch(enableCaching, async (newValue) => {
      try {
        const config = await invoke('get_config')
        config.enable_caching = newValue
        await invoke('update_config', { newConfig: config })
      } catch (err) {
        console.error('Failed to update caching setting:', err)
      }
    })

    return {
      // State
      codeInput,
      selectedLanguage,
      selectedOperation,
      selectedModel,
      isAnalyzing,
      analysisResult,
      lastAnalysis,
      error,
      advancedMode,
      analysisContext,
      filePath,
      enableCaching,
      availableModels,
      detectedLanguage,
      resultFormatMode,
      isClearing,
      isRetrying,
      retryCountdown,
      analysisProgress,
      canCancelAnalysis,
      
      // Data
      enhancedOperations,
      
      // Computed
      canAnalyze,
      
      // Methods
      getOperationName,
      getModelRecommendation,
      getCodeSizeStatus,
      getPlaceholderText,
      getAnalyzeButtonText,
      getEmptyStateMessage,
      getAnalysisProgressMessage,
      formatAnalysisResult,
      
      // Event Handlers
      onLanguageChange,
      onCodeInput,
      onCodePaste,
      toggleAdvancedMode,
      toggleResultFormat,
      clearError,
      loadSampleCode,
      pasteFromClipboard,
      copyToClipboard,
      exportResults,
      analyzeCode,
      cancelAnalysis,
      retryAnalysis,
      clearCache
    }
  }
}
</script>

<style scoped>
/* Enhanced styles with animations and modern design */
.optimized-code-analyzer {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.analyzer-header {
  padding: 1.5rem;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left h3 {
  margin: 0 0 0.5rem 0;
  font-size: 1.5rem;
  background: linear-gradient(45deg, #FFD700, #FFA500);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.performance-metrics {
  display: flex;
  gap: 1rem;
  font-size: 0.75rem;
  opacity: 0.8;
}

.metric {
  padding: 0.25rem 0.5rem;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 0.25rem;
}

.header-controls {
  display: flex;
  gap: 1rem;
  align-items: center;
}

.language-select {
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: white;
  border-radius: 0.5rem;
  padding: 0.5rem;
}

.advanced-panel {
  background: rgba(255, 255, 255, 0.05);
  padding: 1rem 1.5rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.advanced-controls {
  display: flex;
  gap: 1rem;
  flex: 1;
}

.control-group {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.control-group .form-label {
  font-size: 0.75rem;
  opacity: 0.8;
}

.control-group .form-input {
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: white;
  border-radius: 0.25rem;
  padding: 0.25rem 0.5rem;
  font-size: 0.875rem;
}

.cache-controls {
  display: flex;
  gap: 1rem;
  align-items: center;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
}

.analyzer-body {
  flex: 1;
  display: flex;
  gap: 1rem;
  padding: 1rem;
  overflow: hidden;
}

.code-input-section,
.analysis-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  border-radius: 1rem;
  border: 1px solid rgba(255, 255, 255, 0.2);
  overflow: hidden;
}

.section-header {
  padding: 1rem;
  background: rgba(255, 255, 255, 0.1);
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.input-controls {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.code-stats {
  display: flex;
  gap: 0.5rem;
  font-size: 0.75rem;
  opacity: 0.8;
}

.stat {
  padding: 0.25rem 0.5rem;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 0.25rem;
}

.stat.warning {
  background: rgba(255, 193, 7, 0.3);
}

.operation-section {
  padding: 1rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.operation-buttons {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.operation-btn {
  padding: 0.5rem 1rem;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: white;
  border-radius: 0.5rem;
  transition: all 0.3s ease;
  cursor: pointer;
}

.operation-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: translateY(-2px);
}

.operation-btn.active {
  background: linear-gradient(45deg, #FFD700, #FFA500);
  color: #333;
  border-color: #FFD700;
  box-shadow: 0 4px 15px rgba(255, 215, 0, 0.3);
}

.code-input-container {
  flex: 1;
  position: relative;
  margin: 1rem;
}

.code-input {
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: white;
  border-radius: 0.5rem;
  padding: 1rem;
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  resize: none;
  backdrop-filter: blur(5px);
}

.code-input:focus {
  outline: none;
  border-color: #FFD700;
  box-shadow: 0 0 20px rgba(255, 215, 0, 0.3);
}

.input-overlay {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
  pointer-events: none;
}

.detected-language {
  background: rgba(255, 215, 0, 0.2);
  color: #FFD700;
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  backdrop-filter: blur(5px);
}

.analysis-controls {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.analyze-btn {
  background: linear-gradient(45deg, #4CAF50, #45a049);
  border: none;
  color: white;
  padding: 0.75rem 1.5rem;
  border-radius: 0.5rem;
  font-weight: bold;
  transition: all 0.3s ease;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.analyze-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(76, 175, 80, 0.3);
}

.analyze-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.analyze-btn.loading {
  background: linear-gradient(45deg, #FF9800, #F57C00);
}

.loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid transparent;
  border-top: 2px solid white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.btn-icon {
  background: none;
  border: none;
  color: white;
  cursor: pointer;
  padding: 0.5rem;
  border-radius: 0.25rem;
  transition: background-color 0.3s ease;
}

.btn-icon:hover {
  background: rgba(255, 255, 255, 0.1);
}

.analysis-result {
  flex: 1;
  display: flex;
  flex-direction: column;
  margin: 1rem;
  background: rgba(0, 0, 0, 0.3);
  border-radius: 0.5rem;
  overflow: hidden;
  backdrop-filter: blur(5px);
}

.result-header {
  background: rgba(255, 255, 255, 0.1);
  padding: 1rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.result-meta {
  display: flex;
  gap: 1rem;
  font-size: 0.75rem;
  opacity: 0.8;
}

.result-actions {
  display: flex;
  gap: 0.5rem;
}

.result-content-wrapper {
  flex: 1;
  overflow-y: auto;
}

.result-content {
  padding: 1rem;
  white-space: pre-wrap;
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  line-height: 1.6;
}

.result-content.formatted {
  font-family: inherit;
}

.result-content.formatted :deep(.analysis-heading) {
  color: #FFD700;
  margin: 1rem 0 0.5rem 0;
}

.result-content.formatted :deep(.highlight) {
  background: rgba(255, 215, 0, 0.2);
  color: #FFD700;
  padding: 0.125rem 0.25rem;
  border-radius: 0.25rem;
}

.result-content.formatted :deep(.inline-code) {
  background: rgba(255, 255, 255, 0.1);
  color: #FFD700;
  padding: 0.125rem 0.25rem;
  border-radius: 0.25rem;
  font-family: 'Courier New', monospace;
}

.result-content.formatted :deep(.code-block) {
  background: rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 0.5rem;
  padding: 1rem;
  margin: 1rem 0;
  overflow-x: auto;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  text-align: center;
}

.placeholder-animation {
  position: relative;
  margin-bottom: 2rem;
}

.placeholder-icon {
  font-size: 4rem;
  animation: float 3s ease-in-out infinite;
}

.placeholder-waves {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}

.wave {
  width: 80px;
  height: 80px;
  border: 2px solid rgba(255, 215, 0, 0.3);
  border-radius: 50%;
  position: absolute;
  animation: wave 2s linear infinite;
}

.wave:nth-child(2) {
  animation-delay: 0.5s;
}

.wave:nth-child(3) {
  animation-delay: 1s;
}

@keyframes float {
  0%, 100% { transform: translateY(0px); }
  50% { transform: translateY(-10px); }
}

@keyframes wave {
  0% {
    transform: scale(0);
    opacity: 1;
  }
  100% {
    transform: scale(1);
    opacity: 0;
  }
}

.quick-actions {
  display: flex;
  gap: 1rem;
  margin-top: 1rem;
}

.btn-outline {
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: white;
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
  transition: all 0.3s ease;
}

.btn-outline:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: #FFD700;
}

.analyzing-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  text-align: center;
}

.analysis-animation {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 2rem;
}

.brain-icon {
  font-size: 3rem;
  animation: pulse 2s ease-in-out infinite;
}

.thinking-dots {
  display: flex;
  gap: 0.5rem;
}

.dot {
  width: 8px;
  height: 8px;
  background: #FFD700;
  border-radius: 50%;
  animation: thinking 1.5s ease-in-out infinite;
}

.dot:nth-child(2) {
  animation-delay: 0.2s;
}

.dot:nth-child(3) {
  animation-delay: 0.4s;
}

@keyframes pulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.1); }
}

@keyframes thinking {
  0%, 60%, 100% {
    transform: translateY(0);
    opacity: 0.5;
  }
  30% {
    transform: translateY(-10px);
    opacity: 1;
  }
}

.progress-bar {
  width: 100%;
  max-width: 300px;
  height: 4px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
  overflow: hidden;
  margin: 1rem 0;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #FFD700, #FFA500);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.error-message.enhanced {
  margin: 1rem;
  background: rgba(244, 67, 54, 0.1);
  border: 1px solid rgba(244, 67, 54, 0.3);
  border-radius: 0.5rem;
  overflow: hidden;
  backdrop-filter: blur(5px);
}

.error-header {
  background: rgba(244, 67, 54, 0.2);
  padding: 1rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.error-icon {
  font-size: 1.25rem;
}

.error-content {
  padding: 1rem;
}

.error-details {
  margin-top: 1rem;
}

.error-details summary {
  cursor: pointer;
  font-weight: bold;
  margin-bottom: 0.5rem;
}

.error-details pre {
  background: rgba(0, 0, 0, 0.3);
  padding: 1rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  overflow-x: auto;
}

.error-actions {
  margin-top: 1rem;
  display: flex;
  gap: 0.5rem;
}
</style>
