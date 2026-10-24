//! The answer display (D24): sentences first, then a numbers table "for checking", then "Next". Pure: it formats facts
//! the application gathered; it computes no score and never pools strata. No bars anywhere (the display jury of
//! 2026-10-24 picked none): a pick count is a number ("3 of 4"), a change is "15 → 6".
use super::model::Stake;
use serde_json::Value;
use std::fmt::Write as _;

/// One reason of the apply-check answer: its code, its effect and the rest of its data.
#[derive(Debug, Clone)]
pub struct Reason {
    pub code: String,
    pub effect: String,
    pub data: Value,
}
/// A row of the numbers table: one option or subject, one `P·S·B` triple per column (primary, secondary, bystander).
#[derive(Debug, Clone)]
pub struct Row {
    pub label: String,
    pub cells: Vec<Option<[Option<i64>; 3]>>,
}
/// The picks of one stratum in one run: how many jurors chose, and the choices by count.
#[derive(Debug, Clone)]
pub struct PickLine {
    pub stake: Stake,
    pub total: usize,
    pub choices: Vec<(String, usize)>,
}
#[derive(Debug, Clone)]
pub struct Blocker {
    pub persona: String,
    pub stake: Stake,
    pub dimension: String,
    pub text: String,
    /// `fixed | dismissed | row`, none while open
    pub decision: Option<String>,
}
/// Blockers of one stratum in the earlier and the later round of one lens.
#[derive(Debug, Clone)]
pub struct Change {
    pub lens: String,
    pub from_run: i64,
    pub stake: Stake,
    pub before: usize,
    pub after: usize,
}
/// One cell of a stratum's full table (the `--html` file).
#[derive(Debug, Clone)]
pub struct CellLine {
    pub lens: String,
    pub dimension: String,
    pub about: String,
    pub n: i64,
    pub median: i64,
    pub min: i64,
    pub max: i64,
    pub blockers: i64,
}
#[derive(Debug, Clone, Default)]
pub struct Facts {
    pub subject: String,
    pub runs: Vec<i64>,
    pub lenses: Vec<String>,
    pub answer: Option<String>,
    pub next_quorum: Option<String>,
    pub reasons: Vec<Reason>,
    pub columns: Vec<String>,
    /// every row, best first by the primary stratum; the text shows the top `TOP_ROWS`
    pub rows: Vec<Row>,
    pub picks: Vec<PickLine>,
    /// per stratum: the weakest dimension and its median
    pub weakest: Vec<(Stake, String, i64)>,
    pub blockers: Vec<Blocker>,
    pub changes: Vec<Change>,
    pub strata: Vec<(Stake, Vec<CellLine>)>,
    /// the date of the latest run (its `created_at`), so the file is the same on any day
    pub date: String,
    /// (run, persona, lens, brief sha256): what the jurors read
    pub briefs: Vec<(i64, String, String, String)>,
    /// `--slice`: one fixed-template sentence per stratum present (D31), after the other sentences
    pub slice: Vec<String>,
    /// the counted metrics (D34), as fixed-template sentences and footer lines, written by the application from the matrix
    pub agreement: Option<String>,
    pub evidence_line: Option<String>,
    pub shift_line: Option<String>,
    /// `panel: asked 3 builder=primary, seated 3, answered 3; …`
    pub panel_lines: Vec<String>,
    /// per run and per seat when they carry a number, then the total `cost:` line last
    pub cost_lines: Vec<String>,
}
pub const TOP_ROWS: usize = 5;
const SHOWN_BLOCKERS: usize = 3;
const QUOTE: usize = 60;

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}
fn trunc(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    let mut o: String = s.chars().take(n.saturating_sub(1)).collect();
    o.push('…');
    o
}
/// Terminal-safe: ESC sequences (CSI `ESC [ … final`, OSC `ESC ] … BEL|ESC \\`, and any other `ESC x`) are removed whole;
/// every other C0 control but the newline, DEL and the C1 controls are removed; a tab becomes a space. Juror text is
/// model output, so everything the summary prints goes through here (I3).
pub fn safe(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '\u{1b}' => match it.next() {
                Some('[') => {
                    // CSI: parameters and intermediates up to a final byte in @..~
                    for x in it.by_ref() {
                        if ('\u{40}'..='\u{7e}').contains(&x) {
                            break;
                        }
                    }
                }
                Some(']') | Some('P') | Some('X') | Some('^') | Some('_') => {
                    // a string sequence: up to BEL or ESC \\
                    while let Some(x) = it.next() {
                        if x == '\u{7}' {
                            break;
                        }
                        if x == '\u{1b}' {
                            if it.peek() == Some(&'\\') {
                                it.next();
                            }
                            break;
                        }
                    }
                }
                _ => {}
            },
            '\n' => o.push('\n'),
            '\t' => o.push(' '),
            c if (c as u32) < 0x20 || c as u32 == 0x7f || (0x80..0xa0).contains(&(c as u32)) => {}
            c => o.push(c),
        }
    }
    o
}
/// A juror's words for a quote: made safe, runs of whitespace (newlines too) folded to one space, and at most `n`
/// characters, cut only between words and ending with `…`. A first word longer than `n` is kept whole, never cut.
pub fn quote(s: &str, n: usize) -> String {
    let words: Vec<String> = safe(s).split_whitespace().map(String::from).collect();
    let full = words.join(" ");
    if full.chars().count() <= n {
        return full;
    }
    let mut o = String::new();
    for w in &words {
        let len = o.chars().count() + usize::from(!o.is_empty()) + w.chars().count();
        if !o.is_empty() && len + 1 > n {
            break;
        }
        if !o.is_empty() {
            o.push(' ');
        }
        o.push_str(w);
        if len + 1 > n {
            break;
        }
    }
    o.push('…');
    o
}
/// Words wrapped at `width` characters. A word is never cut: one longer than the width stands whole on its own line.
pub fn wrap(s: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for w in s.split_whitespace() {
        if !cur.is_empty() && cur.chars().count() + 1 + w.chars().count() > width {
            lines.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(w);
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

fn count(d: &Value, k: &str) -> usize {
    d[k].as_u64().unwrap_or(0) as usize
}
/// The reasons in plain words, each with its effect (`human`, `escalate`, `advise`); informational ones are left out.
fn reason_words(f: &Facts) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut splits = 0usize;
    let mut open = 0usize;
    let mut seats = 0usize;
    let mut blockers = 0usize;
    for r in f.reasons.iter().filter(|r| r.effect != "info") {
        let d = &r.data;
        let w = match r.code.as_str() {
            "primary_blocker" => {
                blockers += count(d, "blockers");
                continue;
            }
            "split" => {
                splits += 1;
                continue;
            }
            "check_failed" => {
                open += d["unresolved"].as_array().map_or(0, |a| a.len());
                continue;
            }
            "incomplete_jury" => {
                seats += d["missing"].as_array().map_or(0, |a| a.iter().map(|m| m["seats"].as_array().map_or(0, |s| s.len())).sum());
                continue;
            }
            "human_blocker" => "a person raised a blocker that is still open".to_string(),
            "delta_not_landed" => "the fixes from the last round did not all land".to_string(),
            "human_class" => format!("the change touches {}, which a person decides", d["class"].as_str().unwrap_or("a human class")),
            "quorum_unknown" => "no rule was given, so the jury can only advise".to_string(),
            "rule_below_sources" => "the rule given asks less than the changed files need".to_string(),
            "overridden_down" => "the jury was smaller or cheaper than its rule asks".to_string(),
            "effort_below_quorum" => "the jury ran cheaper than the risk asks".to_string(),
            "independence_sequential" => "one context played every juror".to_string(),
            "stale_run" => "the subject changed after the jury ran".to_string(),
            "delta_unavailable" => "the last two rounds cannot be compared".to_string(),
            "deliberated" => "the jurors saw each other's verdicts in a second round, so only their first word was independent".to_string(),
            "uncalibrated" => "the jury does not rank like the people it was checked against".to_string(),
            other => other.replace('_', " "),
        };
        if !out.iter().any(|(_, x)| *x == w) {
            out.push((r.effect.clone(), w));
        }
    }
    let mut counted: Vec<(String, String)> = Vec::new();
    if blockers > 0 {
        counted.push(("escalate".into(), format!("primary jurors raised {}", plural(blockers, "blocker", "blockers"))));
    }
    if splits > 0 {
        counted.push(("escalate".into(), format!("the groups disagree on {}", plural(splits, "score", "scores"))));
    }
    if open > 0 {
        counted.push(("escalate".into(), format!("{} no decision yet", plural(open, "finding has", "findings have"))));
    }
    if seats > 0 {
        counted.push(("advise".into(), format!("{} no verdict yet", plural(seats, "seat has", "seats have"))));
    }
    // the counts first within their effect, then the rest, most serious effect first
    counted.extend(out);
    let rank = |e: &str| match e {
        "human" => 0u8,
        "escalate" => 1,
        _ => 2,
    };
    counted.sort_by_key(|(e, _)| rank(e));
    counted
}
/// `Words` with its first letter upper case.
fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
}
fn list(items: &[String], max: usize) -> String {
    let mut shown: Vec<String> = items.iter().take(max).cloned().collect();
    if items.len() > max {
        shown.push(format!("{} more", items.len() - max));
    }
    match shown.len() {
        0 => String::new(),
        1 => shown[0].clone(),
        2 => format!("{} and {}", shown[0], shown[1]),
        n => format!("{}, and {}", shown[..n - 1].join(", "), shown[n - 1]),
    }
}

/// The plain sentences, in the order the display jury asked for: the answer, who won, the weak spot, the blockers.
pub fn sentences(f: &Facts) -> Vec<String> {
    let mut s = Vec::new();
    // the answer in plain words: the reasons that make it, then (in a second sentence) what holds below it
    let words = reason_words(f);
    let effect = match f.answer.as_deref() {
        Some("human") => "human",
        Some("escalate") => "escalate",
        _ => "advise",
    };
    let mine: Vec<String> = words.iter().filter(|(e, _)| e == effect).map(|(_, w)| w.clone()).collect();
    let below: Vec<String> = words
        .iter()
        .filter(|(e, _)| e == "advise" && effect != "advise")
        .map(|(_, w)| w.strip_suffix(", so the jury can only advise").unwrap_or(w).to_string())
        .collect();
    let why = cap(&list(&mine, 5));
    s.push(match f.answer.as_deref() {
        Some("apply") => "The jury says: apply. Every condition is proven.".to_string(),
        Some("escalate") => format!("The jury says: escalate. {why}."),
        Some("human") => format!("The jury says: a person decides. {why}."),
        Some("advise") => format!("The jury says: advise. {why}."),
        _ => format!("No apply-check answer could be computed for {}.", f.subject),
    });
    if !below.is_empty() {
        s.push(format!("Even without that, the jury could only advise: {}.", list(&below, 3)));
    }
    // who won: the pick when a lens asked for one, else the best row by the primary stratum
    if !f.picks.is_empty() {
        let mut parts = Vec::new();
        for p in &f.picks {
            if let Some((c, n)) = p.choices.first() {
                let tied = p.choices.get(1).is_some_and(|(_, m)| m == n);
                parts.push(if tied {
                    format!("{} was split, {} of {} each for the top choices", p.stake.name(), n, p.total)
                } else {
                    format!("{} chose {c}, {n} of {}", p.stake.name(), p.total)
                });
            }
        }
        let mut t = format!("On the pick, {}.", parts.join("; "));
        if let Some(p) = f.picks.iter().find(|p| p.stake == Stake::Primary) {
            if let (Some(_), Some((c, n))) = (p.choices.first(), p.choices.get(1)) {
                if p.choices[0].1 != *n {
                    let _ = write!(t, " The runner-up with primary was {c}, {n} of {}.", p.total);
                }
            }
        }
        s.push(t);
    } else if f.rows.len() > 1 {
        s.push(format!("By the primary medians {} leads, then {}.", f.rows[0].label, f.rows[1].label));
    }
    if !f.weakest.is_empty() {
        let w: Vec<String> = f.weakest.iter().map(|(k, d, m)| format!("{d} for {} (median {m})", k.name())).collect();
        s.push(format!("The weakest dimension is {}.", list(&w, 3)));
    }
    if !f.changes.is_empty() {
        let c: Vec<String> = f.changes.iter().map(|c| format!("{} {} → {}", c.stake.name(), c.before, c.after)).collect();
        let (lens, from) = (&f.changes[0].lens, f.changes[0].from_run);
        s.push(format!("Since run {from} ({lens}) the blockers went {}.", list(&c, 3)));
    }
    if !f.blockers.is_empty() {
        let shown: Vec<String> = rotate(&f.blockers)
            .into_iter()
            .take(SHOWN_BLOCKERS)
            .map(|b| {
                let dec = b.decision.as_deref().map_or("no decision yet".to_string(), |d| format!("decided: {d}"));
                format!("{} ({}) on {}: \"{}\" ({dec})", b.persona, b.stake.name(), b.dimension, quote(&b.text, QUOTE))
            })
            .collect();
        let more = f.blockers.len().saturating_sub(SHOWN_BLOCKERS);
        let tail = if more > 0 { format!(" and {more} more") } else { String::new() };
        s.push(format!("Blockers, in the jurors' words: {}{tail}.", shown.join("; ")));
    }
    s.extend(f.slice.iter().cloned());
    s.extend(f.agreement.iter().cloned());
    s.extend(f.evidence_line.iter().cloned());
    s.extend(f.shift_line.iter().cloned());
    s
}
/// The blockers in turns across jurors: primary first, then secondary, then bystander; each juror's first blocker before
/// anyone's second, so the few shown speak for as many jurors as they can.
fn rotate(blockers: &[Blocker]) -> Vec<&Blocker> {
    let mut jurors: Vec<(Stake, &str)> = Vec::new();
    for b in blockers {
        if !jurors.iter().any(|(_, p)| *p == b.persona) {
            jurors.push((b.stake, &b.persona));
        }
    }
    jurors.sort_by_key(|(k, _)| *k);
    let mut out: Vec<&Blocker> = Vec::new();
    let mut turn = 0usize;
    while out.len() < blockers.len() {
        for (_, p) in &jurors {
            if let Some(b) = blockers.iter().filter(|b| b.persona == *p).nth(turn) {
                out.push(b);
            }
        }
        turn += 1;
    }
    out
}
/// The fixed sentence chosen from the answer; never generated prose.
pub fn next(f: &Facts) -> String {
    match (f.answer.as_deref(), f.next_quorum.as_deref()) {
        (Some("apply"), _) => "apply the change and record each decision before you merge.".into(),
        (Some("escalate"), Some("human")) => "a person decides; the jury only advises from here.".into(),
        (Some("escalate"), Some(q)) => format!("run the {q} quorum with these reasons in hand before applying anything."),
        (Some("escalate"), None) => "run the next quorum up with these reasons in hand.".into(),
        (Some("human"), _) => "a person decides; show them this summary.".into(),
        (Some("advise"), _) => "treat this as advice: fix what the reasons name, then ask apply-check again.".into(),
        _ => "run apply-check and read its reasons before anything is applied.".into(),
    }
}

fn triple(c: &Option<[Option<i64>; 3]>) -> String {
    match c {
        None => "-".to_string(),
        Some(t) => t.iter().map(|m| m.map_or("-".to_string(), |v| v.to_string())).collect::<Vec<_>>().join("·"),
    }
}
/// The numbers table, in column chunks that fit `width`.
#[allow(clippy::needless_range_loop)]
fn table(f: &Facts, width: usize) -> Vec<String> {
    let rows: Vec<&Row> = f.rows.iter().take(TOP_ROWS).collect();
    if rows.is_empty() || f.columns.is_empty() {
        return vec![];
    }
    let colw: Vec<usize> = f
        .columns
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let cells = rows.iter().map(|r| triple(&r.cells[i]).chars().count()).max().unwrap_or(1);
            c.chars().count().min(12).max(cells)
        })
        .collect();
    let widest = colw.iter().copied().max().unwrap_or(5);
    let label_w = rows.iter().map(|r| r.label.chars().count()).max().unwrap_or(5).min(width.saturating_sub(2 + widest).clamp(4, 24));
    let mut out = Vec::new();
    let mut i = 0;
    while i < f.columns.len() {
        let mut j = i;
        let mut used = label_w;
        while j < f.columns.len() && (j == i || used + 2 + colw[j] <= width) {
            used += 2 + colw[j];
            j += 1;
        }
        let mut head = format!("{:<label_w$}", "");
        for k in i..j {
            let _ = write!(head, "  {:<w$}", trunc(&f.columns[k], colw[k]), w = colw[k]);
        }
        out.push(head.trim_end().to_string());
        for r in &rows {
            let mut l = format!("{:<label_w$}", trunc(&r.label, label_w));
            for k in i..j {
                let _ = write!(l, "  {:<w$}", triple(&r.cells[k]), w = colw[k]);
            }
            out.push(l.trim_end().to_string());
        }
        i = j;
        if i < f.columns.len() {
            out.push(String::new());
        }
    }
    if f.rows.len() > TOP_ROWS {
        out.push(format!("({} more not shown)", f.rows.len() - TOP_ROWS));
    }
    out
}

