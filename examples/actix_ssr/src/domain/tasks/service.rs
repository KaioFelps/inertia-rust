use super::entity::Task;
use std::{sync::LazyLock, time::Duration};
use tokio::{sync::RwLock, time::sleep};

static TASKS_STORAGE: LazyLock<RwLock<Vec<Task>>> = LazyLock::new(|| {
    RwLock::new(vec![
        Task {
            title: "Async Resolvers".into(),
            description: "Lazy props (lazy, on demand, deferred) should be asynchronous!".into(),
            done: true,
        },
        Task {
            title: "Merge Props".into(),
            description:
                "Props that get merged instead of overwrite the existing props in the page!".into(),
            done: true,
        },
        Task {
            title: "Deferred Props".into(),
            description:
                "These props will only be called when needed and after the page first load.".into(),
            done: true,
        },
        Task {
            title: "Easier Props Resolvers".into(),
            description: "Use a macro to create your resolver boilerplate more easily =)".into(),
            done: true,
        },
        Task {
            title: "Release inertia-rust v2".into(),
            description: "We're really close!".into(),
            done: false,
        },
    ])
});

pub async fn save_task(task: Task) {
    TASKS_STORAGE.write().await.insert(0, task);
}

pub async fn get_tasks(page: usize) -> Vec<Task> {
    const PER_PAGE: usize = 3;
    sleep(Duration::from_millis(500)).await;

    TASKS_STORAGE
        .read()
        .await
        .iter()
        .skip((page - 1) * PER_PAGE)
        .take(PER_PAGE)
        .cloned()
        .collect::<Vec<_>>()
}
