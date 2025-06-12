use crate::coroutine::{Coroutine, CoroutineContext, TaskFn};
use std::{
    arch::{asm, naked_asm},
    cell::UnsafeCell,
    collections::BTreeMap,
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[repr(C)]
pub struct InnerRuntime {
    context: CoroutineContext,
    current_coroutine_id: AtomicUsize,
    coroutines: UnsafeCell<BTreeMap<usize, Coroutine>>,
    id_counter: AtomicUsize,
}

impl InnerRuntime {
    pub fn spawn<F: TaskFn>(&self, task: F) {
        let id = self.id_counter.fetch_add(1, Ordering::Relaxed);
        let coroutine = Coroutine::new(id, task, self);
        self.coroutines().insert(coroutine.id(), coroutine);
    }

    pub fn wait(&self) {
        while !self.coroutines().is_empty() {
            self.coroutines().retain(|_, coroutine| {
                self.update_current_coroutine_id(coroutine.id());
                coroutine.resume()
            });
        }
    }

    #[inline(never)]
    pub fn schedule(&self) {
        let current_id = self.current_coroutine_id.load(Ordering::Relaxed);
        // store current context and restore runtime context;
        let current_coroutine = self.coroutines().get(&current_id).unwrap();
        let context = current_coroutine.context();
        let runtime_context = &self.context;

        unsafe {
            asm!(
                "mov x20, {runtime_context}",
                "mov x0, {context}",
                "mov x1, lr",
                "adr x2, 3f",
                "bl {store_context}",
                "mov x0, x20",
                "b {restore_context}",
                "3:",
                context = in(reg) context,
                store_context = sym store_context,
                restore_context = sym restore_context,
                runtime_context = in(reg) runtime_context,
                out("x20") _,
            );
        }
    }

    fn coroutines(&self) -> &mut BTreeMap<usize, Coroutine> {
        unsafe { &mut *self.coroutines.get() }
    }

    fn update_current_coroutine_id(&self, id: usize) {
        self.current_coroutine_id.store(id, Ordering::Relaxed);
    }
}

pub struct Runtime(Arc<InnerRuntime>);

impl Clone for Runtime {
    fn clone(&self) -> Self {
        Runtime(self.0.clone())
    }
}

impl Deref for Runtime {
    type Target = InnerRuntime;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Runtime {
    pub fn new() -> Self {
        let runtime = Arc::new(InnerRuntime {
            id_counter: AtomicUsize::new(0),
            current_coroutine_id: AtomicUsize::new(0),
            coroutines: UnsafeCell::new(BTreeMap::new()),
            context: CoroutineContext::default(),
        });

        Runtime(runtime)
    }
}

// it's not ok to use stack inside this function, because we need to save sp!
// this function simple save all the context to context_ptr
// store context use x8,x10 as temp register
#[unsafe(naked)]
pub(crate) unsafe extern "C" fn store_context(
    context_ptr: *mut CoroutineContext,
    context_lr: usize,
    resume_address: usize,
) {
    naked_asm!(
        "mov x8, x0",
        "stp x19, x20, [x8]",
        "stp x21, x22, [x8, #16]",
        "stp x23, x24, [x8, #32]",
        "stp x25, x26, [x8, #48]",
        "stp x27, x28, [x8, #64]",
        "stp fp, x1, [x8, #80]", // x1 是 lr
        "mov x10, sp",
        "str x10, [x8, #96]",
        "str x2, [x8, #104]", // 保存返回地址，x2 是 resume_address
        "ret",
    );
}

// because of sp is changed inside this function
// so it won't return to caller, and shouldn't use stack inside this function
// get the context address and restore all context and br to it
// restore context use x8 as temp register
#[unsafe(naked)]
pub(crate) unsafe extern "C" fn restore_context(context_ptr: usize) {
    naked_asm!(
        "mov x8, x0", // x8 是 context 上下文地址
        "ldp x19, x20, [x8]",
        "ldp x21, x22, [x8, #16]",
        "ldp x23, x24, [x8, #32]",
        "ldp x25, x26, [x8, #48]",
        "ldp x27, x28, [x8, #64]",
        "ldp fp, lr, [x8, #80]",
        "ldr x10, [x8, #96]", // sp 指针
        "mov sp, x10",
        "ldr x10, [x8, #104]", // pc
        "mov x0, x8",          // context 作为第一个参数
        "br x10",              // 没必要返回，直接跳转过去了
    );
}
