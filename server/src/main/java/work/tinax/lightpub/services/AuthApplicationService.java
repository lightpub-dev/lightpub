package work.tinax.lightpub.services;

import java.security.KeyFactory;
import java.security.KeyPairGenerator;
import java.security.NoSuchAlgorithmException;
import java.security.interfaces.RSAPrivateKey;
import java.security.interfaces.RSAPublicKey;
import java.security.spec.InvalidKeySpecException;
import java.security.spec.PKCS8EncodedKeySpec;
import java.security.spec.X509EncodedKeySpec;
import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.time.ZonedDateTime;
import java.util.List;
import java.util.Optional;

import org.apache.commons.logging.Log;
import org.apache.commons.logging.LogFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

import com.auth0.jwt.JWT;
import com.auth0.jwt.algorithms.Algorithm;
import com.auth0.jwt.exceptions.JWTCreationException;
import com.auth0.jwt.exceptions.JWTVerificationException;
import com.auth0.jwt.interfaces.DecodedJWT;
import com.auth0.jwt.interfaces.JWTVerifier;

import jakarta.transaction.Transactional;
import work.tinax.lightpub.db.models.DBSecret;
import work.tinax.lightpub.db.repositories.SecretRepository;

@Service
public class AuthApplicationService {
    @Autowired
    private SecretRepository secretRepository;

    private static final String PRIVATE_KEY = "privateKey";
    private static final String PUBLIC_KEY = "publicKey";

    record KeyPair(RSAPrivateKey privateKey, RSAPublicKey publicKey) {
    }

    private Optional<RSAPrivateKey> loadPrivateKey() {
        var secret = secretRepository.findById(PRIVATE_KEY);
        if (!secret.isEmpty()) {
            // deserialize into RSAPrivateKey
            var value = secret.get().getValue();
            try {
                KeyFactory keyFactory = KeyFactory.getInstance("RSA");
                return Optional.of((RSAPrivateKey) keyFactory.generatePrivate(new PKCS8EncodedKeySpec(value)));
            } catch (NoSuchAlgorithmException | InvalidKeySpecException e) {
                e.printStackTrace();
                throw new RuntimeException(e);
            }
        }
        return Optional.empty();
    }

    private Optional<RSAPublicKey> loadPublicKey() {
        var secret = secretRepository.findById(PUBLIC_KEY);
        if (!secret.isEmpty()) {
            // deserialize into RSAPublicKey
            var value = secret.get().getValue();
            try {
                KeyFactory keyFactory = KeyFactory.getInstance("RSA");
                return Optional.of((RSAPublicKey) keyFactory.generatePublic(new X509EncodedKeySpec(value)));
            } catch (NoSuchAlgorithmException | InvalidKeySpecException e) {
                e.printStackTrace();
                throw new RuntimeException(e);
            }
        }
        return Optional.empty();
    }

    @Transactional
    private KeyPair getKeyPair() {
        // check if the keypair is in the database
        var privateKeyDB = loadPrivateKey();
        var publicKeyDB = loadPublicKey();
        if (privateKeyDB.isPresent() && publicKeyDB.isPresent()) {
            return new KeyPair(privateKeyDB.get(), publicKeyDB.get());
        }

        // generate a new private key
        // delete the old public key if exists
        secretRepository.deleteById(PUBLIC_KEY);
        secretRepository.deleteById(PRIVATE_KEY);
        KeyPairGenerator gen;
        try {
            gen = KeyPairGenerator.getInstance("RSA");
            gen.initialize(2048);
            var keyPair = gen.generateKeyPair();
            var privateKey = keyPair.getPrivate();
            var publicKey = keyPair.getPublic();
            // save the keys
            var privateKeyEntry = new DBSecret(PRIVATE_KEY, privateKey.getEncoded());
            var publicKeyEntry = new DBSecret(PUBLIC_KEY, publicKey.getEncoded());
            secretRepository.saveAll(List.of(privateKeyEntry, publicKeyEntry));
            return new KeyPair((RSAPrivateKey) privateKey, (RSAPublicKey) publicKey);
        } catch (NoSuchAlgorithmException e) {
            e.printStackTrace();
            throw new RuntimeException(e);
        }
    }

    private Algorithm getAlgorithm() {
        var keyPair = getKeyPair();
        return Algorithm.RSA256(keyPair.publicKey(), keyPair.privateKey());
    }

    public String createToken(String userId) {
        var algorithm = getAlgorithm();
        // sign the userId
        try {
            var issueTime = ZonedDateTime.now().toEpochSecond();
            String token = JWT.create()
                    .withIssuer("lightpub")
                    .withClaim("sub", userId)
                    .withClaim("iat", issueTime)
                    .sign(algorithm);
            return token;
        } catch (JWTCreationException exception) {
            // Invalid Signing configuration / Couldn't convert Claims.
            throw new RuntimeException("failed to create JWT", exception);
        }
    }

    private Log log = LogFactory.getLog(AuthApplicationService.class);

    public Optional<JWTContent> verifyToken(String token) {
        // log.info("receved token: " + token);
        var algorithm = getAlgorithm();
        DecodedJWT decodedJWT;
        try {
            JWTVerifier verifier = JWT.require(algorithm)
                    // specify any specific claim validations
                    .withIssuer("lightpub")
                    .withClaimPresence("sub")
                    .withClaimPresence("iat")
                    // reusable verifier instance
                    .build();

            decodedJWT = verifier.verify(token);
        } catch (JWTVerificationException exception) {
            // Invalid signature/claims
            // throw new RuntimeException("failed to verify JWT", exception);
            exception.printStackTrace();
            return Optional.empty();
        }

        if (decodedJWT == null) {
            return Optional.empty();
        }

        var userId = decodedJWT.getClaim("sub").asString();
        if (userId == null) {
            throw new RuntimeException("failed to get sub from JWT");
        }
        var iat = decodedJWT.getClaim("iat").asLong();
        if (iat == null) {
            throw new RuntimeException("failed to get iat from JWT");
        }
        var issuedAt = LocalDateTime.ofEpochSecond(iat, 0, ZoneOffset.UTC);

        return Optional.of(new JWTContent(userId, issuedAt));
    }
}