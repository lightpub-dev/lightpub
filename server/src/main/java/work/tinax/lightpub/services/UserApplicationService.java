package work.tinax.lightpub.services;

import java.util.Objects;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

import jakarta.transaction.Transactional;
import work.tinax.lightpub.db.models.DBUser;
import work.tinax.lightpub.db.repositories.UserRepository;
import work.tinax.lightpub.domain.models.UserService;
import work.tinax.lightpub.domain.models.factory.UserFactory;

@Service
public class UserApplicationService {

	private final UserService userService;
	private final UserRepository userRepository;
	private final UserFactory userFactory;

	@Autowired
	public UserApplicationService(UserService userService, UserRepository userRepository, UserFactory userFactory) {
		this.userService = userService;
		this.userRepository = userRepository;
		this.userFactory = userFactory;
	}

	@Transactional
	public UserRegisterResult register(String username, String plainPassword, String nickname) {
		var newUser = userFactory.create(username, nickname, plainPassword);

		// duplication check
		if (userService.duplicateCheck(username, null)) {
			throw new RuntimeException("user duplicated");
		}

		userRepository.save(new DBUser());

		return new UserRegisterResult(Objects.requireNonNull(newUser.getId().getId().toString().replace("-", "")));
	}
}