/// The chat text: a title, the sentences, the table "for checking", then "Next:". Under about thirty lines.
pub fn text(f: &Facts, width: usize) -> String {
    let mut out: Vec<String> = Vec::new();
    let runs: Vec<String> = f.runs.iter().map(|r| r.to_string()).collect();
    out.extend(wrap(&format!("PersonaJury summary: {}, runs {} ({})", f.subject, runs.join(" "), f.lenses.join(", ")), width));
    out.push(String::new());
    for s in sentences(f) {
        out.extend(wrap(&s, width));
    }
    let t = table(f, width);
    if !t.is_empty() {
        out.push(String::new());
        out.extend(wrap("Numbers, for checking (medians primary·secondary·bystander; - = nobody):", width));
        out.extend(t);
        for l in &f.panel_lines {
            out.extend(wrap(l, width));
        }
    }
    out.push(String::new());
    out.extend(wrap(&format!("Next: {}", next(f)), width));
    for l in &f.cost_lines {
        out.extend(wrap(l, width));
    }
    safe(&(out.join("\n") + "\n"))
}

// ---- the --html file: one self-contained page, inline CSS, the graph embedded, no scripts ----

fn esc(s: &str) -> String {
    safe(s)
        .chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            c => c.to_string(),
        })
        .collect()
}
const CSS: &str = "body{font:16px/1.5 system-ui,-apple-system,'Segoe UI',Helvetica,Arial,sans-serif;color:#1f2328;background:#fff;margin:0}\
main{max-width:960px;margin:0 auto;padding:24px 16px}h1{font-size:22px;margin:0 0 4px}h2{font-size:17px;margin:28px 0 8px;border-bottom:1px solid #d0d7de;padding-bottom:4px}\
p{margin:8px 0}.muted{color:#57606a}table{border-collapse:collapse;margin:8px 0;font-size:14px}th,td{border:1px solid #d0d7de;padding:4px 10px;text-align:left}\
th{background:#f6f8fa}td.n{font-variant-numeric:tabular-nums;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}\
.primary{background:#ddf4ff}.secondary{background:#fff8c5}.bystander{background:#dafbe1}.next{border-left:4px solid #0550ae;padding:6px 12px;background:#f6f8fa}\
.graph{overflow-x:auto;border:1px solid #d0d7de;border-radius:8px}.graph svg{max-width:100%;height:auto;display:block}\
blockquote{margin:6px 0;padding:2px 12px;border-left:3px solid #cf222e;color:#1f2328}";

