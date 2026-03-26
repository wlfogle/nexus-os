<template>
  <div class="code-analyzer">
    <div class="analyzer-header">
      <h3>🔍 Code Analysis</h3>
      <div class="header-controls">
        <select v-model="selectedLanguage" class="form-input language-select">
          <option value="auto">Auto Detect</option>
          <option value="python">Python</option>
          <option value="javascript">JavaScript</option>
          <option value="typescript">TypeScript</option>
          <option value="rust">Rust</option>
          <option value="go">Go</option>
          <option value="java">Java</option>
          <option value="c++">C++</option>
          <option value="c">C</option>
        </select>
      </div>
    </div>

    <div class="analyzer-body">
      <div class="code-input-section">
        <div class="section-header">
          <h4>Code Input</h4>
          <div class="operation-buttons">
            <button 
              v-for="operation in operations" 
              :key="operation.id"
              :class="['btn', 'operation-btn', { active: selectedOperation === operation.id }]"
              @click="selectedOperation = operation.id"
            >
              {{ operation.icon }} {{ operation.name }}
            </button>
          </div>
        </div>
        <textarea 
          v-model="codeInput" 
          class="code-input"
          placeholder="Paste your code here for AI analysis..."
          rows="15"
        ></textarea>
      </div>

      <div class="analysis-section">
        <div class="section-header">
          <h4>AI Analysis Results</h4>
          <button 
            class="btn btn-primary analyze-btn" 
            @click="analyzeCode" 
            :disabled="isAnalyzing || !codeInput.trim()"
          >
            <span v-if="isAnalyzing" class="loading"></span>
            {{ isAnalyzing ? 'Analyzing...' : 'Analyze Code' }}
          </button>
        </div>
        
        <div v-if="analysisResult" class="analysis-result">
          <div class="result-header">
            <span class="model-info">Model: {{ analysisResult.model_used }}</span>
            <span class="operation-info">Operation: {{ getOperationName(selectedOperation) }}</span>
          </div>
          <pre class="result-content">{{ analysisResult.result }}</pre>
        </div>
        
        <div v-else-if="!isAnalyzing" class="no-result">
          <div class="placeholder-icon">🤖</div>
          <p>Select an operation and paste your code to start analysis</p>
        </div>
        
        <div v-if="error" class="error-message">
          <strong>Error:</strong> {{ error }}
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

export default {
  name: 'CodeAnalyzer',
  setup() {
    const codeInput = ref('')
    const selectedLanguage = ref('auto')
    const selectedOperation = ref('analyze')
    const isAnalyzing = ref(false)
    const analysisResult = ref(null)
    const error = ref('')

    const operations = [
      { id: 'analyze', name: 'Analyze', icon: '🔍' },
      { id: 'fix_bugs', name: 'Fix Bugs', icon: '🐛' },
      { id: 'optimize', name: 'Optimize', icon: '⚡' },
      { id: 'document', name: 'Document', icon: '📚' },
      { id: 'test', name: 'Test', icon: '🧪' }
    ]

    const getOperationName = (operationId) => {
      const operation = operations.find(op => op.id === operationId)
      return operation ? operation.name : operationId
    }

    const detectLanguage = (code) => {
      if (selectedLanguage.value !== 'auto') {
        return selectedLanguage.value
      }
      
      // Simple language detection based on syntax patterns
      if (code.includes('def ') || code.includes('import ') || code.includes('from ')) {
        return 'python'
      } else if (code.includes('function ') || code.includes('const ') || code.includes('let ')) {
        return 'javascript'
      } else if (code.includes('fn ') || code.includes('let mut') || code.includes('use std::')) {
        return 'rust'
      } else if (code.includes('func ') || code.includes('package ') || code.includes('import "')) {
        return 'go'
      } else if (code.includes('#include') || code.includes('int main(')) {
        return 'c++'
      }
      
      return 'text'
    }

    const analyzeCode = async () => {
      if (!codeInput.value.trim()) return
      
      isAnalyzing.value = true
      error.value = ''
      analysisResult.value = null
      
      try {
        const language = detectLanguage(codeInput.value)
        const result = await invoke('analyze_code', {
          code: codeInput.value,
          language: language,
          operation: selectedOperation.value
        })
        
        analysisResult.value = result
      } catch (err) {
        error.value = err.toString()
        console.error('Analysis failed:', err)
      } finally {
        isAnalyzing.value = false
      }
    }

    return {
      codeInput,
      selectedLanguage,
      selectedOperation,
      isAnalyzing,
      analysisResult,
      error,
      operations,
      getOperationName,
      analyzeCode
    }
  }
}
</script>

<style scoped>
.code-analyzer {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
}

.analyzer-header {
  padding: 1.5rem;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.analyzer-header h3 {
  margin: 0;
  color: var(--primary-color);
}

.language-select {
  width: 150px;
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
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.section-header h4 {
  margin: 0;
  color: var(--text-primary);
}

.operation-buttons {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.operation-btn {
  padding: 0.25rem 0.75rem;
  font-size: 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  color: var(--text-secondary);
}

.operation-btn.active {
  background: var(--primary-color);
  color: white;
  border-color: var(--primary-color);
}

.code-input {
  flex: 1;
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  resize: none;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  padding: 1rem;
  background: var(--bg-secondary);
}

.analysis-result {
  flex: 1;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  overflow: hidden;
}

.result-header {
  padding: 0.75rem;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.result-content {
  flex: 1;
  padding: 1rem;
  overflow-y: auto;
  white-space: pre-wrap;
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  line-height: 1.5;
  background: var(--bg-primary);
}

.no-result {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  border: 2px dashed var(--border-color);
  border-radius: 0.375rem;
}

.placeholder-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
}

.error-message {
  padding: 1rem;
  background: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 0.375rem;
  color: var(--error-color);
  font-size: 0.875rem;
}

.analyze-btn {
  white-space: nowrap;
}
</style>
