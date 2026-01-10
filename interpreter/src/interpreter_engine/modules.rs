use crate::interpreter::Interpreter;

impl Interpreter {
    pub fn resolve_import_path(&self, path: &str) -> Result<String, String> {
        // If path contains slashes or backslashes, treat as literal path
        if path.contains('/') || path.contains('\\') {
            return Ok(path.to_string());
        }

        // Otherwise, search for module in standard locations
        let search_paths = vec![
            format!("{}.fenics", path),            // Current dir + .fenics
            format!("libs/{}.fenics", path),       // libs/ subdirectory
            format!("../libs/{}.fenics", path),    // Parent's libs/
            format!("samples/{}.fenics", path),    // samples/ subdirectory
            format!("../samples/{}.fenics", path), // Parent's samples/
        ];

        for candidate in search_paths {
            if std::path::Path::new(&candidate).exists() {
                return Ok(candidate);
            }
        }

        Err(format!(
            "Module '{}' not found in search paths: ./libs/, ../libs/, ./samples/, ../samples/, or current directory",
            path
        ))
    }
}
