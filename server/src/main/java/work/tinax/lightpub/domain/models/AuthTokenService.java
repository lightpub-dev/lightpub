package work.tinax.lightpub.domain.models;

import org.springframework.stereotype.Service;

import work.tinax.lightpub.db.models.DBAuthToken;
import work.tinax.lightpub.db.models.DBUser;
import work.tinax.lightpub.utils.UuidUtils;

@Service
public class AuthTokenService {
    public DBAuthToken toDB(AuthToken authToken) {
        var user = new DBUser();
        var userId = UuidUtils.trim(authToken.getUserId().getId());
        user.setUserId(userId);

        var d = new DBAuthToken();
        d.setToken(authToken.getToken());
        d.setCreatedAt(authToken.getCreatedAt());
        d.setUser(user);
        return d;
    }
}
