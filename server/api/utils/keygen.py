from dataclasses import dataclass

from Crypto.PublicKey import RSA


@dataclass
class KeyPair:
    private_key: str
    public_key: str


def generate_key_pair() -> KeyPair:
    key = RSA.generate(4096)
    private_key = key.exportKey(pkcs=8).decode("utf-8")
    public_key = key.publickey().exportKey(pkcs=8).decode("utf-8")

    return KeyPair(private_key=private_key, public_key=public_key)