/// The page. `svg` is the `graph --svg` output of the same runs.
pub fn html(f: &Facts, svg: &str) -> String {
    let mut o = String::new();
    let runs: Vec<String> = f.runs.iter().map(|r| r.to_string()).collect();
    let _ = write!(
        o,
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\n<title>PersonaJury summary: {}</title>\n<style>{CSS}</style></head><body><main>\n",
        esc(&f.subject)
    );
    let _ = writeln!(o, "<h1>{}</h1>\n<p class=\"muted\">runs {} · lenses {}</p>", esc(&f.subject), runs.join(", "), esc(&f.lenses.join(", ")));
    for s in sentences(f) {
        let _ = writeln!(o, "<p>{}</p>", esc(&s));
    }
    let _ = writeln!(o, "<h2>Numbers, for checking</h2>\n<p class=\"muted\">Medians as primary·secondary·bystander; a dash is a stratum nobody judged.</p>");
    if !f.rows.is_empty() {
        o.push_str("<table><tr><th></th>");
        for c in &f.columns {
            let _ = write!(o, "<th>{}</th>", esc(c));
        }
        o.push_str("</tr>\n");
        for r in &f.rows {
            let _ = write!(o, "<tr><th>{}</th>", esc(&r.label));
            for c in &r.cells {
                let _ = write!(o, "<td class=\"n\">{}</td>", triple(c));
            }
            o.push_str("</tr>\n");
        }
        o.push_str("</table>\n");
    }
    if !f.picks.is_empty() {
        o.push_str("<h2>Picks</h2>\n<table><tr><th>Stratum</th><th>Choice</th><th>Jurors</th></tr>\n");
        for p in &f.picks {
            for (c, n) in &p.choices {
                let _ = writeln!(o, "<tr class=\"{}\"><td>{}</td><td>{}</td><td class=\"n\">{n} of {}</td></tr>", p.stake.name(), p.stake.name(), esc(c), p.total);
            }
        }
        o.push_str("</table>\n");
    }
    let start = svg.find("<svg").unwrap_or(0);
    let _ = writeln!(o, "<h2>The jury</h2>\n<div class=\"graph\">{}</div>", &svg[start..]);
    o.push_str("<h2>Every cell, per stratum</h2>\n");
    for (stake, cells) in &f.strata {
        let _ = writeln!(
            o,
            "<h3 class=\"{0}\">{0}</h3>\n<table><tr><th>lens</th><th>dimension</th><th>about</th><th>jurors</th><th>median</th><th>min</th><th>max</th><th>blockers</th></tr>",
            stake.name()
        );
        for c in cells {
            let _ = writeln!(
                o,
                "<tr><td>{}</td><td>{}</td><td>{}</td><td class=\"n\">{}</td><td class=\"n\">{}</td><td class=\"n\">{}</td><td class=\"n\">{}</td><td class=\"n\">{}</td></tr>",
                esc(&c.lens), esc(&c.dimension), esc(&c.about), c.n, c.median, c.min, c.max, c.blockers
            );
        }
        o.push_str("</table>\n");
    }
    if !f.blockers.is_empty() {
        o.push_str("<h2>Blockers, in the jurors' words</h2>\n");
        for b in &f.blockers {
            let dec = b.decision.as_deref().map_or("no decision yet".to_string(), |d| format!("decided: {d}"));
            let _ = writeln!(o, "<blockquote>{} ({}) on {}: “{}” <span class=\"muted\">({dec})</span></blockquote>", esc(&b.persona), b.stake.name(), esc(&b.dimension), esc(&b.text));
        }
    }
    let _ = writeln!(o, "<h2>Next</h2>\n<p class=\"next\">{}</p>", esc(&next(f)));
    for l in f.panel_lines.iter().chain(&f.cost_lines) {
        let _ = writeln!(o, "<p class=\"muted\">{}</p>", esc(l));
    }
    o.push_str("</main></body></html>\n");
    o
}

