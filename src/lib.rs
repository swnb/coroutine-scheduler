pub mod coroutine;
pub mod runtime;
pub use runtime::Runtime;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_single_coroutine() {
        let runtime = Runtime::new();
        let executed = Rc::new(RefCell::new(false));
        let executed_clone = executed.clone();

        runtime.spawn(move || {
            *executed_clone.borrow_mut() = true;
        });

        runtime.wait();
        assert!(*executed.borrow());
    }

    #[test]
    fn test_multiple_coroutines_execution() {
        let runtime = Runtime::new();
        let executed_tasks = Rc::new(RefCell::new(Vec::new()));

        for task_id in 0..5 {
            let executed_tasks = executed_tasks.clone();
            runtime.spawn(move || {
                executed_tasks.borrow_mut().push(task_id);
            });
        }

        runtime.wait();
        let mut executed = executed_tasks.borrow_mut();
        executed.sort();
        assert_eq!(*executed, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_coroutine_scheduling() {
        let runtime = Runtime::new();
        let execution_order = Rc::new(RefCell::new(Vec::new()));

        for task_id in 0..3 {
            let execution_order = execution_order.clone();
            let runtime_clone = runtime.clone();
            runtime.spawn(move || {
                for i in 0..3 {
                    execution_order.borrow_mut().push((task_id, i));
                    if i < 2 {
                        runtime_clone.schedule();
                    }
                }
            });
        }

        runtime.wait();
        let order = execution_order.borrow();

        // 验证所有任务都完成了
        assert_eq!(order.len(), 9);

        // 验证每个任务都执行了3次
        let mut task_counts = [0; 3];
        for (task_id, _) in order.iter() {
            task_counts[*task_id] += 1;
        }
        assert_eq!(task_counts, [3, 3, 3]);

        // 验证调度行为：应该实现交错执行而不是顺序执行
        let expected_sequential = vec![
            (0, 0),
            (0, 1),
            (0, 2),
            (1, 0),
            (1, 1),
            (1, 2),
            (2, 0),
            (2, 1),
            (2, 2),
        ];

        let actual_execution: Vec<_> = order
            .iter()
            .map(|(task_id, step)| (*task_id, *step))
            .collect();

        assert_ne!(
            actual_execution, expected_sequential,
            "调度器应该实现协程切换，而不是顺序执行"
        );
    }

    #[test]
    fn test_coroutine_yield_behavior() {
        let runtime = Runtime::new();
        let counter = Rc::new(RefCell::new(0));

        let counter_clone = counter.clone();
        let runtime_clone = runtime.clone();
        runtime.spawn(move || {
            for _ in 0..5 {
                let current = *counter_clone.borrow();
                *counter_clone.borrow_mut() = current + 1;
                runtime_clone.schedule();
            }
        });

        let counter_clone2 = counter.clone();
        runtime.spawn(move || {
            let current = *counter_clone2.borrow();
            *counter_clone2.borrow_mut() = current + 100;
        });

        runtime.wait();
        assert_eq!(*counter.borrow(), 105);
    }

    #[test]
    fn test_empty_runtime() {
        let runtime = Runtime::new();
        runtime.wait();
        assert!(true);
    }
}
