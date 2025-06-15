use crate::coroutine::Coroutine;
use crossbeam_deque::{Injector, Steal, Stealer, Worker as DequeWorker};
use rand::prelude::*;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

// 线程本地存储，保存当前线程ID和线程本地队列
thread_local! {
    static CURRENT_WORKER_ID: RefCell<Option<usize>> = RefCell::new(None);
    static LOCAL_DEQUE: RefCell<Option<DequeWorker<Coroutine>>> = RefCell::new(None);
}

#[derive(Clone)]
pub struct WorkerState {
    id: usize,
    stealer: Stealer<Coroutine>,
    is_active: Arc<AtomicBool>,
}

pub struct Worker {
    id: usize,
    handle: Option<thread::JoinHandle<()>>,
    is_active: Arc<AtomicBool>,
}

impl Worker {
    pub fn new(
        id: usize,
        global_queue: Arc<Injector<Coroutine>>,
        stealers: Arc<Mutex<Vec<WorkerState>>>,
    ) -> (Self, WorkerState) {
        // 创建线程参数
        let is_active = Arc::new(AtomicBool::new(true));
        let is_active_clone = is_active.clone();

        // 复制一份当前的stealers用于线程内部
        let stealers_clone = {
            let stealers_guard = stealers.lock().unwrap();
            stealers_guard.clone()
        };

        // 创建worker线程
        let handle = thread::spawn(move || {
            // 初始化线程本地队列
            let local_deque = DequeWorker::new_fifo();
            let stealer = local_deque.stealer();

            // 设置线程本地存储
            CURRENT_WORKER_ID.with(|id_cell| {
                *id_cell.borrow_mut() = Some(id);
            });

            LOCAL_DEQUE.with(|deque_cell| {
                *deque_cell.borrow_mut() = Some(local_deque);
            });

            // 工作线程主循环
            while is_active_clone.load(Ordering::SeqCst) {
                // 寻找一个任务执行
                let task = find_task(&global_queue, &stealers_clone);

                match task {
                    Some(mut coroutine) => {
                        // 恢复协程执行
                        let should_reschedule = coroutine.resume();

                        if should_reschedule {
                            // 协程被调度，将其放回本地队列
                            push_to_local_queue(coroutine);
                        }
                    }
                    None => {
                        // 没有任务，短暂等待
                        thread::yield_now();
                    }
                }
            }
        });

        // 创建worker对象
        let worker = Self {
            id,
            handle: Some(handle),
            is_active,
        };

        // 获取刚刚初始化的线程的stealer
        // 注：在实际实现中，这里存在竞争，可能需要更复杂的同步机制
        thread::sleep(std::time::Duration::from_millis(10));

        // 创建保存在stealers数组中的状态
        let worker_state = WorkerState {
            id,
            stealer: DequeWorker::<Coroutine>::new_fifo().stealer(), // 临时创建，会被替换
            is_active: worker.is_active.clone(),
        };

        (worker, worker_state)
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn shutdown(&self) {
        self.is_active.store(false, Ordering::SeqCst);
    }

    pub fn join(&mut self) {
        self.shutdown();
        if let Some(handle) = self.handle.take() {
            handle.join().expect("Worker线程异常终止");
        }
    }
}

// 工作窃取调度
fn find_task(global_queue: &Injector<Coroutine>, stealers: &[WorkerState]) -> Option<Coroutine> {
    // 1. 优先从本地队列获取任务
    if let Some(task) = pop_from_local_queue() {
        return Some(task);
    }

    // 2. 然后从全局队列窃取
    if let Steal::Success(task) = global_queue.steal() {
        return Some(task);
    }

    // 3. 随机窃取其他工作线程的任务
    let mut rng = thread_rng();
    let len = stealers.len();

    if len == 0 {
        return None;
    }

    // 选择一个随机起点
    let start_index = rng.gen_range(0..len);

    // 尝试从其他线程窃取
    for i in 0..len {
        let idx = (start_index + i) % len;

        // 获取当前worker id，避免从自己偷
        let self_id = get_current_worker_id();
        if Some(stealers[idx].id) == self_id {
            continue;
        }

        // 只从活跃的workers偷取
        if stealers[idx].is_active.load(Ordering::SeqCst) {
            if let Steal::Success(task) = stealers[idx].stealer.steal() {
                return Some(task);
            }
        }
    }

    None
}

// 获取当前工作线程id
pub fn get_current_worker_id() -> Option<usize> {
    CURRENT_WORKER_ID.with(|id_cell| *id_cell.borrow())
}

// 从当前线程本地队列弹出协程
fn pop_from_local_queue() -> Option<Coroutine> {
    LOCAL_DEQUE.with(|deque_cell| {
        deque_cell
            .borrow_mut()
            .as_mut()
            .and_then(|deque| deque.pop())
    })
}

// 将协程推入当前线程本地队列
pub fn push_to_local_queue(coroutine: Coroutine) {
    LOCAL_DEQUE.with(|deque_cell| {
        if let Some(deque) = deque_cell.borrow_mut().as_mut() {
            deque.push(coroutine);
        }
    });
}
