package work.tinax.lightpub.web;

import org.springframework.web.bind.annotation.ExceptionHandler;
import org.springframework.web.bind.annotation.RestControllerAdvice;

import jakarta.servlet.http.HttpServletResponse;

record ErrorResponse(String message) {
}

/**
 * ErrorController
 */
@RestControllerAdvice
public class ErrorController {
    @ExceptionHandler(LightpubException.class)
    public ErrorResponse handleLightpubException(LightpubException e, HttpServletResponse response) {
        response.setStatus(e.getStatus());
        return new ErrorResponse(e.getText());
    }
}