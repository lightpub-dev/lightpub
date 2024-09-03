package work.tinax.lightpub.web;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import work.tinax.lightpub.services.UserApplicationService;

record UserRegisterResponse(String userId) {
}

record UserRegisterRequest(String username, String nickname, String password) {
}

@RestController
@RequestMapping("/auth")
public class AuthController {
	private final UserApplicationService userApplicationService;

	@Autowired
	public AuthController(UserApplicationService userApplicationService) {
		this.userApplicationService = userApplicationService;
	}

	@GetMapping("/hello")
	public String hello() {
		return "aiueo";
	}

	@PostMapping("/register")
	public UserRegisterResponse register(@RequestBody UserRegisterRequest req) {
		var result = userApplicationService.register(req.username(), req.password(), req.nickname());
		return new UserRegisterResponse(result.userId());
	}

	@PostMapping("/login")
	public UserRegisterResponse login(@RequestBody UserRegisterRequest req) {
		var result = userApplicationService.register(req.username(), req.password(), req.nickname());
		return new UserRegisterResponse(result.userId());
	}
}
