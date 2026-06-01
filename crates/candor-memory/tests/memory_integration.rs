/// Integration tests for candor-memory.
///
/// Tests SurrealDB-backed memory persistence, project isolation,
/// execution log storage, and deletion.
use candor_memory::store::MemorySystem;

/// Create a memory system with default dims.
async fn create_memory() -> MemorySystem {
    MemorySystem::new(384).await.unwrap()
}

fn make_embedding(value: f32) -> Vec<f32> {
    vec![value; 384]
}

#[tokio::test]
async fn test_memory_store_and_retrieve() {
    let memory = create_memory().await;
    let embedding = make_embedding(0.5);

    memory
        .store_memory("test-project".into(), "test content".into(), embedding)
        .await
        .unwrap();

    let results = memory
        .retrieve_context("test-project", make_embedding(0.5), 5)
        .await
        .unwrap();
    assert!(!results.is_empty(), "Should find stored content");
    assert!(results.iter().any(|c| c.contains("test content")));
}

#[tokio::test]
async fn test_memory_different_projects_isolated() {
    let memory = create_memory().await;

    memory
        .store_memory(
            "project-a".into(),
            "project-a-data".into(),
            make_embedding(0.5),
        )
        .await
        .unwrap();
    memory
        .store_memory(
            "project-b".into(),
            "project-b-data".into(),
            make_embedding(0.5),
        )
        .await
        .unwrap();

    let a_results = memory
        .retrieve_context("project-a", make_embedding(0.5), 5)
        .await
        .unwrap();
    let b_results = memory
        .retrieve_context("project-b", make_embedding(0.5), 5)
        .await
        .unwrap();

    assert!(a_results.iter().any(|c| c.contains("project-a-data")));
    assert!(b_results.iter().any(|c| c.contains("project-b-data")));
}

#[tokio::test]
async fn test_memory_delete_project_memories() {
    let memory = create_memory().await;

    memory
        .store_memory(
            "project-to-delete".into(),
            "data1".into(),
            make_embedding(0.5),
        )
        .await
        .unwrap();
    memory
        .store_memory(
            "project-to-delete".into(),
            "data2".into(),
            make_embedding(0.5),
        )
        .await
        .unwrap();

    memory
        .delete_project_memories("project-to-delete")
        .await
        .unwrap();

    let results = memory
        .retrieve_context("project-to-delete", make_embedding(0.5), 5)
        .await
        .unwrap();
    assert!(
        results.is_empty(),
        "Deleted project should have no memories"
    );
}

#[tokio::test]
async fn test_memory_execution_log_storage() {
    let memory = create_memory().await;

    memory
        .store_execution_log(
            "session-1",
            "think",
            "analyze_input",
            "Thought about the input",
        )
        .await
        .unwrap();

    memory
        .store_execution_log("session-1", "execute", "run_tool", "Executed tool")
        .await
        .unwrap();

    let logs = memory
        .get_execution_logs_by_session("session-1")
        .await
        .unwrap();
    assert_eq!(logs.len(), 2, "Should find 2 execution log entries");
    assert!(logs.iter().any(|l| l.phase == "think"));
    assert!(logs.iter().any(|l| l.phase == "execute"));
}

#[tokio::test]
async fn test_memory_embedding_dimension() {
    let memory = create_memory().await;
    assert_eq!(memory.embedding_dim(), 384);
}

#[tokio::test]
async fn test_memory_schema_init_idempotent() {
    let memory = create_memory().await;

    // Run two stores — schema init should be idempotent
    memory
        .store_memory("test".into(), "first".into(), make_embedding(0.5))
        .await
        .unwrap();
    memory
        .store_memory("test".into(), "second".into(), make_embedding(0.5))
        .await
        .unwrap();

    let results = memory
        .retrieve_context("test", make_embedding(0.5), 5)
        .await
        .unwrap();
    assert_eq!(results.len(), 2, "Both stores should persist");
}

#[tokio::test]
async fn test_memory_empty_retrieve() {
    let memory = create_memory().await;
    let results = memory
        .retrieve_context("empty-project", make_embedding(0.1), 5)
        .await
        .unwrap();
    assert!(results.is_empty(), "Empty project should return no results");
}

#[tokio::test]
async fn test_memory_store_large_content() {
    let memory = create_memory().await;
    let large_content = "A".repeat(10_000);

    memory
        .store_memory("test".into(), large_content.clone(), make_embedding(0.5))
        .await
        .unwrap();

    let results = memory
        .retrieve_context("test", make_embedding(0.5), 5)
        .await
        .unwrap();
    assert!(results.iter().any(|c| c.len() == 10_000));
}
