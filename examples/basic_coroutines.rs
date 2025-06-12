use coroutine_scheduler::Runtime;

fn main() {
    println!("=== 基本协程调度器示例 ===");

    let runtime = Runtime::new();

    // 创建多个协程任务，每个任务会执行多次并主动让出控制权
    for task_id in 1..=5 {
        runtime.spawn({
            let runtime = runtime.clone();
            move || {
                for step in 1..=3 {
                    println!("任务 {} 正在执行步骤 {}", task_id, step);
                    if step < 3 {
                        // 主动让出控制权，让其他协程有机会执行
                        runtime.schedule();
                    }
                }
                println!("任务 {} 完成", task_id);
            }
        });
    }

    println!("开始运行所有协程...");
    runtime.wait();
    println!("所有协程执行完成!");

    // 演示嵌套spawn
    println!("\n=== 嵌套协程示例 ===");
    let runtime2 = Runtime::new();

    runtime2.spawn({
        let runtime2 = runtime2.clone();
        move || {
            println!("主协程开始执行");

            // 在协程内部spawn新的协程
            runtime2.spawn(move || {
                println!("  子协程1执行");
            });

            runtime2.spawn(move || {
                println!("  子协程2执行");
            });

            println!("主协程完成");
        }
    });

    runtime2.wait();
    println!("嵌套协程示例完成!");
}
