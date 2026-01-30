//! Unstable functions that deal with memory. The exact semantics of how memory
//! is handled when it is returned from `MapMemory` are not yet well-defined,
//! and are subject to change. Ideally we'd use the Rust allocator API, but that
//! is still in-progress.

use crate::definitions::{
    Connection, Error, InvokeType, MemoryFlags, Syscall, SyscallResult, ThreadId,
};
use crate::syscall;

/// Move the buffer to the server, blocking if
/// the mailbox is full.
pub fn r#move(
    connection: Connection,
    opcode: usize,
    data: Box<[u8]>,
    arg1: usize,
    arg2: usize,
) -> Result<(), Error> {
    let (addr, len) = (data.as_ptr() as usize, data.len());
    // Memory will be moved even if this call fails
    core::mem::forget(data);

    unsafe {
        syscall(
            Syscall::SendMessage,
            connection.0 as _,
            InvokeType::Move as _,
            opcode,
            addr,
            len,
            arg1,
            arg2,
        )?
    };
    Ok(())
}

/// Attempt to mutably lend the buffer to the server.
/// Returns an error if the server's mailbox is full.
pub fn try_move(
    connection: Connection,
    opcode: usize,
    data: Box<[u8]>,
    arg1: usize,
    arg2: usize,
) -> Result<(), Error> {
    let (addr, len) = (data.as_ptr() as usize, data.len());
    // Memory will be moved even if this call fails
    core::mem::forget(data);

    unsafe {
        syscall(
            Syscall::TrySendMessage,
            connection.0 as _,
            InvokeType::Move as _,
            opcode,
            addr,
            len,
            arg1,
            arg2,
        )?
    };
    Ok(())
}

/// Allocates memory from the system.
///
/// An optional physical and/or virtual address may be specified in order to
/// ensure memory is allocated at specific offsets, otherwise the kernel will
/// select an address.
///
/// # Safety
///
/// This function is safe unless a virtual address is specified. In that case,
/// the kernel will return an alias to the existing range. This violates Rust's
/// pointer uniqueness guarantee.
pub unsafe fn map_memory<T>(
    phys: Option<core::ptr::NonNull<T>>,
    virt: Option<core::ptr::NonNull<T>>,
    count: usize,
    flags: MemoryFlags,
) -> Result<Box<[T]>, Error> {
    let result = unsafe {
        syscall(
            Syscall::MapMemory,
            phys.map(|p| p.as_ptr() as usize).unwrap_or_default(),
            virt.map(|p| p.as_ptr() as usize).unwrap_or_default(),
            count * size_of::<T>(),
            flags.bits(),
            0,
            0,
            0,
        )?
    };

    if result.0 != SyscallResult::MemoryRange as usize {
        return Err(Error::InternalError);
    }

    let start = core::ptr::with_exposed_provenance_mut::<T>(result.1);
    let len = result.2 / size_of::<T>();
    Ok(unsafe { Box::from_raw(core::slice::from_raw_parts_mut(start, len)) })
}

/// Destroys the given memory, returning it to the compiler.
///
/// Safety: The memory pointed to by `range` should not be used after this
/// function returns, even if this function returns Err().
pub unsafe fn unmap_memory<T>(range: Box<[T]>) -> Result<(), Error> {
    unsafe {
        syscall(
            Syscall::UnmapMemory,
            range.as_ptr() as usize,
            range.len() * size_of::<T>(),
            0,
            0,
            0,
            0,
            0,
        )?
    };
    // Memory has been freed by the kernel
    core::mem::forget(range);
    Ok(())
}

/// Adjusts the memory flags for the given range.
///
/// This can be used to remove flags from a given region in order to harden
/// memory access. Note that flags may only be removed and may never be added.
///
/// Safety: The memory pointed to by `range` may become inaccessible or have its
/// mutability removed. It is up to the caller to ensure that the flags specified
/// by `new_flags` are upheld, otherwise the program will crash.
pub unsafe fn update_memory_flags<T>(
    range: &mut Box<[T]>,
    new_flags: MemoryFlags,
) -> Result<(), Error> {
    unsafe {
        syscall(
            Syscall::UpdateMemoryFlags,
            range.as_mut_ptr() as _,
            range.len() * size_of::<T>(),
            new_flags.bits(),
            0, // Process ID flag is currently None
            0,
            0,
            0,
        )?
    };
    Ok(())
}

/// Creates a thread with a given stack and up to four arguments.
pub fn create_thread(
    start: *mut usize,
    stack: Box<[u8]>,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
) -> Result<ThreadId, Error> {
    let result = unsafe {
        syscall(
            Syscall::CreateThread,
            start as usize,
            stack.as_ptr() as _,
            stack.len(),
            arg0,
            arg1,
            arg2,
            arg3,
        )?
    };

    // Stack is now owned by the thread
    core::mem::forget(stack);

    if result.0 != SyscallResult::ThreadId as usize {
        return Err(Error::InternalError);
    }
    Ok(result.1.into())
}
