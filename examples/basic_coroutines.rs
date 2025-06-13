use coroutine_scheduler::Runtime;

// 获取任务对应的颜色代码
fn get_task_color(task_id: usize) -> &'static str {
    match task_id % 6 {
        1 => "\x1b[31m", // 红色
        2 => "\x1b[32m", // 绿色
        3 => "\x1b[33m", // 黄色
        4 => "\x1b[34m", // 蓝色
        5 => "\x1b[35m", // 洋红色
        0 => "\x1b[36m", // 青色
        _ => "\x1b[0m",  // 默认色
    }
}

// 重置颜色
const RESET_COLOR: &str = "\x1b[0m";

fn main() {
    println!("=== 基本协程调度器示例 ===");

    let runtime = Runtime::new();

    // 创建多个协程任务，每个任务会执行多次并主动让出控制权
    for task_id in 1..=5 {
        runtime.spawn({
            let runtime = runtime.clone();
            move || {
                let color = get_task_color(task_id);
                for step in 1..=3 {
                    println!(
                        "{}任务 {} 正在执行步骤 {}{}",
                        color, task_id, step, RESET_COLOR
                    );
                    if step < 3 {
                        // 主动让出控制权，让其他协程有机会执行
                        runtime.schedule();
                    }
                }
                println!("{}任务 {} 完成{}", color, task_id, RESET_COLOR);
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
            println!("\x1b[36m主协程开始执行\x1b[0m");

            // 在协程内部spawn新的协程
            runtime2.spawn(move || {
                println!("\x1b[32m  子协程1执行\x1b[0m");
            });

            runtime2.spawn(move || {
                println!("\x1b[33m  子协程2执行\x1b[0m");
            });

            println!("\x1b[36m主协程完成\x1b[0m");
        }
    });

    runtime2.wait();
    println!("嵌套协程示例完成!");
}
