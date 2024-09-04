package work.tinax.lightpub.domain.models.factory;

import work.tinax.lightpub.domain.models.AuthToken;
import work.tinax.lightpub.domain.models.UserId;

public interface AuthTokenFactory {
    AuthToken generateNow(UserId userId);
}
