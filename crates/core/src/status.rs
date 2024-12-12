use std::{error::Error, fmt::Display, sync::Arc};

use serde::Serialize;
use tauri::http::HeaderMap;

type StatusMessage = Arc<Box<dyn Error + Send + Sync + 'static>>;

#[derive(Debug, Serialize, Clone)]
pub struct Status {
    code: Code,
    #[serde(with = "crate::ipc::header_map")]
    metadata: HeaderMap,
    #[serde(with = "self::message")]
    message: StatusMessage,
}

/// gRPC status codes used by [`Status`].
///
/// These variants match the [gRPC status codes].
///
/// [gRPC status codes]: https://github.com/grpc/grpc/blob/master/doc/statuscodes.md#status-codes-and-their-use-in-grpc
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Code {
    /// The operation completed successfully.
    Ok = 0,

    /// The operation was cancelled.
    Cancelled = 1,

    /// Unknown error.
    Unknown = 2,

    /// Client specified an invalid argument.
    InvalidArgument = 3,

    /// Deadline expired before operation could complete.
    DeadlineExceeded = 4,

    /// Some requested entity was not found.
    NotFound = 5,

    /// Some entity that we attempted to create already exists.
    AlreadyExists = 6,

    /// The caller does not have permission to execute the specified operation.
    PermissionDenied = 7,

    /// Some resource has been exhausted.
    ResourceExhausted = 8,

    /// The system is not in a state required for the operation's execution.
    FailedPrecondition = 9,

    /// The operation was aborted.
    Aborted = 10,

    /// Operation was attempted past the valid range.
    OutOfRange = 11,

    /// Operation is not implemented or not supported.
    Unimplemented = 12,

    /// Internal error.
    Internal = 13,

    /// The service is currently unavailable.
    Unavailable = 14,

    /// Unrecoverable data loss or corruption.
    DataLoss = 15,

    /// The request does not have valid authentication credentials
    Unauthenticated = 16,
}

impl Serialize for Code {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u16(*self as u16)
    }
}

impl Code {
    /// Get description of this `Code`.
    /// ```
    /// fn make_grpc_request() -> kanamaru::Code {
    ///     // ...
    ///     kanamaru::Code::Ok
    /// }
    /// let code = make_grpc_request();
    /// println!("Operation completed. Human readable description: {}", code.description());
    /// ```
    /// If you only need description in `println`, `format`, `log` and other
    /// formatting contexts, you may want to use `Display` impl for `Code`
    /// instead.
    pub fn description(&self) -> &'static str {
        match self {
            Code::Ok => "The operation completed successfully",
            Code::Cancelled => "The operation was cancelled",
            Code::Unknown => "Unknown error",
            Code::InvalidArgument => "Client specified an invalid argument",
            Code::DeadlineExceeded => "Deadline expired before operation could complete",
            Code::NotFound => "Some requested entity was not found",
            Code::AlreadyExists => "Some entity that we attempted to create already exists",
            Code::PermissionDenied => {
                "The caller does not have permission to execute the specified operation"
            }
            Code::ResourceExhausted => "Some resource has been exhausted",
            Code::FailedPrecondition => {
                "The system is not in a state required for the operation's execution"
            }
            Code::Aborted => "The operation was aborted",
            Code::OutOfRange => "Operation was attempted past the valid range",
            Code::Unimplemented => "Operation is not implemented or not supported",
            Code::Internal => "Internal error",
            Code::Unavailable => "The service is currently unavailable",
            Code::DataLoss => "Unrecoverable data loss or corruption",
            Code::Unauthenticated => "The request does not have valid authentication credentials",
        }
    }
}

mod message {
    use serde::{Serialize, Serializer};

    use super::StatusMessage;

    pub fn serialize<S>(message: &StatusMessage, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        message.to_string().serialize(serializer)
    }
}

impl<E: Error + Send + Sync + 'static> From<E> for Status {
    fn from(value: E) -> Self {
        Self {
            code: Code::Unknown,
            metadata: Default::default(),
            message: Arc::new(Box::new(value)),
        }
    }
}

pub trait AsCode {
    fn as_code(&self) -> Code;
}

impl AsCode for Status {
    fn as_code(&self) -> Code {
        self.code
    }
}

