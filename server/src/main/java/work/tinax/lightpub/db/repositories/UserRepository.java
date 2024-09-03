package work.tinax.lightpub.db.repositories;

import org.eclipse.jdt.annotation.NonNull;
import org.springframework.data.repository.CrudRepository;
import org.springframework.transaction.annotation.Transactional;

import work.tinax.lightpub.db.models.DBUser;

@Transactional(readOnly = true)
public interface UserRepository extends CrudRepository<DBUser, String> {
	DBUser findByUsernameAndHostname(@NonNull String username, String hostname);
}
