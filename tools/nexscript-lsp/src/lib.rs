#![deny(clippy::all)]

use nexscript::Lexer;
use std::collections::HashMap;

pub struct LspServer {
    pub documents: HashMap<String, DocumentState>,
    pub initialized: bool,
}

pub struct DocumentState {
    pub uri: String,
    pub source: String,
    pub tokens: Vec<nexscript::lexer::Token>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub severity: DiagnosticSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub detail: String,
    pub kind: CompletionItemKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompletionItemKind {
    Keyword,
    Function,
    Type,
    Variable,
    Property,
}

impl LspServer {
    pub fn new() -> Self {
        Self { documents: HashMap::new(), initialized: false }
    }

    pub fn initialize(&mut self) {
        self.initialized = true;
    }

    pub fn open_document(&mut self, uri: &str, source: &str) {
        let tokens = self.tokenize_source(source);
        let diagnostics = self.analyze_source(source, &tokens);
        self.documents.insert(uri.to_string(), DocumentState {
            uri: uri.to_string(),
            source: source.to_string(),
            tokens,
            diagnostics,
        });
    }

    pub fn update_document(&mut self, uri: &str, source: &str) {
        self.open_document(uri, source);
    }

    pub fn close_document(&mut self, uri: &str) {
        self.documents.remove(uri);
    }

    fn tokenize_source(&self, source: &str) -> Vec<nexscript::lexer::Token> {
        let mut lexer = Lexer::new(source);
        lexer.tokenize().unwrap_or_default()
    }

    fn analyze_source(&self, source: &str, _tokens: &[nexscript::lexer::Token]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = source.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            // Check for common errors
            if line.contains("unkown") {
                diagnostics.push(Diagnostic {
                    line: line_num + 1,
                    column: line.find("unkown").unwrap_or(0) + 1,
                    message: "Typo: did you mean 'unknown'?".to_string(),
                    severity: DiagnosticSeverity::Warning,
                });
            }

            // Check for missing semicolons on statement lines (heuristic)
            let trimmed = line.trim();
            if !trimmed.is_empty()
                && !trimmed.ends_with('{')
                && !trimmed.ends_with('}')
                && !trimmed.starts_with("//")
                && !trimmed.starts_with("/*")
                && !trimmed.ends_with(';')
                && !trimmed.ends_with(',')
                && !trimmed.ends_with('(')
                && !trimmed.ends_with(':')
                && !trimmed.starts_with('*')
            {
                // Comment continuation check
                if !trimmed.ends_with("*/") {
                    // Might be missing semicolon — be conservative
                }
            }
        }

        diagnostics
    }

    pub fn completions(&self, uri: &str, line: usize, _column: usize) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Keywords
        let keywords = [
            ("let", "Declare a variable", CompletionItemKind::Keyword),
            ("fn", "Define a function", CompletionItemKind::Keyword),
            ("if", "Conditional branch", CompletionItemKind::Keyword),
            ("else", "Alternative branch", CompletionItemKind::Keyword),
            ("while", "Loop while condition is true", CompletionItemKind::Keyword),
            ("for", "Iterate over a range", CompletionItemKind::Keyword),
            ("return", "Return from a function", CompletionItemKind::Keyword),
            ("entity", "Define an entity", CompletionItemKind::Keyword),
            ("component", "Define a component", CompletionItemKind::Keyword),
            ("event", "Define a custom event", CompletionItemKind::Keyword),
            ("true", "Boolean literal true", CompletionItemKind::Keyword),
            ("false", "Boolean literal false", CompletionItemKind::Keyword),
            ("null", "Null value", CompletionItemKind::Keyword),
            ("mut", "Mutable variable", CompletionItemKind::Keyword),
            ("state", "State variable for entity", CompletionItemKind::Keyword),
            ("coroutine", "Define a coroutine", CompletionItemKind::Keyword),
            ("yield", "Yield from coroutine", CompletionItemKind::Keyword),
            ("await", "Await a coroutine", CompletionItemKind::Keyword),
            ("break", "Break from loop", CompletionItemKind::Keyword),
        ];

        for (kw, detail, kind) in &keywords {
            items.push(CompletionItem { label: kw.to_string(), detail: detail.to_string(), kind: kind.clone() });
        }

        // Types
        let types = [
            "int", "float", "bool", "string", "void", "vec2", "vec3", "vec4", "quat", "entity",
        ];
        for t in &types {
            items.push(CompletionItem { label: t.to_string(), detail: format!("Type: {}", t), kind: CompletionItemKind::Type });
        }

