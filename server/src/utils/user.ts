import { UserSpec } from "../app_service/user";
import { LightpubException } from "../error";

export class BadUserSpecFormat extends LightpubException {
  constructor() {
    super(400, "Bad user spec format");
  }
}

export function parseUserspec(spec: string): UserSpec {
  if (!spec.includes("@")) {
    return { userId: spec };
  }

  if (!spec.startsWith("@")) {
    throw new BadUserSpecFormat();
  }
  const sp = spec.split("@");
  if (sp.length === 2) {
    return { username: sp[1], hostname: null };
  } else if (sp.length === 3) {
    return { username: sp[1], hostname: sp[2] };
  }

  throw new BadUserSpecFormat();
}
