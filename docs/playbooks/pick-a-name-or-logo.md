<!-- sources: docs/reports/2026-10-24-logo-jury.md (every logo number); docs/lenses.md (asks_pick, pick refusals); docs/commands.md (pick, subject add) -->

# Pick a name or a logo

When you have a few candidates and must choose one, ask for a **forced pick**. Each persona scores every candidate, then
names one, with a reason. You read the picks per group.

## The story: PersonaJury's own logo

PersonaJury's logo was chosen this way. The full numbers are in
[the logo report](../reports/2026-10-24-logo-jury.md).

**Four directions.** A: twelve seats around one node, a jury box. B: three columns of nodes, the groups. C: a check mark
drawn as nodes, ending in a ring. D: a "J" built from nodes.

**One board per candidate.** Each mark at 16, 32, 128 and 200 pixels, on white and on dark, plus the mark beside the name.
A logo must work in a browser tab, not only on a poster.

**Seven personas** of a developer tool's audience: four who would use it, two who approve it, one passer-by.

**Four questions,** with the scores written in words: can I read it at 16 pixels, does it say what the tool is, is it
its own, does it work everywhere.

**Round 1.** C won 6 of 7 picks: all four users and both approvers. The passer-by chose D. C was best at small sizes and
weakest on looking like nothing else.

**Round 2.** The designer drew two refinements of C, to make it more distinctive. The jury picked the original C, 7 of 7.
Neither refinement was more distinctive, and both were harder to read at 16 pixels. The jury stopped a change that
looked better at the size designers work at.

**One thing to know.** C's legibility score went from 5 to 4 between rounds, and the mark had not changed. Its neighbours
had. Scores are relative to the options in the room. Compare within one run.

## Run a forced pick for a name or a hostname

A name is harder than a logo: people judge it in context. So show each candidate where it will live.

1. **Make one small file per candidate.** For a hostname, a screenshot of a login page with that address in the bar, or
   a short text file: the address, the email it would send from, the line on a business card. Name each file after the
   candidate.
2. **Add them as one subject.** The file names become the options.

   ```bash
   personajury subject add console-name --kind design --attach console.example.com.png app.example.com.png example.app.png
   ```
3. **Write a lens with `asks_pick = true`.** Anchors in words, as always.

   ```toml
   name = "name-pick"
   version = 1
   scale = [1, 5]
   asks_pick = true
   evidence = ["seen", "told", "recalled"]
   standard = ""

   [[dimension]]
   id = "trust"
   title = "I would type my password here"
   anchors = { "1" = "it looks like a phishing page or a test server", "3" = "it looks fine, but I would check the address twice", "5" = "it is plainly the product's own, I would not think twice" }

   [[dimension]]
   id = "say-it"
   title = "I can say it out loud"
   anchors = { "1" = "I would have to spell it", "3" = "I would say it once, then repeat it", "5" = "I say it once and the other person types it right" }
   ```
4. **Brief, answer, read the picks.** `personajury pick <run>` lists each group's choices with their reasons.

The engine refuses a pick with no choice, a choice that is not one of the options, or a choice with no reason.

## What a pick will not tell you

- **Whether the name is free.** Check the domain, the trademark and the package registries yourself.
- **Whether real people agree.** Seven composites tell you where to look. Show the winner to a few real people.
- **The final word.** A name and a logo are *brand*, a human class. The jury advises; a person decides.
