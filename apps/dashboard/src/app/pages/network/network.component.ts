import { DecimalPipe } from '@angular/common';
import { Component, DestroyRef, inject, OnInit, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { catchError, forkJoin, of, switchMap } from 'rxjs';

import { ApiService } from '../../core/api/api.service';
import {
  NetworkEvent,
  NetworkStatsResponse,
} from '../../core/api/models';
import {
  dateFromQuery,
  formatClock,
  optionalNumber,
  setDateQuery,
} from '../../shared/date-query';
import { httpErrorMessage } from '../../shared/http-error';

@Component({
  selector: 'app-network',
  standalone: true,
  imports: [FormsModule, DecimalPipe],
  templateUrl: './network.component.html',
  styleUrl: './network.component.scss',
})
export class NetworkComponent implements OnInit {
  private readonly api = inject(ApiService);
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private readonly destroyRef = inject(DestroyRef);

  readonly loading = signal(true);
  readonly error = signal<string | null>(null);
  readonly stats = signal<NetworkStatsResponse | null>(null);
  readonly events = signal<NetworkEvent[]>([]);
  readonly date = signal('');
  readonly maxLatency = signal(1);

  readonly formatClock = formatClock;
  readonly optionalNumber = optionalNumber;

  ngOnInit(): void {
    dateFromQuery(this.route)
      .pipe(
        switchMap((date) => {
          this.date.set(date);
          this.loading.set(true);
          this.error.set(null);
          return forkJoin({
            stats: this.api.getNetworkStats(date).pipe(
              catchError((err: unknown) => {
                this.error.set(
                  httpErrorMessage(err, 'Failed to load network stats'),
                );
                return of(null);
              }),
            ),
            events: this.api.getNetworkEvents(date).pipe(
              catchError(() => of({ events: [] as NetworkEvent[] })),
            ),
          });
        }),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe(({ stats, events }) => {
        this.stats.set(stats);
        this.events.set(events?.events ?? []);
        const peak = Math.max(
          1,
          ...(stats?.samples ?? [])
            .map((s) => s.latencyMs ?? 0)
            .filter((n) => n > 0),
        );
        this.maxLatency.set(peak);
        this.loading.set(false);
      });
  }

  onDateChange(value: string): void {
    if (!value) {
      return;
    }
    setDateQuery(this.router, this.route, value);
  }

  barHeight(latencyMs: number | null): number {
    if (latencyMs === null || latencyMs <= 0) {
      return 4;
    }
    return Math.max(6, Math.round((latencyMs / this.maxLatency()) * 100));
  }
}
