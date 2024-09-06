export class PaginatedResponse<T, Key> {
  constructor(
    private readonly pageSize: number,
    private readonly data: T[],
    private readonly keyFunc: (item: T) => Key
  ) {}

  response(): {
    result: T[];
    next: Key | null;
  } {
    if (this.data.length <= this.pageSize) {
      return {
        result: this.data,
        next: null,
      };
    }

    return {
      result: this.data.slice(0, this.pageSize),
      next: this.keyFunc(this.data[this.pageSize]),
    };
  }
}

export function parseLimit(
  limitStr: string | undefined | null,
  defaultLimit: number
): number {
  if (limitStr == null) {
    return defaultLimit;
  }

  const limit = parseInt(limitStr, 10);
  if (isNaN(limit)) {
    return defaultLimit;
  }
  return limit;
}
