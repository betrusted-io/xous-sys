#[cfg(feature = "unstable_mem")]
mod memoryflags;

#[cfg(feature = "unstable_mem")]
pub use memoryflags::*;

/// Indicates a particular syscall number as used by the Xous kernel.
#[derive(Copy, Clone)]
#[repr(usize)]
pub enum Syscall {
    MapMemory = 2,
    Yield = 3,
    UpdateMemoryFlags = 12,
    ReceiveMessage = 15,
    SendMessage = 16,
    Connect = 17,
    CreateThread = 18,
    UnmapMemory = 19,
    ReturnMemory = 20,
    TerminateProcess = 22,
    TrySendMessage = 24,
    TryConnect = 25,
    GetThreadId = 32,
    Disconnect = 35,
    JoinThread = 36,
    AdjustProcessLimit = 38,
    ReturnScalar = 40,
}

/// Copies of these invocation types here for when we're running
/// in environments without libxous.
#[derive(Copy, Clone)]
#[repr(usize)]
pub enum SyscallResult {
    /// The operation completed successfully
    Ok = 0,
    /// An error was encountered, with the exact error stored in $a1
    Error = 1,
    /// A slice with the offset in $a1 and the length in $a2
    MemoryRange = 3,
    /// A `u32` connection ID stored in $a1
    ConnectionId = 7,
    /// A message was received
    Message = 9,
    /// A `u32` thread id stored in $a1
    ThreadId = 10,
    /// One scalar value, stored in $a1
    Scalar1 = 14,
    /// Two scalar values, stored in $a1 and $a2
    Scalar2 = 15,
    /// Memory that was returned from a syscall, with return values in $a1 and $a2
    MemoryReturned = 18,
    /// Five scalar values, stored in $a1..=$a5
    Scalar5 = 20,
}

#[derive(Copy, Clone)]
/// A list of all known errors that may be returned by the Xous kernel.
#[repr(usize)]
pub enum Error {
    NoError = 0,
    BadAlignment = 1,
    BadAddress = 2,
    OutOfMemory = 3,
    MemoryInUse = 4,
    InterruptNotFound = 5,
    InterruptInUse = 6,
    InvalidString = 7,
    ServerExists = 8,
    ServerNotFound = 9,
    ProcessNotFound = 10,
    ProcessNotChild = 11,
    ProcessTerminated = 12,
    Timeout = 13,
    InternalError = 14,
    ServerQueueFull = 15,
    ThreadNotAvailable = 16,
    UnhandledSyscall = 17,
    InvalidSyscall = 18,
    ShareViolation = 19,
    InvalidThread = 20,
    InvalidPid = 21,
    UnknownError = 22,
    AccessDenied = 23,
    UseBeforeInit = 24,
    DoubleFree = 25,
    DebugInProgress = 26,
    InvalidLimit = 27,
    /// For lookups that result in not found (e.g., searching for keys, resources, names)
    NotFound = 28,
    /// Used when try_from/try_into can't map a number into a smaller number of options
    InvalidCoding = 29,
    /// Used for ECC errors, glitches, power failures, etc.
    HardwareError = 30,
    /// Used when buffers & messages fail to serialize or deserialize
    SerializationError = 31,
    /// Used when a call is correct but its arguments are out of bounds, invalid, or otherwise poorly
    /// specified
    InvalidArgument = 32,
    /// Catch-all for networking related problems (unreachable network, etc.)
    NetworkError = 33,
    /// Catch-all for storage related problems (particularly write/read ECC errors)
    StorageError = 34,
    /// Catch-all for resources that are busy or already allocated
    Unavailable = 35,
    /// For failed parsing attempts
    ParseError = 36,
    /// Invalid core number on multi-core APIs
    InvalidCore = 37,
    /// Reports when verification/check steps fail. Thrown when correctly functioning algorithms determine
    /// that an object is invalid.
    VerificationError = 38,
    /// Used to report higher-severity system security/integrity issues, such as glitch attacks, ECC errors,
    /// memory violations, bad states for hardened bools. Note that bad passwords/credentials should use
    /// "AccessDenied"
    SecurityError = 39,
}

impl From<usize> for Error {
    // Marking this function as "cold" ensures this error path
    // does not get inlined.
    #[cold]
    fn from(src: usize) -> Self {
        match src {
            0 => Self::NoError,
            1 => Self::BadAlignment,
            2 => Self::BadAddress,
            3 => Self::OutOfMemory,
            4 => Self::MemoryInUse,
            5 => Self::InterruptNotFound,
            6 => Self::InterruptInUse,
            7 => Self::InvalidString,
            8 => Self::ServerExists,
            9 => Self::ServerNotFound,
            10 => Self::ProcessNotFound,
            11 => Self::ProcessNotChild,
            12 => Self::ProcessTerminated,
            13 => Self::Timeout,
            14 => Self::InternalError,
            15 => Self::ServerQueueFull,
            16 => Self::ThreadNotAvailable,
            17 => Self::UnhandledSyscall,
            18 => Self::InvalidSyscall,
            19 => Self::ShareViolation,
            20 => Self::InvalidThread,
            21 => Self::InvalidPid,
            // 22 is UnknownError
            23 => Self::AccessDenied,
            24 => Self::UseBeforeInit,
            25 => Self::DoubleFree,
            26 => Self::DebugInProgress,
            27 => Self::InvalidLimit,
            28 => Self::NotFound,
            29 => Self::InvalidCoding,
            30 => Self::HardwareError,
            31 => Self::SerializationError,
            32 => Self::InvalidArgument,
            33 => Self::NetworkError,
            34 => Self::StorageError,
            35 => Self::Unavailable,
            36 => Self::ParseError,
            37 => Self::InvalidCore,
            38 => Self::VerificationError,
            39 => Self::SecurityError,
            _ => Self::UnknownError,
        }
    }
}

