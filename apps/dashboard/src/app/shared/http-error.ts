import { HttpErrorResponse } from '@angular/common/http';

export function httpErrorMessage(err: unknown, fallback: string): string {
  if (err instanceof HttpErrorResponse) {
    if (err.status === 0) {
      return 'Cannot reach API. Is netchronicle-api running on localhost:8080?';
    }
    return err.error?.message || err.message || fallback;
  }
  if (err instanceof Error) {
    return err.message;
  }
  return fallback;
}
