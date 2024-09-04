package work.tinax.lightpub;

import java.util.Collection;
import java.util.List;

import org.eclipse.jdt.annotation.NonNull;
import org.springframework.security.core.Authentication;
import org.springframework.security.core.GrantedAuthority;

import work.tinax.lightpub.services.JWTContent;

public class JWTAuthentication implements Authentication {

    private boolean authenticated = false;

    @NonNull
    private JWTContent jwt;

    public JWTAuthentication(@NonNull JWTContent jwt) {
        this.jwt = jwt;
    }

    @Override
    public String getName() {
        return jwt.userId();
    }

    @Override
    public Collection<? extends GrantedAuthority> getAuthorities() {
        return List.of();
    }

    @Override
    public Object getCredentials() {
        return null;
    }

    @Override
    public Object getDetails() {
        return jwt;
    }

    @Override
    public Object getPrincipal() {
        return null;
    }

    @Override
    public boolean isAuthenticated() {
        return authenticated;
    }

    @Override
    public void setAuthenticated(boolean isAuthenticated) throws IllegalArgumentException {
        this.authenticated = isAuthenticated;
    }

}