impl From<i32> for Error {
    #[cold]
    fn from(src: i32) -> Self {
        let Ok(src) = core::convert::TryInto::<usize>::try_into(src) else {
            return Self::UnknownError;
        };
        src.into()
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::NoError => "no error occurred",
                Error::BadAlignment => "memory was not properly aligned",
                Error::BadAddress => "an invalid address was supplied",
                Error::OutOfMemory => "the process or service has run out of memory",
                Error::MemoryInUse => "the requested address is in use",
                Error::InterruptNotFound =>
                    "the requested interrupt does not exist on this platform",
                Error::InterruptInUse => "the requested interrupt is currently in use",
                Error::InvalidString => "the specified string was not formatted correctly",
                Error::ServerExists => "a server with that address already exists",
                Error::ServerNotFound => "the requetsed server could not be found",
                Error::ProcessNotFound => "the target process does not exist",
                Error::ProcessNotChild =>
                    "the requested operation can only be done on child processes",
                Error::ProcessTerminated => "the target process has crashed",
                Error::Timeout => "the requested operation timed out",
                Error::InternalError => "an internal error occurred",
                Error::ServerQueueFull => "the server has too many pending messages",
                Error::ThreadNotAvailable => "the specified thread does not exist",
                Error::UnhandledSyscall => "the kernel did not recognize that syscall",
                Error::InvalidSyscall => "the syscall had incorrect parameters",
                Error::ShareViolation => "an attempt was made to share memory twice",
                Error::InvalidThread => "tried to resume a thread that was not ready",
                Error::InvalidPid => "kernel attempted to use a pid that was not valid",
                Error::AccessDenied => "no permission to perform the requested operation",
                Error::UseBeforeInit => "attempt to use a service before initialization finished",
                Error::DoubleFree => "the requested resource was freed twice",
                Error::DebugInProgress => "kernel attempted to activate a thread being debugged",
                Error::InvalidLimit => "process attempted to adjust an invalid limit",
                Error::NotFound => "resource or name not found",
                Error::InvalidCoding => "can't map argument onto a valid coding",
                Error::HardwareError => "hardware error",
                Error::SerializationError => "can't (de)serialize buffer or struct",
                Error::InvalidArgument => "invalid, out of bounds, or poorly specified argument(s)",
                Error::NetworkError => "network error",
                Error::StorageError => "storage error",
                Error::Unavailable => "resources busy or already allocated",
                Error::ParseError => "parse error",
                Error::InvalidCore => "invalid core",
                Error::VerificationError => "verification or integrity check failure on object",
                Error::SecurityError => "security or system integrity error",
                Error::UnknownError => "an unknown error occurred",
            }
        )
    }
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl core::error::Error for Error {}

/// Indicates the type of Message that is sent when making a `SendMessage` syscall.
pub enum InvokeType {
    /// Mutably lend the buffer to the server
    LendMut = 1,
    /// Immutably lend
    Lend = 2,
    #[cfg(feature = "unstable_mem")]
    /// Move the buffer from this process into the server
    Move = 3,
    /// Send a scalar message to the server without blocking
    Scalar = 4,
    /// Send a scalar message to the server and wait for a reply
    BlockingScalar = 5,
}

#[derive(Debug, Copy, Clone)]
/// A representation of a connection to a Xous service.
pub struct Connection(pub(crate) u32);

impl From<u32> for Connection {
    fn from(src: u32) -> Connection {
        Connection(src)
    }
}

impl TryFrom<usize> for Connection {
    type Error = core::num::TryFromIntError;
    fn try_from(src: usize) -> Result<Self, Self::Error> {
        Ok(Connection(src.try_into()?))
    }
}

impl Into<u32> for Connection {
    fn into(self) -> u32 {
        self.0
    }
}

impl TryInto<usize> for Connection {
    type Error = core::num::TryFromIntError;
    fn try_into(self) -> Result<usize, Self::Error> {
        self.0.try_into()
    }
}

#[derive(Debug)]
/// The specified Server address could not be parsed
pub enum ServerAddressError {
    /// the length was not 16 bytes
    InvalidLength,
}

pub struct ServerAddress(pub(crate) [u32; 4]);

impl TryFrom<&str> for ServerAddress {
    type Error = ServerAddressError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let b = value.as_bytes();
        if b.len() == 0 || b.len() > 16 {
            return Err(Self::Error::InvalidLength);
        }

        let mut this_temp = [0u8; 16];
        for (dest, src) in this_temp.iter_mut().zip(b.iter()) {
            *dest = *src;
        }

        let mut this = [0u32; 4];
        for (dest, src) in this.iter_mut().zip(this_temp.chunks_exact(4)) {
            *dest = u32::from_le_bytes(src.try_into().unwrap());
        }
        Ok(ServerAddress(this))
    }
}

impl Into<[u32; 4]> for ServerAddress {
    fn into(self) -> [u32; 4] {
        self.0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ThreadId(usize);

impl From<usize> for ThreadId {
    fn from(src: usize) -> ThreadId {
        ThreadId(src)
    }
}

impl Into<usize> for ThreadId {
    fn into(self) -> usize {
        self.0
    }
}

#[derive(Copy, Clone)]
#[repr(usize)]
/// Limits that can be passed to `AdjustLimit`
pub enum Limits {
    HeapMaximum = 1,
    HeapSize = 2,
}
