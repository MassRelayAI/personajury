//! init: the starter `personajury.toml` (D38). A starting point, never an overwrite: it says "no jury" for the code paths
//! and "a jury, surface class" for the screens, pages and docs, and leaves the human classes as commented examples.
use super::{AppError, Result};
use serde_json::{json, Value};
use std::path::Path;

pub const STARTER: &str = r#"# PersonaJury starter config: a starting point, edit it. `personajury should-run --paths a,b` reads it.
# Rules are data; every project writes its own. The first rule a path matches decides nothing alone: the highest quorum
# among the paths of a change wins (none < low < high < human).

# What one persona's brief costs the agent in tokens, by the project's own estimate (the engine never counts tokens).
tokens_per_brief = 15000

# Code paths: the tests and the adversarial read keep these, so no jury runs on them. Add the folders your project has.
[[rule]]
class = "internals"
paths = ["crates/**", "migrations/**", "tools/**", ".github/**", "router/rules/**", "scripts/**", "tests/**"]
quorum = "none"

# The surface class: every screen, page and doc a reader sees gets a jury. Make the panel first:
#   personajury panel add default --query "3 builder, 2 company, 1 others"
[[rule]]
class = "surface"
paths = ["screens/**", "pages/**", "docs/**"]
quorum = "low"
panel = "default"
lenses = ["critical"]
spend = 0

# The human classes: a persona quorum can never decide these. Uncomment the ones your project has.
# [[rule]]
# class = "money"
# paths = ["**/prices/**", "**/pricing/**"]
# quorum = "human"
#
# [[rule]]
# class = "permission"
# paths = ["**/auth/**"]
# quorum = "human"
"#;

/// Write the starter when the project has none. `apply = false` only says what would happen.
pub fn write(root: &Path, apply: bool) -> Result<Value> {
    let path = root.join("personajury.toml");
    if path.exists() {
        return Ok(json!({"path": path.to_string_lossy(), "state": "present", "written": false}));
    }
    if !apply {
        return Ok(json!({"path": path.to_string_lossy(), "state": "planned", "written": false}));
    }
    std::fs::create_dir_all(root).map_err(|e| AppError::Io(root.to_path_buf(), e))?;
    std::fs::write(&path, STARTER).map_err(|e| AppError::Io(path.clone(), e))?;
    Ok(json!({"path": path.to_string_lossy(), "state": "written", "written": true}))
}

/// `personajury init`.
pub fn run(root: &Path) -> Result<Value> {
    write(root, true)
}