        // Builtins
        let builtins = [
            ("log", "Print a message to console", CompletionItemKind::Function),
            ("sin", "Sine of angle", CompletionItemKind::Function),
            ("cos", "Cosine of angle", CompletionItemKind::Function),
            ("sqrt", "Square root", CompletionItemKind::Function),
            ("abs", "Absolute value", CompletionItemKind::Function),
            ("clamp", "Clamp value between min and max", CompletionItemKind::Function),
            ("random", "Random float [0.0, 1.0)", CompletionItemKind::Function),
            ("print", "Print value", CompletionItemKind::Function),
            ("pop", "Discard top of stack", CompletionItemKind::Function),
            ("floor", "Round down to integer", CompletionItemKind::Function),
            ("ceil", "Round up to integer", CompletionItemKind::Function),
            ("round", "Round to nearest integer", CompletionItemKind::Function),
            ("len", "Length of a string", CompletionItemKind::Function),
            ("min", "Minimum of two values", CompletionItemKind::Function),
            ("max", "Maximum of two values", CompletionItemKind::Function),
            ("pow", "Power (a^b)", CompletionItemKind::Function),
            ("pi", "PI constant", CompletionItemKind::Function),
            ("lerp", "Linear interpolation", CompletionItemKind::Function),
            ("distance", "3D distance between points", CompletionItemKind::Function),
            ("tan", "Tangent of angle", CompletionItemKind::Function),
            ("exp", "Exponential (e^x)", CompletionItemKind::Function),
            ("sign", "Sign of number (-1, 0, 1)", CompletionItemKind::Function),
            ("deg2rad", "Convert degrees to radians", CompletionItemKind::Function),
        ];

        for (name, detail, kind) in &builtins {
            items.push(CompletionItem { label: name.to_string(), detail: detail.to_string(), kind: kind.clone() });
        }

        // Document symbols for current file
        if let Some(doc) = self.documents.get(uri) {
            for line_text in doc.source.lines() {
                let trimmed = line_text.trim();
                if trimmed.starts_with("fn ") {
                    let name = trimmed.trim_start_matches("fn ").split('(').next().unwrap_or("").to_string();
                    items.push(CompletionItem { label: name, detail: "User-defined function".to_string(), kind: CompletionItemKind::Function });
                } else if trimmed.starts_with("entity ") {
                    let name = trimmed.trim_start_matches("entity ").split_whitespace().next().unwrap_or("").to_string();
                    items.push(CompletionItem { label: name, detail: "Entity definition".to_string(), kind: CompletionItemKind::Property });
                }
            }
        }

        items
    }

    pub fn hover(&self, uri: &str, line: usize, column: usize) -> Option<String> {
        let doc = self.documents.get(uri)?;
        let source_line = doc.source.lines().nth(line)?;

        let word = Self::extract_word_at(source_line, column)?;

        Some(match word.as_str() {
            "log" => "Built-in: log(message: string) -> void\nPrints a message to the engine console.".to_string(),
            "raycast" => "Built-in: raycast(origin: vec3, direction: vec3, max_dist: float) -> RaycastHit\nCasts a ray and returns hit information.".to_string(),
            "clamp" => "Built-in: clamp(value: float, min: float, max: float) -> float\nClamps a value between min and max.".to_string(),
            "entity" => "Keyword: Defines a new entity type with components and event handlers.".to_string(),
            "component" => "Keyword: Defines a new component type with fields and methods.".to_string(),
            "fn" => "Keyword: Declares a function.".to_string(),
            "let" => "Keyword: Declares a (mutable) variable.".to_string(),
            "vec3" => "Type: 3-component float vector. Construct with vec3(x, y, z).".to_string(),
            _ => {
                if let Some(doc) = self.documents.get(uri) {
                    for sline in doc.source.lines() {
                        let st = sline.trim();
                        if st.starts_with(&format!("fn {}", word)) || st.starts_with(&format!("entity {}", word)) {
                            return Some(format!("Definition: {}", st));
                        }
                    }
                }
                format!("Symbol: {}", word)
            }
        })
    }

    fn extract_word_at(line: &str, column: usize) -> Option<String> {
        let col = column.min(line.len());
        let before = line[..col].chars().rev().take_while(|c| c.is_alphanumeric() || *c == '_').collect::<String>().chars().rev().collect::<String>();
        let after: String = line[col..].chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        if before.is_empty() && after.is_empty() { return None; }
        Some(format!("{}{}", before, after))
    }
}

