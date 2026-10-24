//! The chat answer (D41): a saved chat window's reply holds one `### <persona-id>` heading and one fenced json block per
//! juror. This reads that shape and nothing else: the block's text goes to the normal verdict checks. Pure.

/// One juror's part of the answer, in the order the headings appear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatBlock {
    pub persona: String,
    /// the json text of the juror's block, or why there is none
    pub body: Result<String, String>,
}

#[derive(Default)]
struct Part {
    persona: String,
    headings: usize,
    jsons: Vec<String>,
    unclosed: bool,
}

/// The heading's persona id: the first word after `###`, with markdown emphasis, backticks and a colon stripped.
fn heading_id(line: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix("###")?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let word = rest.split_whitespace().next()?;
    Some(word.trim_matches(|c| matches!(c, '*' | '`' | ':' | '_')).to_string())
}

/// The blocks of `text` for the given persona ids. Headings that name no persona of the run are ignored (a lens
/// dimension heading, a title); a fence hides everything inside it.
pub fn parse(text: &str, personas: &[String]) -> Vec<ChatBlock> {
    let mut parts: Vec<Part> = Vec::new();
    let mut current: Option<usize> = None;
    // inside a fence: (is a json fence, the text so far)
    let mut fence: Option<(bool, String)> = None;
    for line in text.lines() {
        let t = line.trim_start();
        if let Some((json, buf)) = fence.as_mut() {
            if t.starts_with("```") {
                if let (true, Some(i)) = (*json, current) {
                    parts[i].jsons.push(std::mem::take(buf));
                }
                fence = None;
            } else {
                buf.push_str(line);
                buf.push('\n');
            }
        } else if let Some(info) = t.strip_prefix("```") {
            fence = Some((info.trim().eq_ignore_ascii_case("json"), String::new()));
        } else if let Some(id) = heading_id(line).filter(|id| personas.contains(id)) {
            let i = match parts.iter().position(|p| p.persona == id) {
                Some(i) => i,
                None => {
                    parts.push(Part { persona: id, ..Part::default() });
                    parts.len() - 1
                }
            };
            parts[i].headings += 1;
            current = Some(i);
        }
    }
    if let (Some((true, _)), Some(i)) = (&fence, current) {
        parts[i].unclosed = true;
    }
    parts
        .into_iter()
        .map(|p| {
            let body = if p.headings > 1 {
                Err("answered twice: keep one `### <persona-id>` heading with one json block".to_string())
            } else if p.unclosed {
                Err("the json block is not closed".to_string())
            } else {
                match p.jsons.len() {
                    0 => Err("no ```json block under the heading".to_string()),
                    1 => Ok(p.jsons.into_iter().next().unwrap_or_default()),
                    _ => Err("more than one json block under the heading".to_string()),
                }
            };
            ChatBlock { persona: p.persona, body }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }
    fn body(b: &ChatBlock) -> &str {
        b.body.as_deref().unwrap()
    }

    #[test]
    fn a_heading_and_its_json_block() {
        let t = "Intro\n\n### ana\n\n```json\n{\"persona\": \"ana\"}\n```\n\n### bob\n```json\n{\"persona\": \"bob\"}\n```\n";
        let b = parse(t, &ids(&["ana", "bob"]));
        assert_eq!(b.len(), 2);
        assert_eq!((b[0].persona.as_str(), body(&b[0])), ("ana", "{\"persona\": \"ana\"}\n"));
        assert_eq!(b[1].persona, "bob");
    }

    #[test]
    fn emphasis_backticks_a_colon_and_the_fence_case_are_tolerated() {
        let t = "### `ana`:\n```JSON\n{}\n```\n### **bob**\n```json\n{}\n```\n### cara - the buyer\n```json\n{}\n```\n";
        let b = parse(t, &ids(&["ana", "bob", "cara"]));
        assert_eq!(b.iter().map(|x| x.persona.as_str()).collect::<Vec<_>>(), ["ana", "bob", "cara"]);
        assert!(b.iter().all(|x| x.body.is_ok()));
    }

    #[test]
    fn other_headings_and_fenced_headings_are_not_jurors() {
        let t = "### Summary\n```text\n### ana\n```\n#### ana\n### ana\n```json\n{}\n```\n";
        let b = parse(t, &ids(&["ana"]));
        assert_eq!(b.len(), 1);
        assert_eq!(body(&b[0]), "{}\n");
    }

    #[test]
    fn a_juror_without_a_json_block_or_with_two_or_with_an_open_one_has_no_body() {
        let none = parse("### ana\nfine.\n", &ids(&["ana"]));
        assert!(none[0].body.as_ref().unwrap_err().contains("json"));
        let two = parse("### ana\n```json\n{}\n```\n```json\n{}\n```\n", &ids(&["ana"]));
        assert!(two[0].body.as_ref().unwrap_err().contains("more than one"));
        let open = parse("### ana\n```json\n{\n", &ids(&["ana"]));
        assert!(open[0].body.as_ref().unwrap_err().contains("not closed"));
        let twice = parse("### ana\n```json\n{}\n```\n### ana\n```json\n{}\n```\n", &ids(&["ana"]));
        assert_eq!(twice.len(), 1);
        assert!(twice[0].body.as_ref().unwrap_err().contains("twice"));
    }

    #[test]
    fn a_non_json_fence_under_the_heading_is_skipped() {
        let b = parse("### ana\n```text\nnotes\n```\n```json\n{\"a\":1}\n```\n", &ids(&["ana"]));
        assert_eq!(body(&b[0]), "{\"a\":1}\n");
    }
}
