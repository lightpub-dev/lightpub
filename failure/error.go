package failure

const (
	InternalServerErrorStatus  = 500
	InternalServerErrorMessage = "internal server error"
)

type ErrorResponse interface {
	error
	StatusCode() int
	Message() string
}

// LightpubError is a custom error type that can be used to return an error with a status code.
type LightpubError struct {
	cause      error
	statusCode int
	message    string
}

func NewError(statusCode int, message string) *LightpubError {
	return &LightpubError{
		statusCode: statusCode,
		message:    message,
	}
}

func NewErrorWithCause(statusCode int, message string, cause error) *LightpubError {
	return &LightpubError{
		cause:      cause,
		statusCode: statusCode,
		message:    message,
	}
}

func NewInternalServerError(msg string) *LightpubError {
	return NewError(InternalServerErrorStatus, msg)
}

func NewInternalServerErrorWithCause(msg string, cause error) *LightpubError {
	return NewErrorWithCause(InternalServerErrorStatus, msg, cause)
}

func (e *LightpubError) Error() string {
	s := e.message
	if e.cause != nil {
		s += ": " + e.cause.Error()
	}
	return s
}

func (e *LightpubError) StatusCode() int {
	return e.statusCode
}

func (e *LightpubError) Cause() error {
	return e.cause
}

func (e *LightpubError) Message() string {
	if e.statusCode == InternalServerErrorStatus {
		// hide message for internal server error
		return InternalServerErrorMessage
	}
	return e.message
}
