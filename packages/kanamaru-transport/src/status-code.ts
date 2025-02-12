export enum Code {
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