impl Status {
    /// Create a new `Status` with the associated code and message.
    pub fn new<M>(code: Code, message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status {
            code,
            message: Arc::new(message.into()),
            metadata: HeaderMap::new(),
        }
    }
    /// The operation completed successfully.
    pub fn ok<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::Ok, message)
    }

    /// The operation was cancelled (typically by the caller).
    pub fn cancelled<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::Cancelled, message)
    }

    /// Unknown error. An example of where this error may be returned is if a
    /// `Status` value received from another address space belongs to an error-space
    /// that is not known in this address space. Also errors raised by APIs that
    /// do not return enough error information may be converted to this error.
    pub fn unknown<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::Unknown, message)
    }

    /// Client specified an invalid argument. Note that this differs from
    /// `FailedPrecondition`. `InvalidArgument` indicates arguments that are
    /// problematic regardless of the state of the system (e.g., a malformed file
    /// name).
    pub fn invalid_argument<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::InvalidArgument, message)
    }

    /// Deadline expired before operation could complete. For operations that
    /// change the state of the system, this error may be returned even if the
    /// operation has completed successfully. For example, a successful response
    /// from a server could have been delayed long enough for the deadline to
    /// expire.
    pub fn deadline_exceeded<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::DeadlineExceeded, message)
    }

    /// Some requested entity (e.g., file or directory) was not found.
    pub fn not_found<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::NotFound, message)
    }

    /// Some entity that we attempted to create (e.g., file or directory) already
    /// exists.
    pub fn already_exists<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::AlreadyExists, message)
    }

    /// The caller does not have permission to execute the specified operation.
    /// `PermissionDenied` must not be used for rejections caused by exhausting
    /// some resource (use `ResourceExhausted` instead for those errors).
    /// `PermissionDenied` must not be used if the caller cannot be identified
    /// (use `Unauthenticated` instead for those errors).
    pub fn permission_denied<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::PermissionDenied, message)
    }

    /// Some resource has been exhausted, perhaps a per-user quota, or perhaps
    /// the entire file system is out of space.
    pub fn resource_exhausted<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::ResourceExhausted, message)
    }

    /// Operation was rejected because the system is not in a state required for
    /// the operation's execution. For example, directory to be deleted may be
    /// non-empty, an rmdir operation is applied to a non-directory, etc.
    ///
    /// A litmus test that may help a service implementor in deciding between
    /// `FailedPrecondition`, `Aborted`, and `Unavailable`:
    /// (a) Use `Unavailable` if the client can retry just the failing call.
    /// (b) Use `Aborted` if the client should retry at a higher-level (e.g.,
    ///     restarting a read-modify-write sequence).
    /// (c) Use `FailedPrecondition` if the client should not retry until the
    ///     system state has been explicitly fixed.  E.g., if an "rmdir" fails
    ///     because the directory is non-empty, `FailedPrecondition` should be
    ///     returned since the client should not retry unless they have first
    ///     fixed up the directory by deleting files from it.
    pub fn failed_precondition<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::FailedPrecondition, message)
    }

    /// The operation was aborted, typically due to a concurrency issue like
    /// sequencer check failures, transaction aborts, etc.
    ///
    /// See litmus test above for deciding between `FailedPrecondition`,
    /// `Aborted`, and `Unavailable`.
    pub fn aborted<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::Aborted, message)
    }

    /// Operation was attempted past the valid range. E.g., seeking or reading
    /// past end of file.
    ///
    /// Unlike `InvalidArgument`, this error indicates a problem that may be
    /// fixed if the system state changes. For example, a 32-bit file system will
    /// generate `InvalidArgument if asked to read at an offset that is not in the
    /// range [0,2^32-1], but it will generate `OutOfRange` if asked to read from
    /// an offset past the current file size.
    ///
    /// There is a fair bit of overlap between `FailedPrecondition` and
    /// `OutOfRange`. We recommend using `OutOfRange` (the more specific error)
    /// when it applies so that callers who are iterating through a space can
    /// easily look for an `OutOfRange` error to detect when they are done.
    pub fn out_of_range<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::OutOfRange, message)
    }

    /// Operation is not implemented or not supported/enabled in this service.
    pub fn unimplemented<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::Unimplemented, message)
    }

    /// Internal errors. Means some invariants expected by underlying system has
    /// been broken. If you see one of these errors, something is very broken.
    pub fn internal<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::Internal, message)
    }

    /// The service is currently unavailable.  This is a most likely a transient
    /// condition and may be corrected by retrying with a back-off.
    ///
    /// See litmus test above for deciding between `FailedPrecondition`,
    /// `Aborted`, and `Unavailable`.
    pub fn unavailable<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::Unavailable, message)
    }

    /// Unrecoverable data loss or corruption.
    pub fn data_loss<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::DataLoss, message)
    }

    /// The request does not have valid authentication credentials for the
    /// operation.
    pub fn unauthenticated<M>(message: M) -> Status
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        Status::new(Code::Unauthenticated, message)
    }
    pub fn message(&self) -> &(dyn Error + Send + Sync + 'static) {
        &**self.message.as_ref() as _
    }
    pub fn metadata(&self) -> &HeaderMap {
        &self.metadata
    }
    pub fn metadata_mut(&mut self) -> &mut HeaderMap {
        &mut self.metadata
    }
    pub fn set_message<M>(&mut self, message: M)
    where
        M: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        self.message = Arc::new(message.into())
    }
    pub fn set_code(&mut self, code: Code) {
        self.code = code
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "status: {:?}, message: {:?}, metadata: {:?}",
            self.code, self.message, self.metadata
        )
    }
}