// ---- the --md file: the same human summary as markdown, for a report folder ----

fn md_cell(s: &str) -> String {
    safe(s).split_whitespace().collect::<Vec<_>>().join(" ").replace('|', "\\|")
}
pub fn markdown(f: &Facts) -> String {
    let mut o = String::new();
    let runs: Vec<String> = f.runs.iter().map(|r| r.to_string()).collect();
    let _ = writeln!(o, "# PersonaJury summary: {}\n", f.subject);
    let _ = writeln!(o, "Date: {}\nRuns: {} ({})\n", f.date, runs.join(", "), f.lenses.join(", "));
    for s in sentences(f) {
        let _ = writeln!(o, "{s}\n");
    }
    if !f.rows.is_empty() {
        let _ = writeln!(o, "## Numbers, for checking\n\nMedians as primary·secondary·bystander; a dash is a stratum nobody judged.\n");
        let _ = write!(o, "| |");
        for c in &f.columns {
            let _ = write!(o, " {} |", md_cell(c));
        }
        let _ = write!(o, "\n|---|");
        for _ in &f.columns {
            o.push_str("---|");
        }
        o.push('\n');
        for r in f.rows.iter().take(TOP_ROWS) {
            let _ = write!(o, "| {} |", md_cell(&r.label));
            for c in &r.cells {
                let _ = write!(o, " {} |", triple(c));
            }
            o.push('\n');
        }
        if f.rows.len() > TOP_ROWS {
            let _ = writeln!(o, "\n({} more not shown)", f.rows.len() - TOP_ROWS);
        }
        o.push('\n');
    }
    if !f.blockers.is_empty() {
        let _ = writeln!(o, "## Decisions per finding\n\n| persona | stratum | dimension | blocker | decision |\n|---|---|---|---|---|");
        for b in &f.blockers {
            let _ = writeln!(
                o,
                "| {} | {} | {} | {} | {} |",
                md_cell(&b.persona),
                b.stake.name(),
                md_cell(&b.dimension),
                md_cell(&b.text),
                b.decision.as_deref().unwrap_or("none yet")
            );
        }
        o.push('\n');
    }
    if !f.briefs.is_empty() {
        let _ = writeln!(o, "## Runs and brief hashes\n\n| run | persona | lens | brief sha256 |\n|---|---|---|---|");
        for (r, p, l, h) in &f.briefs {
            let _ = writeln!(o, "| {r} | {} | {} | {h} |", md_cell(p), md_cell(l));
        }
        o.push('\n');
    }
    let _ = writeln!(o, "Next: {}", next(f));
    if !f.panel_lines.is_empty() || !f.cost_lines.is_empty() {
        o.push('\n');
        for l in f.panel_lines.iter().chain(&f.cost_lines) {
            let _ = writeln!(o, "{l}\n");
        }
    }
    safe(&o)
}
