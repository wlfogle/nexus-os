use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use tree_sitter::{Language, Parser, Query, QueryCursor, Node};
use walkdir::WalkDir;
use std::fs;

// Language support
extern "C" {
    fn tree_sitter_rust() -> Language;
    fn tree_sitter_python() -> Language;
    fn tree_sitter_javascript() -> Language;
    fn tree_sitter_typescript() -> Language;
    fn tree_sitter_go() -> Language;
    fn tree_sitter_cpp() -> Language;
    fn tree_sitter_java() -> Language;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeElement {
    pub id: String,
    pub name: String,
    pub element_type: CodeElementType,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub complexity: f32,
    pub dependencies: Vec<String>,
    pub documentation: Option<String>,
    pub test_coverage: Option<f32>,
    pub last_modified: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodeElementType {
    Function,
    Class,
    Module,
    Variable,
    Constant,
    Interface,
    Struct,
    Enum,
    Trait,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseInsight {
    pub total_files: usize,
    pub total_lines: usize,
    pub languages: HashMap<String, usize>,
    pub complexity_score: f32,
    pub test_coverage: f32,
    pub technical_debt: Vec<TechnicalDebt>,
    pub architecture_patterns: Vec<String>,
    pub security_issues: Vec<SecurityIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDebt {
    pub file_path: String,
    pub debt_type: String,
    pub severity: String,
    pub description: String,
    pub estimated_hours: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub file_path: String,
    pub issue_type: String,
    pub severity: String,
    pub description: String,
    pub cwe_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSuggestion {
    pub suggestion_type: String,
    pub file_path: String,
    pub line_number: usize,
    pub current_code: String,
    pub suggested_code: String,
    pub reasoning: String,
    pub confidence: f32,
}

pub struct AdvancedCodeIntelligence {
    parsers: Arc<RwLock<HashMap<String, Parser>>>,
    code_elements: Arc<RwLock<HashMap<String, CodeElement>>>,
    project_insights: Arc<RwLock<Option<CodebaseInsight>>>,
    suggestions_cache: Arc<RwLock<HashMap<String, Vec<CodeSuggestion>>>>,
}

impl AdvancedCodeIntelligence {
    pub fn new() -> Result<Self, AppError> {
        let mut parsers = HashMap::new();
        
        // Initialize parsers for different languages
        let languages = vec![
            ("rust", unsafe { tree_sitter_rust() }),
            ("python", unsafe { tree_sitter_python() }),
            ("javascript", unsafe { tree_sitter_javascript() }),
            ("typescript", unsafe { tree_sitter_typescript() }),
        ];

        for (lang, language) in languages {
            let mut parser = Parser::new();
            parser.set_language(language).map_err(|e| AppError::Validation(format!("Failed to set language {}: {}", lang, e)))?;
            parsers.insert(lang.to_string(), parser);
        }

        Ok(Self {
            parsers: Arc::new(RwLock::new(parsers)),
            code_elements: Arc::new(RwLock::new(HashMap::new())),
            project_insights: Arc::new(RwLock::new(None)),
            suggestions_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    // Deep code analysis with semantic understanding
    pub async fn analyze_codebase(&self, project_path: &Path) -> Result<CodebaseInsight, AppError> {
        let mut total_files = 0;
        let mut total_lines = 0;
        let mut languages = HashMap::new();
        let mut complexity_scores = Vec::new();
        let mut technical_debt = Vec::new();
        let mut security_issues = Vec::new();

        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                let language = self.detect_language(extension);
                if !language.is_empty() {
                    total_files += 1;
                    *languages.entry(language.clone()).or_insert(0) += 1;

                    // Analyze individual file
                    if let Ok(content) = fs::read_to_string(path) {
                        let lines = content.lines().count();
                        total_lines += lines;

                        // Calculate complexity
                        let complexity = self.calculate_complexity(&content, &language).await?;
                        complexity_scores.push(complexity);

                        // Detect technical debt
                        let debt = self.detect_technical_debt(path, &content).await?;
                        technical_debt.extend(debt);

                        // Security analysis
                        let security = self.analyze_security(path, &content, &language).await?;
                        security_issues.extend(security);
                    }
                }
            }
        }

        let average_complexity = if complexity_scores.is_empty() {
            0.0
        } else {
            complexity_scores.iter().sum::<f32>() / complexity_scores.len() as f32
        };

        let architecture_patterns = self.detect_architecture_patterns(project_path).await?;

        let insight = CodebaseInsight {
            total_files,
            total_lines,
            languages,
            complexity_score: average_complexity,
            test_coverage: self.calculate_test_coverage(project_path).await?,
            technical_debt,
            architecture_patterns,
            security_issues,
        };

        // Cache the insights
        let mut insights = self.project_insights.write().await;
        *insights = Some(insight.clone());

        Ok(insight)
    }

    // Generate intelligent code suggestions
    pub async fn generate_suggestions(&self, file_path: &Path, content: &str) -> Result<Vec<CodeSuggestion>, AppError> {
        let mut suggestions = Vec::new();
        let language = self.detect_language(
            file_path.extension().and_then(|s| s.to_str()).unwrap_or("")
        );

        // Parse the code
        let parsers = self.parsers.read().await;
        if let Some(parser) = parsers.get(&language) {
            // This would be expanded with actual parsing logic
            // For now, providing example suggestions
            
            // Detect long functions
            let lines: Vec<&str> = content.lines().collect();
            let mut in_function = false;
            let mut function_start = 0;
            let mut brace_count = 0;

            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                
                // Simple function detection (would be more sophisticated with AST)
                if (trimmed.starts_with("fn ") || trimmed.starts_with("def ") || 
                   trimmed.starts_with("function ")) && !in_function {
                    in_function = true;
                    function_start = i;
                    brace_count = 0;
                }

                if in_function {
                    brace_count += trimmed.chars().filter(|&c| c == '{').count() as i32;
                    brace_count -= trimmed.chars().filter(|&c| c == '}').count() as i32;

                    if brace_count == 0 && trimmed.contains('}') {
                        let function_length = i - function_start;
                        if function_length > 50 {
                            suggestions.push(CodeSuggestion {
                                suggestion_type: "refactor".to_string(),
                                file_path: file_path.to_string_lossy().to_string(),
                                line_number: function_start + 1,
                                current_code: lines[function_start..=i].join("\n"),
                                suggested_code: "// Consider breaking this function into smaller functions".to_string(),
                                reasoning: format!("Function is {} lines long, consider refactoring for better maintainability", function_length),
                                confidence: 0.8,
                            });
                        }
                        in_function = false;
                    }
                }
            }

            // Detect code duplication
            let duplication_suggestions = self.detect_code_duplication(file_path, content).await?;
            suggestions.extend(duplication_suggestions);

            // Performance optimization suggestions
            let performance_suggestions = self.suggest_performance_optimizations(file_path, content, &language).await?;
            suggestions.extend(performance_suggestions);
        }

        // Cache suggestions
        let mut cache = self.suggestions_cache.write().await;
        cache.insert(file_path.to_string_lossy().to_string(), suggestions.clone());

        Ok(suggestions)
    }

    // Advanced refactoring capabilities
    pub async fn suggest_refactoring(&self, file_path: &Path, selection: Option<(usize, usize)>) -> Result<Vec<CodeSuggestion>, AppError> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| AppError::FileSystem(format!("Failed to read file: {}", e)))?;

        let mut suggestions = Vec::new();

        // Extract method refactoring
        if let Some((start, end)) = selection {
            let lines: Vec<&str> = content.lines().collect();
            if start < lines.len() && end < lines.len() && start < end {
                let selected_code = lines[start..=end].join("\n");
                
                suggestions.push(CodeSuggestion {
                    suggestion_type: "extract_method".to_string(),
                    file_path: file_path.to_string_lossy().to_string(),
                    line_number: start + 1,
                    current_code: selected_code.clone(),
                    suggested_code: self.generate_extracted_method(&selected_code).await?,
                    reasoning: "Extract selected code into a separate method for better organization".to_string(),
                    confidence: 0.75,
                });
            }
        }

        // Suggest design pattern implementations
        let pattern_suggestions = self.suggest_design_patterns(file_path, &content).await?;
        suggestions.extend(pattern_suggestions);

        Ok(suggestions)
    }

    // Real-time code quality analysis
    pub async fn analyze_code_quality(&self, content: &str, language: &str) -> Result<serde_json::Value, AppError> {
        let complexity = self.calculate_complexity(content, language).await?;
        let maintainability = self.calculate_maintainability(content, language).await?;
        let readability = self.calculate_readability(content).await?;
        
        let quality_score = (complexity * 0.3 + maintainability * 0.4 + readability * 0.3).min(10.0);

        Ok(serde_json::json!({
            "overall_score": quality_score,
            "complexity": complexity,
            "maintainability": maintainability,
            "readability": readability,
            "recommendations": self.generate_quality_recommendations(quality_score, complexity, maintainability, readability).await?
        }))
    }

    // Helper methods
    fn detect_language(&self, extension: &str) -> String {
        match extension {
            "rs" => "rust".to_string(),
            "py" => "python".to_string(),
            "js" => "javascript".to_string(),
            "ts" => "typescript".to_string(),
            "go" => "go".to_string(),
            "cpp" | "cc" | "cxx" => "cpp".to_string(),
            "java" => "java".to_string(),
            _ => "unknown".to_string(),
        }
    }

    async fn calculate_complexity(&self, _content: &str, _language: &str) -> Result<f32, AppError> {
        // Simplified complexity calculation
        // In reality, this would use AST analysis
        Ok(5.0) // Placeholder
    }

    async fn calculate_maintainability(&self, _content: &str, _language: &str) -> Result<f32, AppError> {
        Ok(7.5) // Placeholder
    }

    async fn calculate_readability(&self, content: &str) -> Result<f32, AppError> {
        let lines = content.lines().count();
        let avg_line_length = if lines > 0 {
            content.len() as f32 / lines as f32
        } else {
            0.0
        };
        
        // Simple readability score based on line length
        let readability = (100.0 - avg_line_length.min(100.0)) / 10.0;
        Ok(readability.max(1.0).min(10.0))
    }

    async fn detect_technical_debt(&self, file_path: &Path, content: &str) -> Result<Vec<TechnicalDebt>, AppError> {
        let mut debt = Vec::new();
        
        // Look for TODO, FIXME, HACK comments
        for (i, line) in content.lines().enumerate() {
            let lower_line = line.to_lowercase();
            if lower_line.contains("todo") || lower_line.contains("fixme") || lower_line.contains("hack") {
                debt.push(TechnicalDebt {
                    file_path: file_path.to_string_lossy().to_string(),
                    debt_type: "comment_debt".to_string(),
                    severity: "medium".to_string(),
                    description: format!("Technical debt indicator found: {}", line.trim()),
                    estimated_hours: 2.0,
                });
            }
        }

        Ok(debt)
    }

    async fn analyze_security(&self, file_path: &Path, content: &str, language: &str) -> Result<Vec<SecurityIssue>, AppError> {
        let mut issues = Vec::new();
        
        // Basic security pattern detection
        let security_patterns = match language {
            "python" => vec![
                ("eval(", "Code Injection", "high"),
                ("exec(", "Code Injection", "high"),
                ("os.system(", "Command Injection", "high"),
            ],
            "javascript" => vec![
                ("eval(", "Code Injection", "high"),
                ("innerHTML =", "XSS Vulnerability", "medium"),
                ("document.write(", "XSS Vulnerability", "medium"),
            ],
            _ => vec![],
        };

        for (i, line) in content.lines().enumerate() {
            for (pattern, issue_type, severity) in &security_patterns {
                if line.contains(pattern) {
                    issues.push(SecurityIssue {
                        file_path: file_path.to_string_lossy().to_string(),
                        issue_type: issue_type.to_string(),
                        severity: severity.to_string(),
                        description: format!("Potential {} found on line {}: {}", issue_type, i + 1, line.trim()),
                        cwe_id: None,
                    });
                }
            }
        }

        Ok(issues)
    }

    async fn calculate_test_coverage(&self, _project_path: &Path) -> Result<f32, AppError> {
        // Placeholder - would integrate with coverage tools
        Ok(75.0)
    }

    async fn detect_architecture_patterns(&self, _project_path: &Path) -> Result<Vec<String>, AppError> {
        // Placeholder - would analyze project structure
        Ok(vec!["MVC".to_string(), "Repository Pattern".to_string()])
    }

    async fn detect_code_duplication(&self, _file_path: &Path, _content: &str) -> Result<Vec<CodeSuggestion>, AppError> {
        // Placeholder for duplication detection
        Ok(vec![])
    }

    async fn suggest_performance_optimizations(&self, _file_path: &Path, _content: &str, _language: &str) -> Result<Vec<CodeSuggestion>, AppError> {
        // Placeholder for performance suggestions
        Ok(vec![])
    }

    async fn generate_extracted_method(&self, _code: &str) -> Result<String, AppError> {
        // Placeholder for method extraction
        Ok("// Generated extracted method\nfn extracted_method() {\n    // TODO: Implement extracted logic\n}".to_string())
    }

    async fn suggest_design_patterns(&self, _file_path: &Path, _content: &str) -> Result<Vec<CodeSuggestion>, AppError> {
        // Placeholder for design pattern suggestions
        Ok(vec![])
    }

    async fn generate_quality_recommendations(&self, _overall: f32, _complexity: f32, _maintainability: f32, _readability: f32) -> Result<Vec<String>, AppError> {
        Ok(vec![
            "Consider adding more documentation".to_string(),
            "Break down complex functions".to_string(),
            "Add unit tests for better coverage".to_string(),
        ])
    }
}
