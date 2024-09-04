package work.tinax.lightpub.services;

import java.time.LocalDateTime;

public record JWTContent(String userId, LocalDateTime issuedAt) {
}
