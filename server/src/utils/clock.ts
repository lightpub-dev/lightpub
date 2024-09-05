export function now(): number {
  return Date.now();
}

export function clockNow(): Clock {
  return new Clock();
}

export class Clock extends Date {
  asNumber() {
    return this.getTime();
  }
}
