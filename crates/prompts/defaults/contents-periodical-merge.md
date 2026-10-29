# A detailed table of contents of a newspaper or magazine issue

You receive the articles found on each page of a newspaper or magazine issue, page by page, as JSON. Some articles run over several pages and appear on each of them. Your task is to combine the pages into a detailed table of contents of the whole issue: one entry per article, with every page it is on.

## Instructions

1. **Leave out what is not an article**: covers, pages without articles, advertisements and notices, which the pages describe in `page_note`.
2. **Find the articles that appear on several pages**, from:
   - the title (identical or very close);
   - the notes on where an article continues ("continued on page 5", "to be continued");
   - the same subject, people and wording on consecutive pages, or on the page a note points to;
   - entries titled "Continued: …".
3. **Merge them into one entry per article**:
   - Keep the article's full title, with its section in square brackets before it, as printed; correct obvious reading errors (stray spaces inside a word, a zero for an O).
   - **Pages**: every page the article is on, as numbers and ranges ("1-3", "1, 3, 5"), using the page labels you were given.
   - **Authors and names**: merge the mentions from the different pages without repeating anyone. The same person may be written differently on two pages ("Tiendrébéogo" and "Imam Ismaël Tiendrébéogo"): keep the fullest form once.
   - **Summaries and other text fields**: write one text for the whole article from the partial ones, without repeating yourself.
4. **List the entries in the order the articles begin**, without duplicates and without the pages that hold no article.

## Rules

- Titles that are identical or nearly so (punctuation, capitals): one article.
- A clear continuity of subject on consecutive pages: probably one article, even when the title is not repeated.
- An explicit note ("continued on page X"): one article.
- Use only what the pages give; do not add articles or details.
- Write the fields in {languages}, unless a field says otherwise, and in that one language for every entry: when the pages gave a summary or another text in a different language, write it in this one. Titles and names stay as printed.
