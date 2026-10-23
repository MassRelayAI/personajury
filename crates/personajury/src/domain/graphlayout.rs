//! The layout of the jury graph (D11): integer coordinates, columns left to right, rows computed from the content.
//! No force simulation and no float, so the same store always gives the same bytes. Pure, no I/O.

/// The card is always this wide; the height grows with the rows.
pub const WIDTH: i32 = 1200;
pub const MARGIN: i32 = 30;

pub const PERSONA_W: i32 = 172;
pub const PERSONA_H: i32 = 40;
pub const PERSONA_GAP: i32 = 8;
/// the vertical room one stratum column's title takes above its first persona, and the step between two columns:
/// the columns descend like stairs, so no edge from one stratum runs behind another stratum's boxes
pub const TITLE_H: i32 = 58;
pub const STAIR_GAP: i32 = 24;
/// space between two stratum columns
pub const STRATUM_GAP: i32 = 6;

pub const FINDING_W: i32 = 204;
pub const FINDING_H: i32 = 24;
pub const FINDING_GAP: i32 = 4;

pub const DIM_H: i32 = 22;
pub const SUB_H: i32 = 20;
/// how far an `@about` sub-node is indented under its dimension
pub const SUB_INDENT: i32 = 10;
pub const BADGE_W: i32 = 18;
pub const BADGE_H: i32 = 14;
pub const BADGE_GAP: i32 = 2;
/// the three badges of a node and the room around them
pub const BADGES_W: i32 = 3 * BADGE_W + 2 * BADGE_GAP;

/// the least gap between the persona columns and the first lens column, where the edges fan out
pub const FAN_MIN: i32 = 44;
/// gap between the lens columns and the findings
pub const FINDING_FAN: i32 = 24;
pub const LANE_GAP: i32 = 16;
pub const LANE_MIN_W: i32 = 120;

/// One thing in a column: its height and the space above it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Item {
    pub h: i32,
    pub gap_before: i32,
}

/// Top offsets of the items of a column and the column's total height. The first item's gap is not counted.
pub fn flow(items: &[Item]) -> (Vec<i32>, i32) {
    let mut y = 0;
    let mut out = Vec::with_capacity(items.len());
    for (i, it) in items.iter().enumerate() {
        if i > 0 {
            y += it.gap_before;
        }
        out.push(y);
        y += it.h;
    }
    (out, y)
}

/// `n` equal boxes of height `h` with `gap` between them.
pub fn stack(n: usize, h: i32, gap: i32) -> (Vec<i32>, i32) {
    flow(&vec![Item { h, gap_before: gap }; n])
}

/// The offset that centres a column of height `column` in `total`.
pub fn centre(total: i32, column: i32) -> i32 {
    ((total - column) / 2).max(0)
}

/// A cubic edge from (x1, y1) to (x2, y2), horizontal at both ends.
pub fn edge_path(x1: i32, y1: i32, x2: i32, y2: i32) -> String {
    let xm = (x1 + x2) / 2;
    format!("M{x1} {y1}C{xm} {y1} {xm} {y2} {x2} {y2}")
}

/// A conservative width of `s` at `size` px in a system sans (a fallback font may be wider than the first choice).
pub fn text_px(s: &str, size: i32) -> i32 {
    (s.chars().count() as i32 * size * 58 + 99) / 100
}

/// `s` cut with an ellipsis so that it fits `max_px`; a text that fits is returned whole.
pub fn fit(s: &str, max_px: i32, size: i32) -> String {
    if text_px(s, size) <= max_px {
        return s.to_string();
    }
    let per = (size * 58 + 99) / 100;
    let keep = ((max_px / per.max(1)) - 1).max(1) as usize;
    let mut t: String = s.chars().take(keep).collect();
    while t.ends_with(char::is_whitespace) {
        t.pop();
    }
    t.push('\u{2026}');
    t
}

/// x and width of each lens lane, left to right, given the widest label each wants. Lanes take what they need up
/// to the room there is; the room left over widens the fan between the personas and the first lane.
pub fn lanes(wants: &[i32]) -> Vec<(i32, i32)> {
    let personas_right = MARGIN + 3 * PERSONA_W + 2 * STRATUM_GAP;
    let findings_x = WIDTH - MARGIN - FINDING_W;
    let n = wants.len().max(1) as i32;
    let room = findings_x - FINDING_FAN - personas_right - FAN_MIN - LANE_GAP * (n - 1);
    let need: Vec<i32> = wants.iter().map(|w| (w + 10 + BADGES_W + 12).max(LANE_MIN_W)).collect();
    let total: i32 = need.iter().sum();
    let widths: Vec<i32> = if total <= room {
        need
    } else {
        need.iter().map(|w| (w * room / total).max(LANE_MIN_W.min(room / n))).collect()
    };
    let used: i32 = widths.iter().sum::<i32>() + LANE_GAP * (n - 1);
    let mut x = findings_x - FINDING_FAN - used;
    let mut out = Vec::new();
    for w in widths {
        out.push((x, w));
        x += w + LANE_GAP;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stack_of_thirty_never_overlaps() {
        let (ys, total) = stack(30, PERSONA_H, PERSONA_GAP);
        assert_eq!(ys.len(), 30);
        for w in ys.windows(2) {
            assert_eq!(w[1] - w[0], PERSONA_H + PERSONA_GAP);
        }
        assert_eq!(total, 30 * PERSONA_H + 29 * PERSONA_GAP);
        assert_eq!(stack(0, 10, 4), (vec![], 0));
    }

    #[test]
    fn a_flow_counts_the_gap_before_each_item_but_the_first() {
        let (ys, total) = flow(&[
            Item { h: 22, gap_before: 99 },
            Item { h: 20, gap_before: 4 },
            Item { h: 22, gap_before: 10 },
        ]);
        assert_eq!(ys, vec![0, 26, 56]);
        assert_eq!(total, 78);
    }

    #[test]
    fn fit_keeps_what_fits_and_cuts_the_rest_with_an_ellipsis() {
        assert_eq!(fit("works", 200, 11), "works");
        let cut = fit("a very long blocker sentence that has to be cut", 100, 11);
        assert!(cut.ends_with('\u{2026}') && text_px(&cut, 11) <= 100, "{cut}");
        assert!(!fit("x", 1, 11).is_empty());
    }

    #[test]
    fn lanes_stay_between_the_fan_and_the_findings() {
        for wants in [vec![60], vec![60, 100], vec![60, 100, 90, 120], vec![400, 400]] {
            let l = lanes(&wants);
            assert_eq!(l.len(), wants.len());
            let personas_right = MARGIN + 3 * PERSONA_W + 2 * STRATUM_GAP;
            assert!(l[0].0 >= personas_right + FAN_MIN || wants.len() > 2, "{l:?}");
            for w in l.windows(2) {
                assert_eq!(w[1].0, w[0].0 + w[0].1 + LANE_GAP);
            }
            let (x, w) = l[l.len() - 1];
            assert!(x + w <= WIDTH - MARGIN - FINDING_W - FINDING_FAN, "{l:?}");
        }
    }

    #[test]
    fn an_edge_is_horizontal_at_both_ends() {
        assert_eq!(edge_path(10, 20, 110, 80), "M10 20C60 20 60 80 110 80");
    }
}
