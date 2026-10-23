<!-- sources: fixtures/lenses/consulting-deck.toml and investor-pitch.toml (dimensions and anchors); docs/reports/2026-10-24-how-far-can-a-jury-go.md §1–§2 (citations by label; the lenses are fixture data); docs/commands.md (subject add kinds) -->

# Test a deck or a pitch before the meeting

A deck has one job: carry a reader from a question to a decision. A jury reads it the way your audience will, slide by
slide, and says where they got lost.

This playbook has no numbers yet. We have not run a published deck jury. The method is the same as the others.

## 1. Slides as pictures

Export one PNG per slide, plus the PDF and the speaker notes if you have them. Name the files in order: `01-title.png`,
`02-problem.png`, … A juror cites a slide by its file name, so every claim points at a slide.

```bash
jurygraph subject add q3-deck --kind document --attach slides/*.png notes.txt
```

## 2. Personas who will be in the room

Think about who reads it, and how.
- **A client executive** (primary): reads every title, wants the decision on one slide.
- **A CFO** (primary or secondary): reads to the end, checks every number for a source, a date and a base.
- **A board member who skims** (bystander): reads the titles and the last slide, nothing else.

Add others if they matter: a lender, an engineer who must build what the deck promises, a sceptic from another team.
Write each one from real notes when you can: what they asked last time, what made them say no.

## 3. Two lenses that ship as examples

Both are in [`fixtures/lenses/`](../../fixtures/lenses/). Load one with `jurygraph lens add --file`.

**`consulting-deck`** asks four things:
- **storyline:** reading only the titles, can I retell the argument?
- **action titles:** is every title a full sentence that the slide proves?
- **evidence:** does every number name its source, its date and its base?
- **the ask:** do I know what decision is asked of me, its cost, its owner and its date?

**`investor-pitch`** asks four things:
- **problem:** who has it, and what does it cost them today?
- **why now:** what changed, with a fact I can check?
- **proof:** numbers with their period and trend, or users I could call?
- **the ask:** how much, what it buys, which milestone by which date?

Run `critical` too. It asks what stops the reader.

## 4. Read it per group

The skimming board member and the CFO will disagree. That is the point. If the CFO scores *evidence* low and the board
member scores *storyline* high, the story works and the numbers do not. Never average them into one score; the engine
will not.

Then decide each finding as in the [marketing-site playbook](marketing-site.md#5-the-foreman-and-the-owners-five-rules).
Rule 3 matters most here: **fix only what is true today.** Never add a claim to a slide to please a juror.

## What a deck jury will not do

- **Check your numbers.** It judges whether a number *looks* sourced and readable. It does not check the arithmetic. For a
  model, use the `financial-model` example lens, and a person who signs for it.
- **Do due diligence.** The pitch lens judges the pitch as read, not the business.
- **Hear you present.** It reads the slides and the notes. Your voice in the room is not in the files.
