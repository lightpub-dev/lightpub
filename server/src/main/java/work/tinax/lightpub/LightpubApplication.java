package work.tinax.lightpub;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

@RestController
@SpringBootApplication
public class LightpubApplication {

	public static void main(String[] args) {
		SpringApplication.run(LightpubApplication.class, args);
	}

	@RequestMapping("/")
	public String root() {
		return "hello lightpub!";
	}

}
