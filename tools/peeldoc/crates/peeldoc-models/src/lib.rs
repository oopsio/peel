use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectDoc {
    pub name: String,
    pub description: Option<String>,
    pub modules: Vec<ModuleDoc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModuleDoc {
    pub name: String,
    pub path: String,
    pub items: Vec<DocItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum DocItem {
    Function(FunctionDoc),
    Variable(VariableDoc),
    Class(ClassDoc), // If Peel supports classes
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionDoc {
    pub name: String,
    pub description: String,
    pub is_async: bool,
    pub params: Vec<ParamDoc>,
    pub return_type: Option<String>,
    pub examples: Vec<String>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParamDoc {
    pub name: String,
    pub description: Option<String>,
    pub param_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VariableDoc {
    pub name: String,
    pub description: String,
    pub var_type: Option<String>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassDoc {
    pub name: String,
    pub description: String,
    pub members: Vec<DocItem>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub name: String,
    pub content: String,
}
