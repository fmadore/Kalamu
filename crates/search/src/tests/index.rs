//! The scope of a search, and an index brought up to date, made again or stopped.

#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn scope_follows_collections_tags_and_links() {
    let f = Fixture::new();
    let a = f.document("A", InputKind::Pdf);
    let b = f.document("B", InputKind::Pdf);
    let unread = f.document("C", InputKind::Pdf);
    f.extraction(&a, &["Ahmadiyya à Ouagadougou"]);
    let job_b = f.extraction(&b, &["Ahmadiyya à Bobo", "page"]);
    let collection = Collection {
        id: CollectionId::new(),
        project_id: f.project,
        name: "Ouaga".into(),
        created_at: Timestamp::now(),
    };
    f.store.put_collection(&collection).unwrap();
    f.store.add_member(collection.id, a.0).unwrap();
    let mut doc_b = f.store.document(b.0).unwrap();
    doc_b.tag("Bobo");
    f.store.put_document(&doc_b).unwrap();

    let view = f.find(&request("ahmadiyya"));
    assert_eq!(view.documents.len(), 2);
    assert_eq!(view.coverage.unread, 1);
    let mut in_collection = request("ahmadiyya");
    in_collection.collection = Some(collection.id);
    assert_eq!(
        listed(&f.find(&in_collection)),
        vec!["A | 1 | 0 | Ahmadiyya"]
    );
    let mut tagged = request("ahmadiyya");
    tagged.tag = Some("bobo".into());
    assert_eq!(listed(&f.find(&tagged)), vec!["B | 1 | 0 | Ahmadiyya"]);

    // A job untied from its document is no longer searched for it; one deleted is forgotten.
    f.store.remove_link(b.0, job_b.id, b.1.id).unwrap();
    assert_eq!(
        listed(&f.find(&request("ahmadiyya"))),
        vec!["A | 1 | 0 | Ahmadiyya"]
    );
    f.store.delete_job(job_b.id).unwrap();
    assert_eq!(f.search.prune().unwrap(), 1);
    assert!(!f.index.indexed_jobs().unwrap().contains(&job_b.id));
    let _ = unread;
}

#[test]
fn a_lost_index_is_made_again_from_the_store() {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteStore::in_memory().unwrap());
    let path = dir.path().join("search.sqlite");
    let f = Fixture::on(store.clone(), Arc::new(SqliteIndex::open(&path).unwrap()));
    let doc = f.document("Lettre", InputKind::Pdf);
    f.extraction(&doc, &["Hamallisme à Nioro"]);
    assert_eq!(f.find(&request("hamallisme")).matches, 1);

    f.search.rebuild().unwrap();
    assert_eq!(f.index.size().unwrap().passages, 0);
    assert_eq!(f.find(&request("hamallisme")).matches, 1);

    // The file damaged on disk: opened again, it is made anew, and the search still finds.
    drop(f);
    std::fs::write(&path, b"damaged beyond reading, not a database").unwrap();
    let project = store.projects().unwrap()[0].id;
    let search = Search::new(
        store.clone(),
        store.clone(),
        Arc::new(SqliteIndex::open(&path).unwrap()),
    );
    let view = search
        .search(project, &request("hamallisme"), &never)
        .unwrap();
    assert_eq!(view.matches, 1);
}

#[test]
fn a_newer_search_stops_one_still_indexing() {
    let f = Fixture::new();
    let doc = f.document("Lettre", InputKind::Pdf);
    f.extraction(&doc, &["Wahhabisme"]);
    let stopped = f.search.search(f.project, &request("wahhabisme"), &|| true);
    assert!(matches!(stopped, Err(SearchError::Stopped)));
    // What was indexed before stopping is kept; the next search finishes the rest.
    assert_eq!(f.find(&request("wahhabisme")).matches, 1);
}
