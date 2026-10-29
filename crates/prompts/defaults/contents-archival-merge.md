# An item-by-item inventory of an archival file

You receive the items found on each page of an archival file, page by page, as JSON. Many items run over several pages and appear on each of them. Your task is to combine the pages into a detailed inventory of the file: one entry per item, in the order of the file, with every page it is on, as an archivist describes a file item by item.

## Instructions

1. **Find the items that run over several pages**, from:
   - the notes on where an item continues ("continues on the next page", "continued from the previous page", "2/3");
   - the same title, author, date or reference number;
   - consecutive pages on the same subject with no new heading, letterhead or date line between them.
2. **Merge them into one entry per item**:
   - **Title**: the item's own title or subject line when it has one; otherwise a short descriptive title in square brackets, as the pages give it.
   - **Pages**: every page the item is on, as numbers and ranges ("4-7"), using the page labels you were given.
   - **Dates, authors, recipients**: take them from the page where the item states them (the date line, the signature); do not repeat them.
   - **Summary and other text fields**: one text for the whole item, from the partial ones, without repeating yourself.
3. **Keep separate items separate**: two letters on the same subject, a letter and its reply, a report and its covering letter are distinct entries, even on consecutive pages.
4. **List the entries in the order of the file**, without the blank or illegible pages.

## Rules

- Use only what the pages give; do not add items or details.
- Keep names, places, institutions and reference numbers exactly as the pages spell them, with their spelling of the time.
- Write the fields in {languages}, unless a field says otherwise, and in that one language for every entry: when the pages gave a summary or another text in a different language, write it in this one. Titles and names stay as printed.
