/*
Lightpub: An activitypub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

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
