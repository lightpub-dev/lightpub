package work.tinax.lightpub;

import java.io.IOException;

import org.apache.commons.logging.Log;
import org.apache.commons.logging.LogFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.security.core.context.SecurityContextHolder;
import org.springframework.stereotype.Component;
import org.springframework.web.filter.OncePerRequestFilter;

import jakarta.servlet.FilterChain;
import jakarta.servlet.ServletException;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import work.tinax.lightpub.services.AuthApplicationService;
import work.tinax.lightpub.web.InvalidAuthException;

@Component
public class JWTFilter extends OncePerRequestFilter {

    @Autowired
    private AuthApplicationService authApplicationService;

    private Log log = LogFactory.getLog(JWTFilter.class);

    @Override
    protected void doFilterInternal(HttpServletRequest request, HttpServletResponse response, FilterChain filterChain)
            throws ServletException, IOException {
        var auth = request.getHeader("authorization");
        if (auth == null || !auth.startsWith("Bearer ")) {
            filterChain.doFilter(request, response);
            return;
        }

        var token = auth.substring(7);
        var result = authApplicationService.verifyToken(token);

        if (result.isEmpty()) {
            response.setStatus(401);
            response.setContentType("application/json");
            throw new InvalidAuthException();
        }

        var jwt = result.get();
        var jwtAuth = new JWTAuthentication(jwt);
        // TODO: check if the token is expired
        jwtAuth.setAuthenticated(true);
        SecurityContextHolder.getContext().setAuthentication(jwtAuth);
        log.info("authentication set");
        filterChain.doFilter(request, response);
    }

}
