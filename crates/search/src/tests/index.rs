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
        .search(project, &request("hamallisme"), &never, &quiet)
        .unwrap();
    assert_eq!(view.matches, 1);
}

#[test]
fn a_newer_search_stops_one_still_indexing() {
    let f = Fixture::new();
    let doc = f.document("Lettre", InputKind::Pdf);
    f.extraction(&doc, &["Wahhabisme"]);
    let stopped = f
        .search
        .search(f.project, &request("wahhabisme"), &|| true, &quiet);
    assert!(matches!(stopped, Err(SearchError::Stopped)));
    // What was indexed before stopping is kept; the next search finishes the rest.
    assert_eq!(f.find(&request("wahhabisme")).matches, 1);
}

#[test]
fn preparing_ahead_tells_its_progress_and_leaves_searches_nothing_to_do() {
    let f = Fixture::new();
    let first = f.document("Lettre", InputKind::Pdf);
    let second = f.document("Rapport", InputKind::Pdf);
    f.extraction(&first, &["Le hamallisme à Nioro", "Une page"]);
    f.extraction(&second, &["Rien"]);

    let told = std::cell::RefCell::new(Vec::new());
    let prepared = f
        .search
        .prepare(f.project, &never, &|done, total| {
            told.borrow_mut().push((done, total))
        })
        .unwrap();
    assert_eq!(prepared, 3);
    // Once before, then after each job's batch, the jobs in no set order.
    let told = told.into_inner();
    assert_eq!((told[0], told.last().copied()), ((0, 3), Some((3, 3))));
    assert_eq!(told.len(), 3);
    assert_eq!(f.index.size().unwrap().passages, 3);

    // A search then has nothing left to prepare, and says nothing of it.
    let told = std::cell::RefCell::new(Vec::new());
    let view = f
        .search
        .search(f.project, &request("hamallisme"), &never, &|done, total| {
            told.borrow_mut().push((done, total))
        })
        .unwrap();
    assert_eq!(view.matches, 1);
    assert!(told.into_inner().is_empty());
    // Nor does preparing again.
    assert_eq!(f.search.prepare(f.project, &never, &quiet).unwrap(), 0);
    // A newer search stops it, keeping nothing half done.
    f.search.rebuild().unwrap();
    assert!(matches!(
        f.search.prepare(f.project, &|| true, &quiet),
        Err(SearchError::Stopped)
    ));
}
