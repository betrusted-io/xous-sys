#![no_std]
mod definitions;
pub use definitions::*;

#[cfg(feature = "unstable_mem")]
mod unstable;
#[cfg(feature = "unstable_mem")]
pub use unstable::*;

/// Perform a raw syscall without checking the return value.
///
/// Safety: The safety of the function depends on the syscall
/// passed in `a0`.
#[inline]
pub unsafe fn raw_syscall(
    mut a0: usize,
    mut a1: usize,
    mut a2: usize,
    mut a3: usize,
    mut a4: usize,
    mut a5: usize,
    mut a6: usize,
    mut a7: usize,
) -> (usize, usize, usize, usize, usize, usize, usize, usize) {
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("a0") a0,
            inlateout("a1") a1,
            inlateout("a2") a2,
            inlateout("a3") a3,
            inlateout("a4") a4,
            inlateout("a5") a5,
            inlateout("a6") a6,
            inlateout("a7") a7,
        )
    };
    (a0, a1, a2, a3, a4, a5, a6, a7)
}

/// Perform a type-checked syscall and check the return value.
///
/// Safety: The safety of this function depends on the syscall.
#[inline]
pub unsafe fn syscall(
    a0: Syscall,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
) -> Result<(usize, usize, usize, usize, usize, usize, usize, usize), Error> {
    let result = unsafe { raw_syscall(a0 as usize, a1, a2, a3, a4, a5, a6, a7) };
    if result.0 == 1 {
        return Err(result.1.into());
    }
    Ok((
        result.0, result.1, result.2, result.3, result.4, result.5, result.6, result.7,
    ))
}

/// Mutably lend the buffer to the server, blocking if
/// the mailbox is full.
pub fn lend_mut(
    connection: Connection,
    opcode: usize,
    data: &mut [u8],
    arg1: usize,
    arg2: usize,
) -> Result<(usize, usize), Error> {
    let result = unsafe {
        syscall(
            Syscall::SendMessage,
            connection.0 as _,
            InvokeType::LendMut as _,
            opcode,
            data.as_ptr() as usize,
            data.len(),
            arg1,
            arg2,
        )?
    };
    Ok((result.1, result.2))
}

/// Attempt to mutably lend the buffer to the server.
/// Returns an error if the server's mailbox is full.
pub fn try_lend_mut(
    connection: Connection,
    opcode: usize,
    data: &mut [u8],
    arg1: usize,
    arg2: usize,
) -> Result<(usize, usize), Error> {
    let result = unsafe {
        syscall(
            Syscall::TrySendMessage,
            connection.0 as _,
            InvokeType::LendMut as _,
            opcode,
            data.as_ptr() as usize,
            data.len(),
            arg1,
            arg2,
        )?
    };
    Ok((result.1, result.2))
}

/// Lend the buffer to the server. Blocks if the mailbox is full.
pub fn lend(
    connection: Connection,
    opcode: usize,
    data: &[u8],
    arg1: usize,
    arg2: usize,
) -> Result<(usize, usize), Error> {
    let result = unsafe {
        syscall(
            Syscall::SendMessage,
            connection.0 as _,
            InvokeType::Lend as _,
            opcode,
            data.as_ptr() as usize,
            data.len(),
            arg1,
            arg2,
        )?
    };
    Ok((result.1, result.2))
}

/// Attempt to lend the slice to the server. Returns an error if
/// the mailbox is full.
pub fn try_lend(
    connection: Connection,
    opcode: usize,
    data: &[u8],
    arg1: usize,
    arg2: usize,
) -> Result<(usize, usize), Error> {
    let result = unsafe {
        syscall(
            Syscall::TrySendMessage,
            connection.0 as _,
            InvokeType::Lend as _,
            opcode,
            data.as_ptr() as usize,
            data.len(),
            arg1,
            arg2,
        )?
    };
    Ok((result.1, result.2))
}

/// Send 5 scalar values to the server, blocking if the mailbox is full.
pub fn scalar(connection: Connection, args: [usize; 5]) -> Result<(), Error> {
    unsafe {
        syscall(
            Syscall::SendMessage,
            connection.0 as _,
            InvokeType::Scalar as _,
            args[0],
            args[1],
            args[2],
            args[3],
            args[4],
        )?
    };
    Ok(())
}

/// Attempt to send 5 scalar values to the server.
pub fn try_scalar(connection: Connection, args: [usize; 5]) -> Result<(), Error> {
    unsafe {
        syscall(
            Syscall::TrySendMessage,
            connection.0 as _,
            InvokeType::Scalar as _,
            args[0],
            args[1],
            args[2],
            args[3],
            args[4],
        )?
    };
    Ok(())
}

