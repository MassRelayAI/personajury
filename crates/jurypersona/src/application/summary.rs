//! summary: the answer display (D24). Gathers what matrix, pick, delta, apply-check and the findings already say and hands
//! it to `domain::summary` to format. It computes no score of its own and pools nothing across strata; it writes a file
//! only when asked (`--html`, `--md`, or `--save` into the toml's `report_dir`).
use super::verdict::{findings_of, load_run, open};
use super::{apply_check, delta, graph, matrix, pick, AppError, Result};
use crate::domain::summary::{self as sm, Facts};
use crate::domain::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const STAKES: [Stake; 3] = [Stake::Primary, Stake::Secondary, Stake::Bystander];

fn stake_of(v: &Value) -> Option<Stake> {
    v.as_str().and_then(Stake::parse)
}
/// A human row's id is `human:<name>:<seat>`; it speaks in the seat's stratum.
fn seat_of(persona: &str) -> &str {
    if persona.starts_with("human:") {
        persona.rsplit(':').next().unwrap_or(persona)
    } else {
        persona
    }
}

fn gather(root: &Path, runs: &[i64]) -> Result<(Facts, Vec<RunRow>)> {
    let store = open(root)?;
    let mut given: Vec<RunRow> = Vec::new();
    for id in runs {
        let r = load_run(&store, *id)?;
        if !given.iter().any(|g| g.id == r.id) {
            given.push(r);
        }
    }
    given.sort_by_key(|r| r.id);
    let mut f = Facts { subject: given[0].subject_name.clone(), runs: given.iter().map(|r| r.id).collect(), ..Default::default() };
    for r in &given {
        if !f.lenses.contains(&r.lens_name) {
            f.lenses.push(r.lens_name.clone());
        }
        for b in &r.briefs {
            f.briefs.push((r.id, b.persona.clone(), r.lens_name.clone(), b.hash.clone()));
        }
    }
    f.date = given.last().map(|r| r.created_at.chars().take(10).collect()).unwrap_or_default();

    // the answer to obey; when it cannot be computed the summary says so
    if let Ok(a) = apply_check::run(root, &f.runs, None, None) {
        f.answer = a["answer"].as_str().map(String::from);
        f.next_quorum = a["next_quorum"].as_str().map(String::from);
        f.reasons = a["reasons"]
            .as_array()
            .map(|v| {
                v.iter()
                    .map(|r| sm::Reason {
                        code: r["code"].as_str().unwrap_or("").into(),
                        effect: r["effect"].as_str().unwrap_or("").into(),
                        data: r.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
    }

    // the numbers: a P·S·B median per (option or subject) × dimension, from the matrix, never combined
    let m = matrix::show(root, &f.runs, false, false, false, Detail::Compact)?;
    let strata = m["strata"].as_array().cloned().unwrap_or_default();
    let mut cols: Vec<(String, String)> = Vec::new(); // (lens, dimension), first seen
    for s in &strata {
        for c in s["cells"].as_array().into_iter().flatten() {
            let k = (c["lens"].as_str().unwrap_or("").to_string(), c["dimension"].as_str().unwrap_or("").to_string());
            if !cols.contains(&k) {
                cols.push(k);
            }
        }
    }
    f.columns = cols
        .iter()
        .map(|(l, d)| if cols.iter().filter(|(_, d2)| d2 == d).count() > 1 { format!("{l}/{d}") } else { d.clone() })
        .collect();
    let mut table: BTreeMap<String, Vec<Option<[Option<i64>; 3]>>> = BTreeMap::new();
    for s in &strata {
        let Some(stake) = stake_of(&s["stake"]) else { continue };
        let si = STAKES.iter().position(|x| *x == stake).unwrap_or(0);
        let mut cells: Vec<sm::CellLine> = Vec::new();
        for c in s["cells"].as_array().into_iter().flatten() {
            let lens = c["lens"].as_str().unwrap_or("").to_string();
            let dim = c["dimension"].as_str().unwrap_or("").to_string();
            let about = c["about"].as_str().map(String::from);
            let label = about.clone().unwrap_or_else(|| f.subject.clone());
            let ci = cols.iter().position(|k| *k == (lens.clone(), dim.clone())).unwrap_or(0);
            let row = table.entry(label.clone()).or_insert_with(|| vec![None; cols.len()]);
            row[ci].get_or_insert([None; 3])[si] = c["median"].as_i64();
            cells.push(sm::CellLine {
                lens,
                dimension: dim,
                about: label,
                n: c["n"].as_i64().unwrap_or(0),
                median: c["median"].as_i64().unwrap_or(0),
                min: c["min"].as_i64().unwrap_or(0),
                max: c["max"].as_i64().unwrap_or(0),
                blockers: c["blockers"].as_i64().unwrap_or(0),
            });
        }
        // the weakest dimension of this stratum: the subject's own lines first, else any line
        let own: Vec<&sm::CellLine> = cells.iter().filter(|c| c.about == f.subject).collect();
        let pool: Vec<&sm::CellLine> = if own.is_empty() { cells.iter().collect() } else { own };
        if let Some(w) = pool.iter().min_by_key(|c| c.median) {
            f.weakest.push((stake, w.dimension.clone(), w.median));
        }
        f.strata.push((stake, cells));
    }
    // best first by the primary stratum's own medians (an ordering only; nothing is shown that is not a median)
    let key = |cells: &Vec<Option<[Option<i64>; 3]>>| -> i64 {
        let v: Vec<i64> = cells.iter().filter_map(|c| c.as_ref().and_then(|t| t[0])).collect();
        if v.is_empty() {
            -1
        } else {
            v.iter().sum::<i64>() * 1000 / v.len() as i64
        }
    };
    let mut rows: Vec<sm::Row> = table.into_iter().map(|(label, cells)| sm::Row { label, cells }).collect();
    rows.sort_by(|a, b| key(&b.cells).cmp(&key(&a.cells)).then_with(|| a.label.cmp(&b.label)));
    f.rows = rows;

    // the picks, per stratum, as counts
    for r in &given {
        let p = pick::show(root, r.id)?;
        for s in p["strata"].as_array().into_iter().flatten() {
            let Some(stake) = stake_of(&s["stake"]) else { continue };
            let choices: Vec<(String, usize)> = s["choices"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|c| (c["choice"].as_str().unwrap_or("").to_string(), c["count"].as_u64().unwrap_or(0) as usize))
                .collect();
            f.picks.push(sm::PickLine { stake, total: choices.iter().map(|c| c.1).sum(), choices });
        }
    }

    // the blockers of the judged (latest per lens) runs, primary first, each with its decision
    let mut judged: Vec<&RunRow> = Vec::new();
    for r in &given {
        match judged.iter_mut().find(|j| j.lens_name == r.lens_name) {
            Some(j) => *j = r,
            None => judged.push(r),
        }
    }
    let mut blockers: Vec<sm::Blocker> = Vec::new();
    for r in &judged {
        let rows = store.verdict_rows(r.id, false)?;
        for fd in findings_of(&r.subject_name, &rows) {
            let stake = r.stakes.get(seat_of(&fd.persona)).copied().unwrap_or(Stake::Bystander);
            let decision = store.latest_decision(&fd.id)?.map(|d| d.outcome);
            blockers.push(sm::Blocker { persona: fd.persona, stake, dimension: fd.dimension, text: fd.text, decision });
        }
    }
    blockers.sort_by_key(|b| b.stake);
    f.blockers = blockers;

    // two rounds of one lens: the blockers before → after, per stratum
    for lens in &f.lenses.clone() {
        let ids: Vec<i64> = given.iter().filter(|r| &r.lens_name == lens).map(|r| r.id).collect();
        if let [.., a, b] = ids[..] {
            if let Ok(d) = delta::show(root, a, b) {
                for stake in STAKES {
                    let c = &d["blockers_by_stratum"][stake.name()];
                    if let (Some(before), Some(after)) = (c["before"].as_u64(), c["after"].as_u64()) {
                        f.changes.push(sm::Change { lens: lens.clone(), from_run: a, stake, before: before as usize, after: after as usize });
                    }
                }
            }
        }
    }
    Ok((f, given))
}

fn report_dir(root: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(root.join("jurypersona.toml")) {
        Ok(t) => Ok(trigger::parse_config(&t)?.report_dir.filter(|d| !d.trim().is_empty())),
        Err(_) => Ok(None),
    }
}
/// A path the project names (`report_dir`, an MCP `md`/`html`): relative, no `..`, and inside the root once the parts that
/// exist are resolved (a symlink out of the root is refused). Returns it joined to the root (I2).
pub(crate) fn inside_root(root: &Path, rel: &Path, what: &str) -> Result<PathBuf> {
    use std::path::Component;
    let bad = |why: &str| AppError::InvalidReportDir(format!("{what} {}: {why}; give a relative path inside the project root", rel.display()));
    if rel.as_os_str().is_empty() {
        return Err(bad("empty"));
    }
    for c in rel.components() {
        match c {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir => return Err(bad("holds `..`")),
            Component::RootDir | Component::Prefix(_) => return Err(bad("is absolute")),
        }
    }
    let base = root.canonicalize().map_err(|e| AppError::Io(root.to_path_buf(), e))?;
    let joined = root.join(rel);
    // the deepest part that exists decides where the rest lands
    let mut probe = joined.clone();
    while !probe.exists() {
        if !probe.pop() {
            break;
        }
    }
    let real = probe.canonicalize().map_err(|e| AppError::Io(probe.clone(), e))?;
    if !real.starts_with(&base) {
        return Err(bad("resolves outside the root"));
    }
    Ok(joined)
}
fn write(p: &Path, s: &str) -> Result<()> {
    if let Some(dir) = p.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| AppError::Io(dir.to_path_buf(), e))?;
    }
    std::fs::write(p, s.as_bytes()).map_err(|e| AppError::Io(p.to_path_buf(), e))
}

/// `confined` (the MCP adapter): `html` and `md` are checked like `report_dir`. A path typed at the CLI may be anywhere.
pub fn show(
    root: &Path,
    runs: &[i64],
    html: Option<&Path>,
    md: Option<&Path>,
    save: bool,
    width: Option<usize>,
    confined: bool,
) -> Result<Value> {
    let width = width.unwrap_or(80);
    if !(30..=200).contains(&width) {
        return Err(AppError::InvalidArguments("--width is 30 to 200 characters".into()));
    }
    if runs.is_empty() {
        return Err(AppError::InvalidArguments("name at least one run".into()));
    }
    let dir = if save {
        let d = report_dir(root)?.ok_or(AppError::NoReportDir)?;
        Some(inside_root(root, Path::new(&d), "report_dir")?)
    } else {
        None
    };
    let target = |p: Option<&Path>, what: &str| -> Result<Option<PathBuf>> {
        match p {
            Some(p) if confined => inside_root(root, p, what).map(Some),
            Some(p) => Ok(Some(p.to_path_buf())),
            None => Ok(None),
        }
    };
    let (html, md) = (target(html, "html")?, target(md, "md")?);
    let (f, _) = gather(root, runs)?;
    let mut out = json!({ "runs": f.runs, "answer": f.answer, "text": sm::text(&f, width) });
    if let Some(p) = html {
        write(&p, &sm::html(&f, &graph::render(root, &f.runs)?))?;
        out["html"] = json!(p);
    }
    if let Some(p) = md {
        write(&p, &sm::markdown(&f))?;
        out["md"] = json!(p);
    }
    if let Some(d) = dir {
        let p = d.join(format!("{}-{}.md", f.date, f.subject));
        write(&p, &sm::markdown(&f))?;
        out["saved"] = json!(p);
    }
    Ok(out)
}
