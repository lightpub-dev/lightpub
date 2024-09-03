package work.tinax.lightpub.domain.models;

import org.eclipse.jdt.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

import jakarta.transaction.Transactional;
import work.tinax.lightpub.db.repositories.UserRepository;

@Service
public class UserService {

	private final UserRepository userRepository;

	@Autowired
	public UserService(UserRepository userRepository) {
		this.userRepository = userRepository;
	}

	@Transactional
	public boolean duplicateCheck(String username, @Nullable String hostname) {
		var user = userRepository.findByUsernameAndHostname(username, hostname);
		return user != null;
	}
}
