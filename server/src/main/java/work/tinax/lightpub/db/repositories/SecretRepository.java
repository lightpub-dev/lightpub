package work.tinax.lightpub.db.repositories;

import org.springframework.data.repository.CrudRepository;

import work.tinax.lightpub.db.models.DBSecret;

public interface SecretRepository extends CrudRepository<DBSecret, String> {

}
