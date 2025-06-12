use std::{
    alloc::{self, Layout, LayoutError},
    arch::asm,
};

use crate::runtime::{InnerRuntime, restore_context, store_context};

type Register = usize;
pub trait TaskFn: FnOnce() + 'static {}
impl<F: FnOnce() + 'static> TaskFn for F {}

#[derive(Debug, Default)]
// 内存布局要求跟我写的代码保持一致，编译器不能改动
// 16 字节对齐
#[repr(C, align(16))]
pub struct CoroutineContext {
    x19: Register, //0
    x20: Register, //8
    x21: Register, //16
    x22: Register, //24
    x23: Register, //32
    x24: Register, //40
    x25: Register, //48
    x26: Register, //56
    x27: Register, //64
    x28: Register, //72
    fp: Register,  // 80
    lr: Register,  // 88
    sp: Register,  // 96
    pc: Register,  // 104
    task: usize,
    runtime_ptr: usize,
}

impl CoroutineContext {
    fn init<F: TaskFn>(
        &mut self,
        sp: usize,
        fp: usize,
        pc: usize,
        task: *mut F,
        runtime: &InnerRuntime,
    ) {
        self.sp = sp;
        self.fp = fp;
        self.pc = pc;
        self.task = task as usize;
        self.runtime_ptr = runtime as *const InnerRuntime as usize;
    }

    #[inline(never)]
    pub fn resume(&mut self) {
        unsafe {
            asm!(
                "mov x20, {runtime_context_ptr}",
                "mov x21, {coroutine_context_ptr}",
                "mov x0, x20",
                "mov x1, lr",
                "adr x2, 2f",
                "bl {store_context}",
                "mov x0, x21",
                "b {restore_context}",
                "2:",
                runtime_context_ptr = in(reg) self.runtime_ptr,
                store_context = sym store_context,
                restore_context = sym restore_context,
                coroutine_context_ptr = in(reg) self,
                out("x20") _,
                out("x21") _,
            );
        }
    }
}

#[repr(C)]
pub struct Coroutine {
    id: usize,
    context: *mut CoroutineContext,
    stack_bottom: *mut u8,
    stack_size: usize,
}

impl Drop for Coroutine {
    fn drop(&mut self) {
        unsafe {
            alloc::dealloc(
                (self.stack_bottom as *mut u8).sub(self.stack_size),
                Self::LAYOUT.unwrap(),
            );
        };
    }
}

impl Coroutine {
    const COROUTINE_CONTEXT_SIZE: usize = size_of::<CoroutineContext>();
    // 一个内存分页的大小
    const STACK_SIZE: usize = 4 << 20;
    const LAYOUT: Result<Layout, LayoutError> = Layout::from_size_align(Self::STACK_SIZE, 16);

    pub fn new<F: TaskFn>(id: usize, task: F, runtime: &InnerRuntime) -> Self {
        let task = Box::into_raw(Box::new(task));
        let stack = unsafe { alloc::alloc(Self::LAYOUT.unwrap()) };
        // top of stack is high address and bottom of stack is low address
        let stack_top = unsafe { stack.add(Self::STACK_SIZE) };
        // put CoroutineContext on the top of stack
        // reserve COROUTINE_CONTEXT_SIZE to store context;
        // CoroutineContext is 16 byte align
        let context =
            unsafe { stack_top.sub(Self::COROUTINE_CONTEXT_SIZE) as *mut CoroutineContext };
        unsafe { std::ptr::write_bytes(stack_top, 0, Self::COROUTINE_CONTEXT_SIZE) };
        let stack_bottom = context as *mut u8;
        let stack_top = stack_bottom;
        let context = unsafe { &mut *context };

        context.init(
            stack_top as usize,
            stack_top as usize,
            run_coroutine::<F> as usize,
            task,
            runtime,
        );

        Self {
            id,
            context,
            stack_bottom,
            stack_size: Self::STACK_SIZE - Self::COROUTINE_CONTEXT_SIZE,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn context(&self) -> &CoroutineContext {
        unsafe { &*self.context }
    }

    pub fn context_mut(&mut self) -> &mut CoroutineContext {
        unsafe { &mut *self.context }
    }

    pub fn resume(&mut self) -> bool {
        if self.context().pc == 0 {
            return false;
        }

        self.context_mut().resume();
        true
    }
}

#[inline(never)]
unsafe extern "C" fn run_coroutine<F: TaskFn>(context: *mut CoroutineContext) {
    let context = unsafe { &mut *context };
    let task = unsafe { Box::from_raw(context.task as *mut F) };
    task();
    // after task return , just restore runtime context and return;
    context.pc = 0;
    unsafe {
        asm!(
            "mov x0, {runtime_context}",
            "b {restore_context}", // 没有必要 return，直接跳回去了
            options(noreturn),
            runtime_context = in(reg) context.runtime_ptr,
            restore_context = sym restore_context,
        );
    }
}
