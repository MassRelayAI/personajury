//! init: the starter `personajury.toml` (D38). A starting point, never an overwrite: the human rules (pricing, billing,
//! checkout, legal, terms, privacy, auth, secrets, `.env*`) active, `high` for the other screens, pages and docs, and
//! "no jury" for code, migrations, tools and CI.
use super::{AppError, Result};
use serde_json::{json, Value};
use std::path::Path;

pub const STARTER: &str = r#"# PersonaJury starter config: a starting point, edit it. `personajury should-run --paths a,b` reads it.
# Rules are data; every project writes its own. The order of the rules decides nothing: the highest quorum among the
# paths of a change wins (none < low < high < human), so a human rule below holds even inside a code folder.

# What one persona's brief costs the agent in tokens, by the project's own estimate (the engine never counts tokens).
tokens_per_brief = 15000

# The human classes: a persona quorum can never decide these, so they ship active. Add the paths your project has.
[[rule]]
class = "money"
paths = ["**/pricing*", "**/pricing*/**", "**/prices*", "**/prices*/**", "**/billing*", "**/billing*/**", "**/checkout*", "**/checkout*/**"]
quorum = "human"

[[rule]]
class = "legal"
paths = ["**/legal*", "**/legal*/**", "**/terms*", "**/terms*/**", "**/privacy*", "**/privacy*/**"]
quorum = "human"

[[rule]]
class = "permission"
paths = ["**/auth*", "**/auth*/**"]
quorum = "human"

[[rule]]
class = "secret"
paths = ["**/.env*", "**/secrets*", "**/secrets*/**"]
quorum = "human"

# The surface class: every other screen, page and doc a reader sees gets a jury at the high quorum. Make the panel first:
#   personajury panel add default --query "3 builder, 2 company, 1 others"
[[rule]]
class = "surface"
paths = ["screens/**", "pages/**", "docs/**", "site/**"]
quorum = "high"
panel = "default"
lenses = ["critical"]
spend = 0

# Code, migrations, tools and CI: the tests and the adversarial read keep these, so no jury runs on them. Add the
# folders your project has.
[[rule]]
class = "internals"
paths = ["crates/**", "src/**", "migrations/**", "tools/**", ".github/**", "router/rules/**", "scripts/**", "tests/**"]
quorum = "none"
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
    let out = write(root, true)?;
    // The workspace exists after init, so read verbs (`lens list`) answer at once on an empty one.
    crate::store::Store::open(root)?;
    Ok(out)
}
