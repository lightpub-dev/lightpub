package work.tinax.lightpub.db.repositories;

import org.springframework.data.repository.CrudRepository;

import work.tinax.lightpub.db.models.DBAuthToken;

public interface AuthTokenRepository extends CrudRepository<DBAuthToken, String> {
    DBAuthToken findByToken(String token);
}
