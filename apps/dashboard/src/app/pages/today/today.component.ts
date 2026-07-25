import { DecimalPipe } from '@angular/common';
import { Component, DestroyRef, inject, OnInit, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { catchError, of, switchMap } from 'rxjs';

import { ApiService } from '../../core/api/api.service';
import { DailyReport, LiveStatus } from '../../core/api/models';
import { dateFromQuery, setDateQuery } from '../../shared/date-query';
import { httpErrorMessage } from '../../shared/http-error';

@Component({
  selector: 'app-today',
  standalone: true,
  imports: [FormsModule, DecimalPipe],
  templateUrl: './today.component.html',
  styleUrl: './today.component.scss',
})
export class TodayComponent implements OnInit {
  private readonly api = inject(ApiService);
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private readonly destroyRef = inject(DestroyRef);

  readonly loading = signal(true);
  readonly error = signal<string | null>(null);
  readonly report = signal<DailyReport | null>(null);
  readonly live = signal<LiveStatus | null>(null);
  readonly date = signal('');

  ngOnInit(): void {
    dateFromQuery(this.route)
      .pipe(
        switchMap((date) => {
          this.date.set(date);
          this.loading.set(true);
          this.error.set(null);
          return this.api.getDailyReport(date).pipe(
            catchError((err: unknown) => {
              this.error.set(httpErrorMessage(err, 'Failed to load daily report'));
              return of(null);
            }),
          );
        }),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe((report) => {
        this.report.set(report);
        this.loading.set(false);
      });

    this.api
      .getLiveStatus()
      .pipe(
        catchError(() => of(null)),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe((live) => this.live.set(live));
  }

  onDateChange(value: string): void {
    if (!value) {
      return;
    }
    setDateQuery(this.router, this.route, value);
  }

  distractionPct(report: DailyReport): number {
    return Math.round(report.distractionRatio * 100);
  }
}
