export interface PaginatedResponse<T> {
  result: T[];
  next: string | null;
}
