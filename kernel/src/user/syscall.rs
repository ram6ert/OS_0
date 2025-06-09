use crate::{
    INIT_PROGRAM,
    arch::x86_64::{serial::COM1, wait_for_irq},
    mm::definitions::KERNEL_REGION_BEGIN,
    task::{MemoryReader, TASK_MANAGER, schedule_next_task},
};

pub fn handle_syscall(num: usize, parameters: &[usize]) -> usize {
    if num == 1 {
        syscall_write(parameters)
    } else if num == 3 {
        syscall_getpid(parameters)
    } else if num == 4 {
        syscall_spawn(parameters)
    } else if num == 5 {
        syscall_yield(parameters)
    } else {
        !0
    }
}

fn syscall_write(parameters: &[usize]) -> usize {
    let fp = parameters[0];
    let data = parameters[1];
    let len = parameters[2];
    if data + len < KERNEL_REGION_BEGIN && fp == 1 {
        unsafe {
            let bs = core::slice::from_raw_parts(data as *const u8, len);
            for b in bs {
                #[allow(static_mut_refs)]
                COM1.write_byte(*b);
            }
            0
        }
    } else {
        1
    }
}

fn syscall_getpid(_parameters: &[usize]) -> usize {
    TASK_MANAGER
        .lock()
        .current_task()
        .map(|x| x.id())
        .unwrap_or(0)
}

fn syscall_spawn(_parameters: &[usize]) -> usize {
    TASK_MANAGER
        .lock()
        .add_task(MemoryReader::new(INIT_PROGRAM.as_ptr(), INIT_PROGRAM.len()));
    0
}

// not real yield()
fn syscall_yield(_parameters: &[usize]) -> usize {
    unsafe {
        wait_for_irq();
    }
    0
}
