import { Injectable, computed, signal } from '@angular/core';

import { AuthResponse, AuthSession } from './models';

const STORAGE_KEY = 'netchronicle.auth';

@Injectable({ providedIn: 'root' })
export class AuthService {
  private readonly sessionSignal = signal<AuthSession | null>(this.readStorage());

  readonly session = this.sessionSignal.asReadonly();
  readonly isSignedIn = computed(() => !!this.sessionSignal()?.token);
  readonly displayName = computed(
    () => this.sessionSignal()?.displayName || this.sessionSignal()?.email || null,
  );

  token(): string | null {
    return this.sessionSignal()?.token ?? null;
  }

  setSession(auth: AuthResponse): void {
    const session: AuthSession = {
      userId: auth.userId,
      email: auth.email,
      displayName: auth.displayName,
      token: auth.token,
      expiresAt: auth.expiresAt,
    };
    this.sessionSignal.set(session);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(session));
  }

  clear(): void {
    this.sessionSignal.set(null);
    localStorage.removeItem(STORAGE_KEY);
  }

  private readStorage(): AuthSession | null {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) {
        return null;
      }
      const parsed = JSON.parse(raw) as AuthSession;
      if (!parsed?.token) {
        return null;
      }
      return parsed;
    } catch {
      return null;
    }
  }
}
