use anyhow::Result;
use peeldoc_models::*;
use std::fs;
use std::path::Path;
use tera::{Context, Tera};

pub struct Ssg {
    tera: Tera,
}

impl Ssg {
    pub fn new() -> Self {
        let mut tera = Tera::default();
        // Register templates
        tera.add_raw_template("layout", include_str!("templates/layout.html")).unwrap();
        tera.add_raw_template("module", include_str!("templates/module.html")).unwrap();
        tera.add_raw_template("index", include_str!("templates/index.html")).unwrap();
        Self { tera }
    }

    pub async fn generate(&self, project: &ProjectDoc, out_dir: &str) -> Result<()> {
        let out_path = Path::new(out_dir);
        if !out_path.exists() {
            fs::create_dir_all(out_path)?;
        }

        // Generate Index
        let mut ctx = Context::new();
        ctx.insert("project", project);
        ctx.insert("base_path", ".");
        ctx.insert("current_module", "");
        let index_html = self.tera.render("index", &ctx)?;
        fs::write(out_path.join("index.html"), index_html)?;

        // Generate Modules
        let modules_dir = out_path.join("modules");
        if !modules_dir.exists() {
            fs::create_dir_all(&modules_dir)?;
        }

        for module in &project.modules {
            let mut mod_ctx = Context::new();
            mod_ctx.insert("project", project);
            mod_ctx.insert("module", module);
            mod_ctx.insert("current_module", &module.name);
            mod_ctx.insert("base_path", "..");
            let module_html = self.tera.render("module", &mod_ctx)?;
            fs::write(modules_dir.join(format!("{}.html", module.name)), module_html)?;
        }

        // Write static assets (CSS/JS)
        fs::write(out_path.join("style.css"), include_str!("static/style.css"))?;
        fs::write(out_path.join("script.js"), include_str!("static/script.js"))?;

        Ok(())
    }
}
