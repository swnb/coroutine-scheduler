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
        executed.sort(); // 排序因为执行顺序可能不固定
        assert_eq!(*executed, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_coroutine_scheduling() {
        println!("=== 开始协程调度测试 ===");
        let runtime = Runtime::new();
        let execution_order = Rc::new(RefCell::new(Vec::new()));

        println!("创建3个协程，每个协程执行3个步骤...");
        for task_id in 0..3 {
            let execution_order = execution_order.clone();
            let runtime_clone = runtime.clone();
            runtime.spawn(move || {
                println!(">>> 协程 {} 开始执行", task_id);
                for i in 0..3 {
                    println!("  >> 协程 {} 执行步骤 {} [调用前]", task_id, i);
                    execution_order.borrow_mut().push((task_id, i));
                    println!("  >> 协程 {} 执行步骤 {} [记录完成]", task_id, i);

                    if i < 2 {
                        println!("  >> 协程 {} 即将调用 schedule() 让出控制权...", task_id);
                        runtime_clone.schedule(); // 主动让出控制权
                        println!("  >> 协程 {} 从 schedule() 返回，继续执行", task_id);
                    }
                }
                println!(">>> 协程 {} 完成所有步骤", task_id);
            });
        }

        println!("所有协程已创建，开始调用 runtime.wait()...");
        runtime.wait();
        println!("runtime.wait() 返回，所有协程执行完毕");

        let order = execution_order.borrow();
        println!("=== 最终执行顺序分析 ===");
        println!("实际执行顺序: {:?}", *order);

        // 分析执行模式
        println!("=== 执行模式分析 ===");
        for (task_id, step) in order.iter() {
            println!("  协程 {} -> 步骤 {}", task_id, step);
        }

        // 验证所有任务都完成了
        assert_eq!(order.len(), 9, "应该总共执行9个步骤（3个协程 × 3个步骤）");

        // 验证每个任务都执行了3次
        let mut task_counts = [0; 3];
        for (task_id, _) in order.iter() {
            task_counts[*task_id] += 1;
        }
        assert_eq!(task_counts, [3, 3, 3], "每个协程都应该执行3个步骤");

        // 分析调度行为
        let expected_sequential = vec![
            (0, 0),
            (0, 1),
            (0, 2), // 如果没有真正的调度，协程0会连续执行完
            (1, 0),
            (1, 1),
            (1, 2), // 然后协程1连续执行完
            (2, 0),
            (2, 1),
            (2, 2), // 最后协程2连续执行完
        ];

        let expected_interleaved = vec![
            (0, 0),
            (1, 0),
            (2, 0), // 如果有真正的调度，应该交错执行
            (0, 1),
            (1, 1),
            (2, 1),
            (0, 2),
            (1, 2),
            (2, 2),
        ];

        let actual_execution: Vec<_> = order
            .iter()
            .map(|(task_id, step)| (*task_id, *step))
            .collect();

        println!("=== 调度行为判断 ===");
        if actual_execution == expected_sequential {
            println!("❌ 发现BUG: 协程按顺序执行，schedule() 没有实现真正的调度切换");
            println!("   当前行为: 每个协程连续执行完所有步骤后才执行下一个协程");
            println!("   问题原因: schedule() 调用后立即返回到同一协程，而不是切换到其他协程");
        } else if actual_execution == expected_interleaved {
            println!("✅ 调度正常: 协程正确实现了交错执行");
        } else {
            println!("⚠️ 未知执行模式: {:?}", actual_execution);
        }

        // 这个断言会失败，因为当前有调度bug
        assert_ne!(
            actual_execution, expected_sequential,
            "BUG: 调度器没有实现真正的协程切换，协程应该交错执行而不是顺序执行"
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
                runtime_clone.schedule(); // 每次递增后让出控制权
            }
        });

        let counter_clone2 = counter.clone();
        runtime.spawn(move || {
            // 这个协程只是验证能正常执行
            let current = *counter_clone2.borrow();
            *counter_clone2.borrow_mut() = current + 100;
        });

        runtime.wait();
        assert_eq!(*counter.borrow(), 105); // 5 + 100
    }

    #[test]
    fn test_empty_runtime() {
        let runtime = Runtime::new();
        // 没有spawn任何协程，wait应该立即返回
        runtime.wait();
        // 如果能到这里说明wait正确处理了空协程列表的情况
        assert!(true);
    }

    #[test]
    fn test_simple_scheduling() {
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
        println!("简单测试的执行顺序: {:?}", *order);

        let actual: Vec<_> = order
            .iter()
            .map(|(task_id, step)| (*task_id, *step))
            .collect();

        // 检查是顺序执行还是交错执行
        let sequential = vec![
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

        let interleaved = vec![
            (0, 0),
            (1, 0),
            (2, 0),
            (0, 1),
            (1, 1),
            (2, 1),
            (0, 2),
            (1, 2),
            (2, 2),
        ];

        if actual == sequential {
            println!("❌ 顺序执行 (可能的bug)");
        } else if actual == interleaved {
            println!("✅ 交错执行 (正确)");
        } else {
            println!("⚠️ 其他模式: {:?}", actual);
        }

        assert_eq!(order.len(), 9);
    }
}
