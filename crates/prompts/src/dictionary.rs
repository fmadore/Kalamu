//! The prompt a project's dictionary sends when the researcher asks a model to propose merges
//! (#48). Prompts lists it, to be read, edited and reset like any default, but New job never
//! offers it: it reads the dictionary's own rows, not a document. Its answer's fields are fixed,
//! since the dictionary reads them. iwac-ai-pipelines' authority curation has a model review candidate pairs with the
//! documents naming them; its rules (a spelling variant is not proof, a curator approves every
//! merge) are kept, and the rows come with the titles of documents that name them.

use kalamu_core::Task;

use crate::defaults::Default;

/// The preset's id, by which the dictionary asks for it.
pub const DICTIONARY_MERGES: &str = "dictionary.merges";

const INSTRUCTIONS: &str = "You help a researcher clean the dictionary of a research project. \
AI analyses of the project's documents listed names or terms in one field (people, places, \
organisations, themes). Each row below is one name or term as the analyses wrote it, with its \
spellings and the documents that name it. The researcher will check every group you propose; \
nothing is merged without them.

## Task

Find the rows that are the same person, place, organisation or term written differently, and \
propose each such set as a group.

## Rules

- Group rows when they are one: spelling variants, accents, transliterations from another \
script, abbreviations and initials, titles and honorifics (El Hadj, Dr, Imam), word order, \
reading errors in the source, and the same name in another language.
- Do not group rows that are only related: a person and their family, a town and its region, \
an organisation and its branch, a broader and a narrower theme, two themes often found together.
- A shared surname or a short form (\"Traoré\", \"the Imam\") is not enough: group it with a \
fuller name only when the documents make it clear, and say so.
- When the forms and documents do not settle it, leave the rows apart rather than guess; a \
group you are unsure of is \"possible\", never \"certain\".
- Never group rows the researcher kept apart: they are listed after the rows.
- Use only the references given (R1, R2). A row is in one group at most; a group has at least \
two rows.
- The preferred label is the fullest correct form among the rows, in the spelling the \
documents use most.
- When there is no group to propose, return no group.";

const REQUEST: &str = "Propose groups among the rows above, following the instructions given.";

pub(crate) const DICTIONARY: Default = Default {
    id: DICTIONARY_MERGES,
    task: Task::Analysis,
    name: "Dictionary: proposed merges",
    description: "Sent when a project's dictionary asks a model to propose merges. Its \
                  answer's fields are fixed: the dictionary reads them.",
    instructions: INSTRUCTIONS,
    user_prompt: REQUEST,
    schema: Some(kalamu_core::dictionary::proposal::schema),
    merge: None,
    dense: false,
    made_for: None,
};
