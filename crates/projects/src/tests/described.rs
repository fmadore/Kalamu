//! Descriptions of sources: a document's own parts beside its project's, never touched by a
//! result, written out with where each came from; and tags renamed across a project.

use kalamu_core::{DescriptionField as F, ProjectStore, SourceDescription, SourceDocuments};

use super::*;

fn description(parts: &[(F, &str)]) -> SourceDescription {
    parts.iter().map(|(f, v)| (*f, (*v).to_string())).collect()
}

#[test]
fn a_document_takes_the_projects_parts_it_leaves_out() {
    let f = fixture();
    let p = f.projects.create("Archives", "").unwrap();
    let docs = f
        .projects
        .add_documents(p.id, vec![f.file("a.pdf", "a"), f.file("b.pdf", "b")], None)
        .unwrap();
    f.projects
        .set_source_defaults(
            p.id,
            description(&[
                (F::Repository, "  Archives nationales du Sénégal "),
                (F::Fonds, "AOF"),
                (F::Rights, "   "),
            ]),
        )
        .unwrap();
    f.projects
        .describe_document(
            p.id,
            docs[0],
            description(&[
                (F::Fonds, "Fonds moderne"),
                (F::Shelfmark, "19G 1"),
                (F::Citation, "ANS, 19G 1.  \n  Lettre du cercle.\n"),
            ]),
        )
        .unwrap();

    let first = f.projects.document(p.id, docs[0]).unwrap();
    let parts: Vec<(F, &str, DescribedBy)> = first
        .described
        .iter()
        .map(|d| (d.field, d.value.as_str(), d.by))
        .collect();
    assert_eq!(
        parts,
        [
            (
                F::Repository,
                "Archives nationales du Sénégal",
                DescribedBy::Project
            ),
            (F::Fonds, "Fonds moderne", DescribedBy::Document),
            (F::Shelfmark, "19G 1", DescribedBy::Document),
            // Line breaks inside are kept, spaces at their ends are not.
            (
                F::Citation,
                "ANS, 19G 1.\n  Lettre du cercle.",
                DescribedBy::Document
            ),
        ]
    );
    // An empty part is not kept: it takes the project's.
    let second = f.projects.document(p.id, docs[1]).unwrap();
    assert_eq!(second.described.len(), 2);
    assert!(
        second
            .described
            .iter()
            .all(|d| d.by == DescribedBy::Project)
    );

    // Kept through a restart of the store, as written.
    let stored = f.store.document(docs[0]).unwrap();
    assert_eq!(stored.source_description.len(), 3);
    assert_eq!(f.store.project(p.id).unwrap().source_defaults.len(), 2);

    // Another document of another project cannot be described from this one.
    let other = f.projects.create("Other", "").unwrap();
    assert!(matches!(
        f.projects
            .describe_document(other.id, docs[0], SourceDescription::new()),
        Err(ProjectError::Elsewhere(_))
    ));
}

#[test]
fn descriptions_are_written_out_with_where_each_part_came_from() {
    let f = fixture();
    let p = f.projects.create("Archives", "").unwrap();
    let docs = f
        .projects
        .add_documents(p.id, vec![f.file("a.pdf", "a"), f.file("b.pdf", "b")], None)
        .unwrap();
    let letters = f.projects.create_collection(p.id, "Lettres").unwrap();
    f.projects
        .add_to_collection(p.id, letters.id, &docs[..1])
        .unwrap();
    f.projects.tag_documents(p.id, &docs[..1], "Dakar").unwrap();
    f.projects
        .tag_documents(p.id, &docs[..1], "Islam, 1957")
        .unwrap();
    f.projects
        .set_source_defaults(p.id, description(&[(F::Repository, "ANS")]))
        .unwrap();
    f.projects
        .describe_document(
            p.id,
            docs[0],
            description(&[(F::Citation, "« Lettre », 1957,\nf. 2")]),
        )
        .unwrap();

    let csv = csv_body(f.projects.descriptions_table(p.id, None).unwrap());
    let mut rows = csv.lines();
    assert_eq!(
        rows.next().unwrap(),
        "document_id,title,file_name,path,collections,tags,repository,repository_from,fonds,\
         fonds_from,series,series_from,file,file_from,shelfmark,shelfmark_from,creator,\
         creator_from,date,date_from,url,url_from,citation,citation_from,rights,rights_from"
    );
    let path = f.store.document(docs[0]).unwrap().source.path;
    let expected = format!(
        "{},a,a.pdf,{path},Lettres,\"Dakar; Islam, 1957\",ANS,project,,,,,,,,,,,,,,,\
         \"« Lettre », 1957,\nf. 2\",document,,",
        docs[0]
    );
    assert!(csv.contains(&expected), "{csv}");

    // A collection keeps to its documents.
    let one = csv_body(
        f.projects
            .descriptions_table(p.id, Some(letters.id))
            .unwrap(),
    );
    assert!(!one.contains("b.pdf"));

    let json: serde_json::Value =
        serde_json::from_str(&f.projects.export_descriptions_json(p.id, None).unwrap()).unwrap();
    assert_eq!(json["format"], "kalamu-descriptions");
    let first = &json["documents"][0];
    assert_eq!(first["collections"][0], "Lettres");
    assert_eq!(first["description"]["repository"]["from"], "project");
    assert_eq!(
        first["description"]["citation"]["value"],
        "« Lettre », 1957,\nf. 2"
    );
    assert!(first["description"].get("fonds").is_none());
}

