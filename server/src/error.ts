export class LightpubException extends Error {
  constructor(public status: number, public message: string) {
    super(message);
    this.name = this.constructor.name;
  }
}

export class LightpubInternalException extends LightpubException {
  constructor() {
    super(500, "Internal Server Error");
  }
}
