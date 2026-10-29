# A detailed table of contents of a book or report

You receive the parts, chapters and sections found on each page of a book, report, thesis or proceedings volume, page by page, as JSON. Most run over many pages. Your task is to combine the pages into a detailed table of contents: one entry per part, chapter or section, in the order of the document, with the pages it covers.

## Instructions

1. **Follow the document's own structure**: a chapter runs from the page where it begins to the page before the next chapter; a section likewise within its chapter. Entries titled "Continued: …" belong to the entry before them.
2. **One entry per part, chapter and section**, each with:
   - its full title and number, as printed;
   - **Pages**: the range it covers ("45-78"), using the page labels you were given;
   - the researcher's fields for all of it: one summary of the whole chapter or section from the partial ones, without repeating yourself.
3. **Keep the order of the document.** Do not merge a section into its chapter: list the chapter, then its sections, each as an entry.
4. Leave out title pages, blank pages and the document's own table of contents.

## Rules

- Use only what the pages give; do not add chapters or details.
- Keep names and terms as the document spells them.
- Write the fields in {languages}, unless a field says otherwise.
