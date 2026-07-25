import { Component, DestroyRef, OnInit, inject, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { catchError, of } from 'rxjs';

import { ApiService } from '../../core/api/api.service';
import { AuthService } from '../../core/api/auth.service';
import {
  ApiKeyItem,
  DeviceItem,
  UserSettings,
} from '../../core/api/models';
import { formatClock } from '../../shared/date-query';
import { httpErrorMessage } from '../../shared/http-error';

@Component({
  selector: 'app-settings',
  standalone: true,
  imports: [FormsModule],
  templateUrl: './settings.component.html',
  styleUrl: './settings.component.scss',
})
export class SettingsComponent implements OnInit {
  private readonly api = inject(ApiService);
  private readonly destroyRef = inject(DestroyRef);
  readonly auth = inject(AuthService);

  readonly formatClock = formatClock;

  // Account
  email = '';
  password = '';
  displayName = '';
  readonly authMode = signal<'login' | 'register'>('login');
  readonly authBusy = signal(false);
  readonly authError = signal<string | null>(null);
  readonly authNotice = signal<string | null>(null);

  // Settings
  readonly settingsLoading = signal(false);
  readonly settingsError = signal<string | null>(null);
  readonly settingsSaved = signal(false);
  readonly settings = signal<UserSettings | null>(null);
  trackingEnabled = true;
  pollIntervalSecs: number | null = null;
  idleThresholdSecs: number | null = null;
  networkSampleIntervalSecs: number | null = null;
  privacyHideTitles = false;
  privacyHideUrls = false;

  // API keys
  readonly keys = signal<ApiKeyItem[]>([]);
  readonly keysError = signal<string | null>(null);
  readonly createdKey = signal<string | null>(null);
  newKeyName = '';

  // Devices
  readonly devices = signal<DeviceItem[]>([]);
  readonly devicesError = signal<string | null>(null);
  deviceAgentId = '';
  deviceName = '';

  // Privacy
  readonly privacyNotice = signal<string | null>(null);
  readonly privacyError = signal<string | null>(null);
  wipeConfirm = false;
  readonly wipeBusy = signal(false);

  ngOnInit(): void {
    this.refreshPanels();
  }

  setAuthMode(mode: 'login' | 'register'): void {
    this.authMode.set(mode);
    this.authError.set(null);
    this.authNotice.set(null);
  }

  submitAuth(): void {
    this.authBusy.set(true);
    this.authError.set(null);
    this.authNotice.set(null);

    const request =
      this.authMode() === 'login'
        ? this.api.login(this.email.trim(), this.password)
        : this.api.register(
            this.email.trim(),
            this.password,
            this.displayName.trim() || undefined,
          );

    request.pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (session) => {
        this.auth.setSession(session);
        this.password = '';
        this.authBusy.set(false);
        this.authNotice.set(
          this.authMode() === 'login' ? 'Signed in.' : 'Account created.',
        );
        this.refreshPanels();
      },
      error: (err: unknown) => {
        this.authBusy.set(false);
        this.authError.set(httpErrorMessage(err, 'Auth failed'));
      },
    });
  }

  signOut(): void {
    this.auth.clear();
    this.authNotice.set('Signed out. Local API fallback still works when AUTH_REQUIRED=false.');
    this.settings.set(null);
    this.keys.set([]);
    this.devices.set([]);
    this.createdKey.set(null);
  }

  saveSettings(): void {
    this.settingsSaved.set(false);
    this.settingsError.set(null);
    this.api
      .patchSettings({
        trackingEnabled: this.trackingEnabled,
        pollIntervalSecs: this.toOptionalNumber(this.pollIntervalSecs),
        idleThresholdSecs: this.toOptionalNumber(this.idleThresholdSecs),
        networkSampleIntervalSecs: this.toOptionalNumber(
          this.networkSampleIntervalSecs,
        ),
        privacyHideTitles: this.privacyHideTitles,
        privacyHideUrls: this.privacyHideUrls,
      })
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: (updated) => {
          this.applySettings(updated);
          this.settingsSaved.set(true);
        },
        error: (err: unknown) => {
          this.settingsError.set(httpErrorMessage(err, 'Failed to save settings'));
        },
      });
  }

  createKey(): void {
    this.keysError.set(null);
    this.createdKey.set(null);
    this.api
      .createApiKey(this.newKeyName.trim() || undefined)
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: (created) => {
          this.createdKey.set(created.apiKey);
          this.newKeyName = '';
          this.loadKeys();
        },
        error: (err: unknown) => {
          this.keysError.set(httpErrorMessage(err, 'Failed to create API key'));
        },
      });
  }

  revokeKey(id: string): void {
    this.api
      .revokeApiKey(id)
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: () => this.loadKeys(),
        error: (err: unknown) => {
          this.keysError.set(httpErrorMessage(err, 'Failed to revoke key'));
        },
      });
  }

  registerDevice(): void {
    this.devicesError.set(null);
    this.api
      .registerDevice(this.deviceAgentId.trim(), this.deviceName.trim() || undefined)
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: () => {
          this.deviceAgentId = '';
          this.deviceName = '';
          this.loadDevices();
        },
        error: (err: unknown) => {
          this.devicesError.set(httpErrorMessage(err, 'Failed to register device'));
        },
      });
  }

  exportActivity(format: 'json' | 'csv'): void {
    this.privacyError.set(null);
    this.privacyNotice.set(null);
    this.api
      .exportActivity(format)
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: (response) => {
          const blob = response.body;
          if (!blob) {
            this.privacyError.set('Empty export response');
            return;
          }
          const url = URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = `netchronicle-export.${format}`;
          a.click();
          URL.revokeObjectURL(url);
          this.privacyNotice.set(`Downloaded ${format.toUpperCase()} export.`);
        },
        error: (err: unknown) => {
          this.privacyError.set(httpErrorMessage(err, 'Export failed'));
        },
      });
  }

  wipeData(): void {
    if (!this.wipeConfirm) {
      this.privacyError.set('Check the confirmation box first.');
      return;
    }
    this.wipeBusy.set(true);
    this.privacyError.set(null);
    this.privacyNotice.set(null);
    this.api
      .getDeleteToken()
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: (token) => {
          this.api
            .deleteData(token.confirmation)
            .pipe(takeUntilDestroyed(this.destroyRef))
            .subscribe({
              next: (result) => {
                this.wipeBusy.set(false);
                this.wipeConfirm = false;
                this.privacyNotice.set(
                  `Wiped activity data (${result.deletedRows} rows).`,
                );
              },
              error: (err: unknown) => {
                this.wipeBusy.set(false);
                this.privacyError.set(httpErrorMessage(err, 'Wipe failed'));
              },
            });
        },
        error: (err: unknown) => {
          this.wipeBusy.set(false);
          this.privacyError.set(httpErrorMessage(err, 'Could not get delete token'));
        },
      });
  }

  private refreshPanels(): void {
    this.loadSettings();
    this.loadKeys();
    this.loadDevices();
  }

  private loadSettings(): void {
    this.settingsLoading.set(true);
    this.settingsError.set(null);
    this.api
      .getSettings()
      .pipe(
        catchError((err: unknown) => {
          this.settingsError.set(httpErrorMessage(err, 'Failed to load settings'));
          return of(null);
        }),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe((settings) => {
        this.settingsLoading.set(false);
        if (settings) {
          this.applySettings(settings);
        }
      });
  }

  private applySettings(settings: UserSettings): void {
    this.settings.set(settings);
    this.trackingEnabled = settings.trackingEnabled;
    this.pollIntervalSecs = settings.pollIntervalSecs;
    this.idleThresholdSecs = settings.idleThresholdSecs;
    this.networkSampleIntervalSecs = settings.networkSampleIntervalSecs;
    this.privacyHideTitles = settings.privacyHideTitles;
    this.privacyHideUrls = settings.privacyHideUrls;
  }

  private loadKeys(): void {
    this.api
      .listApiKeys()
      .pipe(
        catchError((err: unknown) => {
          this.keysError.set(httpErrorMessage(err, 'Failed to load API keys'));
          return of([] as ApiKeyItem[]);
        }),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe((keys) => this.keys.set(keys));
  }

  private loadDevices(): void {
    this.api
      .listDevices()
      .pipe(
        catchError((err: unknown) => {
          this.devicesError.set(httpErrorMessage(err, 'Failed to load devices'));
          return of([] as DeviceItem[]);
        }),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe((devices) => this.devices.set(devices));
  }

  private toOptionalNumber(value: number | null | string): number | null {
    if (value === null || value === undefined || value === '') {
      return null;
    }
    const n = typeof value === 'number' ? value : Number(value);
    return Number.isFinite(n) ? n : null;
  }
}