/// Send 5 scalar arguments to a server and wait for a response.
/// If the server mailbox is full, will block until it is available.
pub fn blocking_scalar(connection: Connection, args: [usize; 5]) -> Result<[usize; 5], Error> {
    let result = unsafe {
        syscall(
            Syscall::SendMessage,
            connection.0 as _,
            InvokeType::BlockingScalar as _,
            args[0],
            args[1],
            args[2],
            args[3],
            args[4],
        )?
    };
    Ok([result.1, result.2, result.3, result.4, result.5])
}

/// Attempt to send 5 scalar arguments to a server. Returns an error
/// if the server mailbox is full.
pub fn try_blocking_scalar(connection: Connection, args: [usize; 5]) -> Result<[usize; 5], Error> {
    let result = unsafe {
        syscall(
            Syscall::TrySendMessage,
            connection.0 as _,
            InvokeType::BlockingScalar as _,
            args[0],
            args[1],
            args[2],
            args[3],
            args[4],
        )?
    };
    Ok([result.1, result.2, result.3, result.4, result.5])
}

/// Connects to a Xous server represented by the specified `address`.
///
/// The current thread will block until the server is available. Returns
/// an error if the server cannot accept any more connections.
pub fn connect(address: ServerAddress) -> Result<Connection, Error> {
    let result = unsafe {
        syscall(
            Syscall::Connect,
            address.0[0] as usize,
            address.0[1] as usize,
            address.0[2] as usize,
            address.0[3] as usize,
            0,
            0,
            0,
        )?
    };
    Ok(Connection(result.1 as u32))
}

/// Attempts to connect to a Xous server represented by the specified `address`.
///
/// If the server does not exist then None is returned.
pub fn try_connect(address: ServerAddress) -> Result<Option<Connection>, Error> {
    let result = unsafe {
        syscall(
            Syscall::Connect,
            address.0[0] as usize,
            address.0[1] as usize,
            address.0[2] as usize,
            address.0[3] as usize,
            0,
            0,
            0,
        )
    };
    match result {
        Ok(val) => Ok(Some(Connection(val.1 as u32))),
        Err(Error::ServerNotFound) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Attempts to disconnect from the specified Xous server.
///
/// Safety: If this connection is in use elsewhere in this program,
/// then those connections will fail. The internal [Connection] ID
/// may be reused in a future connection attempt.
pub unsafe fn disconnect(connection: Connection) -> Result<(), Error> {
    unsafe { syscall(Syscall::Disconnect, connection.0 as _, 0, 0, 0, 0, 0, 0)? };
    Ok(())
}

/// Terminates the current process and returns the specified code to the parent process.
pub fn exit(exit_code: u32) -> ! {
    let _ = unsafe { syscall(Syscall::TerminateProcess, exit_code as _, 0, 0, 0, 0, 0, 0) };
    unreachable!();
}

/// Suspends the current thread and allow another thread to run. This thread may
/// continue executing again immediately if there are no other threads available
/// to run on the system.
pub fn do_yield() {
    let _ = unsafe { syscall(Syscall::Yield, 0, 0, 0, 0, 0, 0, 0) };
}

/// Waits for the given thread to terminate and returns the exit code from that thread.
pub fn join_thread(thread_id: ThreadId) -> Result<usize, Error> {
    let result = unsafe { syscall(Syscall::JoinThread, thread_id.into(), 0, 0, 0, 0, 0, 0)? };
    Ok(result.1)
}

/// Gets the current thread's ID.
pub fn thread_id() -> Result<ThreadId, Error> {
    let result = unsafe { syscall(Syscall::GetThreadId, 0, 0, 0, 0, 0, 0, 0)? };
    Ok(result.1.into())
}

/// Adjusts the given `knob` limit to match the new value `new`. The current value must
/// match the `current` in order for this to take effect.
///
/// The new value is returned as a result of this call. If the call fails, then the old
/// value is returned. In either case, this function returns successfully.
///
/// An error is generated if the `knob` is not a valid limit, or if the call
/// would not succeed.
pub fn adjust_limit(knob: Limits, current: usize, new: usize) -> Result<usize, Error> {
    let result = unsafe {
        syscall(
            Syscall::AdjustProcessLimit,
            knob as usize,
            current,
            new,
            0,
            0,
            0,
            0,
        )?
    };

    if result.0 == SyscallResult::Scalar2 as usize && result.1 == knob as usize {
        Ok(result.2)
    } else if result.0 == SyscallResult::Scalar5 as usize && result.1 == knob as usize {
        Ok(result.1)
    } else {
        Err(Error::InternalError)
    }
}
