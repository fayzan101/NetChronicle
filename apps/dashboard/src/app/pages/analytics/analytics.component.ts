import { DecimalPipe } from '@angular/common';
import { Component, DestroyRef, computed, inject, OnInit, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { catchError, forkJoin, of, switchMap } from 'rxjs';

import { ApiService } from '../../core/api/api.service';
import {
  HourBucket,
  NamedMinutes,
  ReportResponse,
  SessionItem,
} from '../../core/api/models';
import {
  dateFromQuery,
  formatClock,
  formatDuration,
  formatMinutes,
  setDateQuery,
} from '../../shared/date-query';
import { httpErrorMessage } from '../../shared/http-error';

@Component({
  selector: 'app-analytics',
  standalone: true,
  imports: [FormsModule, DecimalPipe],
  templateUrl: './analytics.component.html',
  styleUrl: './analytics.component.scss',
})
export class AnalyticsComponent implements OnInit {
  private readonly api = inject(ApiService);
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private readonly destroyRef = inject(DestroyRef);

  readonly loading = signal(true);
  readonly error = signal<string | null>(null);
  readonly sessions = signal<SessionItem[]>([]);
  readonly report = signal<ReportResponse | null>(null);
  readonly date = signal('');

  readonly formatClock = formatClock;
  readonly formatDuration = formatDuration;
  readonly formatMinutes = formatMinutes;

  readonly avgScore = computed(() => {
    const scores = this.sessions()
      .map((s) => s.productivityScore)
      .filter((n): n is number => n !== null && n !== undefined);
    if (!scores.length) {
      return null;
    }
    return scores.reduce((a, b) => a + b, 0) / scores.length;
  });

  readonly categoryBars = computed(() => {
    const fromReport = this.report()?.summary?.categoryMinutes;
    if (fromReport?.length) {
      return this.normalizeBars(
        fromReport.map((c) => ({
          label: c.category || c.app || c.domain || 'other',
          minutes: c.minutes,
        })),
      );
    }
    const map = new Map<string, number>();
    for (const session of this.sessions()) {
      const start = new Date(session.startTime).getTime();
      const end = session.endTime
        ? new Date(session.endTime).getTime()
        : start;
      const minutes = Math.max(1, Math.round((end - start) / 60000));
      const key = session.category || 'uncategorized';
      map.set(key, (map.get(key) ?? 0) + minutes);
    }
    return this.normalizeBars(
      [...map.entries()].map(([label, minutes]) => ({ label, minutes })),
    );
  });

  readonly timeOfDay = computed((): HourBucket[] => {
    return this.report()?.summary?.timeOfDay ?? [];
  });

  readonly topApps = computed((): NamedMinutes[] => {
    const fromReport = this.report()?.summary?.topApps;
    if (fromReport?.length) {
      return fromReport;
    }
    const map = new Map<string, number>();
    for (const session of this.sessions()) {
      for (const app of session.primaryApps) {
        map.set(app, (map.get(app) ?? 0) + 1);
      }
    }
    return [...map.entries()]
      .sort((a, b) => b[1] - a[1])
      .slice(0, 8)
      .map(([app, minutes]) => ({ app, minutes }));
  });

  readonly maxHour = computed(() =>
    Math.max(1, ...this.timeOfDay().map((h) => h.totalMinutes)),
  );

  ngOnInit(): void {
    dateFromQuery(this.route)
      .pipe(
        switchMap((date) => {
          this.date.set(date);
          this.loading.set(true);
          this.error.set(null);
          return forkJoin({
            sessions: this.api.getSessions(date).pipe(
              catchError((err: unknown) => {
                this.error.set(
                  httpErrorMessage(err, 'Failed to load sessions'),
                );
                return of({ sessions: [] as SessionItem[], limit: 0, offset: 0 });
              }),
            ),
            report: this.api.getReport('daily', date).pipe(
              catchError(() => of(null)),
            ),
          });
        }),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe(({ sessions, report }) => {
        this.sessions.set(sessions.sessions ?? []);
        this.report.set(report);
        this.loading.set(false);
      });
  }

  onDateChange(value: string): void {
    if (!value) {
      return;
    }
    setDateQuery(this.router, this.route, value);
  }

  private normalizeBars(
    items: { label: string; minutes: number }[],
  ): { label: string; minutes: number; pct: number }[] {
    const max = Math.max(1, ...items.map((i) => i.minutes));
    return items
      .slice()
      .sort((a, b) => b.minutes - a.minutes)
      .map((i) => ({
        ...i,
        pct: Math.round((i.minutes / max) * 100),
      }));
  }
}
