package service

const (
	InternalServerErrorStatus  = 500
	InternalServerErrorMessage = "internal server error"
)

// ServiceError is a custom error type that can be used to return an error with a status code.
type ServiceError struct {
	cause      error
	statusCode int
	message    string
}

func NewServiceError(statusCode int, message string) *ServiceError {
	return &ServiceError{
		statusCode: statusCode,
		message:    message,
	}
}

func NewServiceErrorWithCause(statusCode int, message string, cause error) *ServiceError {
	return &ServiceError{
		cause:      cause,
		statusCode: statusCode,
		message:    message,
	}
}

func NewInternalServerError(msg string) *ServiceError {
	return NewServiceError(InternalServerErrorStatus, msg)
}

func NewInternalServerErrorWithCause(msg string, cause error) *ServiceError {
	return NewServiceErrorWithCause(InternalServerErrorStatus, msg, cause)
}

func (e *ServiceError) Error() string {
	return e.message
}

func (e *ServiceError) StatusCode() int {
	return e.statusCode
}

func (e *ServiceError) Cause() error {
	return e.cause
}

func (e *ServiceError) Message() string {
	if e.statusCode == InternalServerErrorStatus {
		// hide message for internal server error
		return InternalServerErrorMessage
	}
	return e.message
}
