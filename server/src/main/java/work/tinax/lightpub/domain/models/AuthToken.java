package work.tinax.lightpub.domain.models;

import java.time.LocalDateTime;

public class AuthToken {
    private String token;
    private LocalDateTime createdAt;
    private UserId userId;

    public AuthToken(String token, LocalDateTime createdAt, UserId userId) {
        this.token = token;
        this.createdAt = createdAt;
        this.userId = userId;
    }

    public String getToken() {
        return token;
    }

    public void setToken(String token) {
        this.token = token;
    }

    public LocalDateTime getCreatedAt() {
        return createdAt;
    }

    public void setCreatedAt(LocalDateTime createdAt) {
        this.createdAt = createdAt;
    }

    public UserId getUserId() {
        return userId;
    }

    public void setUserId(UserId userId) {
        this.userId = userId;
    }

}
