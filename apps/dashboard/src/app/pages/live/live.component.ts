import { DecimalPipe } from '@angular/common';
import { Component, DestroyRef, inject, OnInit, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { catchError, of, switchMap, timer } from 'rxjs';

import { ApiService } from '../../core/api/api.service';
import { LiveStatus } from '../../core/api/models';
import { formatDuration } from '../../shared/date-query';
import { httpErrorMessage } from '../../shared/http-error';

@Component({
  selector: 'app-live',
  standalone: true,
  imports: [DecimalPipe],
  templateUrl: './live.component.html',
  styleUrl: './live.component.scss',
})
export class LiveComponent implements OnInit {
  private readonly api = inject(ApiService);
  private readonly destroyRef = inject(DestroyRef);

  readonly loading = signal(true);
  readonly error = signal<string | null>(null);
  readonly status = signal<LiveStatus | null>(null);
  readonly stale = signal(false);

  readonly formatDuration = formatDuration;

  ngOnInit(): void {
    timer(0, 2000)
      .pipe(
        switchMap(() =>
          this.api.getLiveStatus().pipe(
            catchError((err: unknown) => {
              this.error.set(httpErrorMessage(err, 'Failed to reach live status'));
              this.stale.set(true);
              return of(null);
            }),
          ),
        ),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe((status) => {
        if (status) {
          this.status.set(status);
          this.error.set(null);
          this.stale.set(false);
        }
        this.loading.set(false);
      });
  }

  headline(status: LiveStatus): string {
    return status.currentApp || status.currentSite || 'Nothing active';
  }

  secondary(status: LiveStatus): string | null {
    if (status.currentApp && status.currentSite) {
      return status.currentSite;
    }
    return null;
  }
}