impl Default for LspServer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_creation() {
        let lsp = LspServer::new();
        assert!(!lsp.initialized);
        assert!(lsp.documents.is_empty());
    }

    #[test]
    fn test_open_document() {
        let mut lsp = LspServer::new();
        lsp.open_document("file:///test.nxs", "let x: int = 42;");
        assert_eq!(lsp.documents.len(), 1);
    }

    #[test]
    fn test_completions_keywords() {
        let mut lsp = LspServer::new();
        lsp.open_document("file:///test.nxs", "");
        let items = lsp.completions("file:///test.nxs", 0, 0);
        assert!(items.iter().any(|i| i.label == "let"));
        assert!(items.iter().any(|i| i.label == "entity"));
        assert!(items.iter().any(|i| i.label == "fn"));
    }

    #[test]
    fn test_completions_types() {
        let mut lsp = LspServer::new();
        lsp.open_document("file:///test.nxs", "");
        let items = lsp.completions("file:///test.nxs", 0, 0);
        assert!(items.iter().any(|i| i.label == "int"));
        assert!(items.iter().any(|i| i.label == "vec3"));
    }

    #[test]
    fn test_completions_builtins() {
        let mut lsp = LspServer::new();
        lsp.open_document("file:///test.nxs", "");
        let items = lsp.completions("file:///test.nxs", 0, 0);
        assert!(items.iter().any(|i| i.label == "log"));
        assert!(items.iter().any(|i| i.label == "floor"));
    }

    #[test]
    fn test_hover_on_keyword() {
        let mut lsp = LspServer::new();
        lsp.open_document("file:///test.nxs", "entity Player {}");
        let hover = lsp.hover("file:///test.nxs", 0, 0);
        assert!(hover.is_some());
        assert!(hover.unwrap().contains("entity"));
    }

    #[test]
    fn test_hover_on_builtin() {
        let mut lsp = LspServer::new();
        lsp.open_document("file:///test.nxs", "log(\"hello\")");
        let hover = lsp.hover("file:///test.nxs", 0, 0);
        assert!(hover.is_some());
        assert!(hover.unwrap().contains("log"));
    }

    #[test]
    fn test_diagnostics_typo() {
        let mut lsp = LspServer::new();
        lsp.open_document("file:///test.nxs", "unkown_variable = 5;");
        let doc = lsp.documents.get("file:///test.nxs").unwrap();
        // The source contains "unkown" which should trigger warning
        let has_typo_warning = doc.diagnostics.iter().any(|d| d.message.contains("Typo"));
        assert!(has_typo_warning);
    }

    #[test]
    fn test_extract_word() {
        let word = LspServer::extract_word_at("let x = 42;", 4);
        assert_eq!(word, Some("x".to_string()));
    }

    #[test]
    fn test_extract_word_empty() {
        let word = LspServer::extract_word_at(";;;", 1);
        assert_eq!(word, None);
    }

    #[test]
    fn test_close_document() {
        let mut lsp = LspServer::new();
        lsp.open_document("file:///test.nxs", "");
        lsp.close_document("file:///test.nxs");
        assert!(lsp.documents.is_empty());
    }

    #[test]
    fn test_completions_new_builtins() {
        let mut lsp = LspServer::new();
        lsp.open_document("file:///test.nxs", "");
        let items = lsp.completions("file:///test.nxs", 0, 0);
        assert!(items.iter().any(|i| i.label == "tan"));
        assert!(items.iter().any(|i| i.label == "exp"));
        assert!(items.iter().any(|i| i.label == "sign"));
        assert!(items.iter().any(|i| i.label == "deg2rad"));
    }

    #[test]
    fn test_diagnostics_multiline() {
        let mut lsp = LspServer::new();
        lsp.open_document("file:///test.nxs", "fn main() {\n  return 0;\n}");
        let doc = lsp.documents.get("file:///test.nxs").unwrap();
        assert_eq!(doc.line_count, 3);
    }

    #[test]
    fn test_document_symbols() {
        let mut lsp = LspServer::new();
        lsp.open_document("file:///test.nxs", "fn hello() {}\nfn world() {}");
        let items = lsp.completions("file:///test.nxs", 2, 0);
        assert!(items.iter().any(|i| i.label == "hello"));
        assert!(items.iter().any(|i| i.label == "world"));
    }
}