#[test]
fn a_tag_is_renamed_across_the_project_and_merges_with_one_in_use() {
    let f = fixture();
    let p = f.projects.create("Archives", "").unwrap();
    let docs = f
        .projects
        .add_documents(
            p.id,
            vec![
                f.file("a.pdf", "a"),
                f.file("b.pdf", "b"),
                f.file("c.pdf", "c"),
            ],
            None,
        )
        .unwrap();
    f.projects
        .tag_documents(p.id, &docs[..2], "pelerinage")
        .unwrap();
    f.projects.tag_documents(p.id, &docs[2..], "Hajj").unwrap();

    // Capitals only: respelled.
    assert_eq!(
        f.projects
            .rename_tag(p.id, "PELERINAGE", "Pèlerinage")
            .unwrap()
            .changed,
        2
    );
    let tags = |f: &Fixture| -> Vec<(String, u32)> {
        f.projects
            .view(p.id)
            .unwrap()
            .tags
            .into_iter()
            .map(|t| (t.name, t.documents))
            .collect()
    };
    assert_eq!(
        tags(&f),
        [("Hajj".to_string(), 1), ("Pèlerinage".to_string(), 2)]
    );
    // Onto a tag in use, in other capitals: one tag, the project's spelling.
    f.projects.tag_documents(p.id, &docs[..1], "Hajj").unwrap();
    let merged = f
        .projects
        .rename_tag(p.id, "pèlerinage", "  hajj ")
        .unwrap();
    assert_eq!(
        (merged.changed, merged.name.as_str()),
        (2, "Hajj"),
        "the name the documents now carry, for the table to filter on"
    );
    assert_eq!(tags(&f), [("Hajj".to_string(), 3)]);
    assert_eq!(f.store.document(docs[0]).unwrap().tags, ["Hajj"]);

    assert_eq!(
        f.projects
            .rename_tag(p.id, "absent", "Other")
            .unwrap()
            .changed,
        0
    );
    assert!(matches!(
        f.projects.rename_tag(p.id, "Hajj", "  "),
        Err(ProjectError::NoTag)
    ));
}

#[test]
fn a_results_provenance_names_each_document_its_source_is() {
    let f = fixture();
    let p = f.projects.create("Archives", "").unwrap();
    let docs = f
        .projects
        .add_documents(p.id, vec![f.file("a.pdf", "a")], None)
        .unwrap();
    f.projects
        .set_source_defaults(p.id, description(&[(F::Repository, "ANS")]))
        .unwrap();
    f.projects
        .describe_document(p.id, docs[0], description(&[(F::Shelfmark, "19G 1")]))
        .unwrap();
    f.projects.rename_document(p.id, docs[0], "Lettre").unwrap();
    let source = f.store.documents(p.id).unwrap()[0].source.clone();
    let b = f.file("b.pdf", "b");
    let job = f.job(Task::TextExtraction, vec![source.clone(), b.clone()]);
    // Not yet tied to the project: nothing to name.
    assert!(f.projects.documents(job.id, source.id).unwrap().is_empty());

    f.projects.link_job(p.id, job.id, None).unwrap();
    let named = f.projects.documents(job.id, source.id).unwrap();
    assert_eq!(named.len(), 1);
    assert_eq!(named[0].document_id, docs[0]);
    assert_eq!(named[0].project_name, "Archives");
    assert_eq!(named[0].title, "Lettre");
    let parts: Vec<(F, &str, DescribedBy)> = named[0]
        .description
        .iter()
        .map(|(field, part)| (*field, part.value.as_str(), part.from))
        .collect();
    assert_eq!(
        parts,
        [
            (F::Repository, "ANS", DescribedBy::Project),
            (F::Shelfmark, "19G 1", DescribedBy::Document),
        ]
    );
    // The other source became a document of its own, described by the project alone.
    let other = f.projects.documents(job.id, b.id).unwrap();
    assert_eq!(other.len(), 1);
    assert_eq!(other[0].description.len(), 1);

    // The results to write again when a document changes, or when the project's defaults do.
    assert_eq!(
        f.projects.sources_of(p.id, Some(&[docs[0]])).unwrap(),
        [(job.id, source.id)]
    );
    assert_eq!(f.projects.sources_of(p.id, None).unwrap().len(), 2);

    // In a second project too, each with its own defaults.
    let q = f.projects.create("Thèse", "").unwrap();
    f.projects.link_job(q.id, job.id, None).unwrap();
    let both = f.projects.documents(job.id, source.id).unwrap();
    assert_eq!(both.len(), 2);
    assert!(
        both.iter()
            .any(|d| d.project_id == q.id && d.description.is_empty())
    );
}
