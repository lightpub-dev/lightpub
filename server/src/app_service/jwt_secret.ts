import { inject, injectable } from "tsyringe";
import { SECRET_REPOSITORY } from "../registry_key";
import { type ISecretRepository } from "../repository/secret";
import NodeRSA = require("node-rsa");

const privateKeyKey = "private_key";
const publicKeyKey = "public_key";

@injectable()
export class JWTSecretProvider {
  constructor(
    @inject(SECRET_REPOSITORY) private secretRepository: ISecretRepository
  ) {}

  private async generateKey(): Promise<{
    private: string;
    public: string;
  }> {
    const key = new NodeRSA({ b: 2048 });
    const privKey = key.exportKey("pkcs8-private-pem");
    const pubKey = key.exportKey("pkcs8-public-pem");
    await this.secretRepository.setSecret(privateKeyKey, privKey);
    await this.secretRepository.setSecret(publicKeyKey, pubKey);
    return {
      private: privKey,
      public: pubKey,
    };
  }

  async privateKey(): Promise<string> {
    const fromDB = await this.secretRepository.getSecret(privateKeyKey);
    if (fromDB !== null) return fromDB;

    return (await this.generateKey()).private;
  }

  async publicKey(): Promise<string> {
    const fromDB = await this.secretRepository.getSecret(publicKeyKey);
    if (fromDB !== null) return fromDB;

    return (await this.generateKey()).public;
  }
}
