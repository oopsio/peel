use anyhow::Result;
use peeldoc_models::*;
use regex::Regex;
use std::fs;

pub struct Parser {
    item_regex_fn: Regex,
    doc_comment_regex: Regex,
    doc_tag_regex: Regex,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            // Match (async)? fn <name>(<params>) [-> <ret_type>]?
            item_regex_fn: Regex::new(r"(?m)(async\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*(?:->\s*([^{]*))?").unwrap(),
            // Match lines starting with ///
            doc_comment_regex: Regex::new(r"(?m)^(\s*///\s*(.*))").unwrap(),
            // Match @<tag> <content>
            doc_tag_regex: Regex::new(r"@(\w+)\s+(.*)").unwrap(),
        }
    }

    pub fn parse_file(&self, path: &str, content: &str) -> Result<ModuleDoc> {
        let mut items = Vec::new();

        // Very basic extraction: find all line-by-line documentation blocks
        // This is a naive implementation; a real one would step through tokens or lines.
        let lines: Vec<&str> = content.lines().collect();
        let mut current_docs = Vec::new();

        for i in 0..lines.len() {
            let line = lines[i];
            
            if let Some(caps) = self.doc_comment_regex.captures(line) {
                current_docs.push(caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string());
            } else if !current_docs.is_empty() {
                // Peek at this line to see if it's a function
                if let Some(caps) = self.item_regex_fn.captures(line) {
                    let is_async = caps.get(1).is_some();
                    let name = caps.get(2).map(|m| m.as_str()).unwrap().to_string();
                    let params_raw = caps.get(3).map(|m| m.as_str()).unwrap_or("");
                    let return_type = caps.get(4).map(|m| m.as_str().trim().to_string());

                    // Process doc comment for description and tags
                    let mut description_lines = Vec::new();
                    let mut tags = Vec::new();
                    let mut params_docs = Vec::new();

                    for doc_line in &current_docs {
                        if let Some(tag_caps) = self.doc_tag_regex.captures(doc_line) {
                            let tag_name = tag_caps.get(1).unwrap().as_str();
                            let tag_content = tag_caps.get(2).unwrap().as_str();

                            if tag_name == "param" {
                                if let Some((p_name, p_desc)) = tag_content.split_once(' ') {
                                    params_docs.push(ParamDoc {
                                        name: p_name.to_string(),
                                        description: Some(p_desc.to_string()),
                                        param_type: None, // Could parse name: type
                                    });
                                } else {
                                    params_docs.push(ParamDoc {
                                        name: tag_content.to_string(),
                                        description: None,
                                        param_type: None,
                                    });
                                }
                            } else {
                                tags.push(Tag {
                                    name: tag_name.to_string(),
                                    content: tag_content.to_string(),
                                });
                            }
                        } else {
                            description_lines.push(doc_line.clone());
                        }
                    }

                    // Parse parameters for types if possible
                    let mut params = Vec::new();
                    for p in params_raw.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                        if let Some((p_name, p_type)) = p.split_once(':') {
                            let name = p_name.trim().to_string();
                            let type_str = p_type.trim().to_string();
                            
                            // Try to find matching doc
                            let doc = params_docs.iter().find(|pd| pd.name == name).map(|pd| pd.description.clone()).flatten();
                            params.push(ParamDoc {
                                name,
                                description: doc,
                                param_type: Some(type_str),
                            });
                        } else {
                            params.push(ParamDoc {
                                name: p.to_string(),
                                description: None,
                                param_type: None,
                            });
                        }
                    }

                    items.push(DocItem::Function(FunctionDoc {
                        name,
                        description: description_lines.join("\n"),
                        is_async,
                        params,
                        return_type,
                        examples: Vec::new(), // Extract @example if needed
                        tags,
                    }));
                }
                current_docs.clear();
            }
        }

        let name = std::path::Path::new(path)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown".into());

        Ok(ModuleDoc {
            name,
            path: path.to_string(),
            items,
        })
    }
}
