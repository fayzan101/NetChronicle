import { DecimalPipe } from '@angular/common';
import { Component, DestroyRef, inject, OnInit, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { catchError, combineLatest, map, of, switchMap } from 'rxjs';

import { ApiService } from '../../core/api/api.service';
import {
  ReportListItem,
  ReportResponse,
  ReportType,
} from '../../core/api/models';
import {
  dateFromQuery,
  formatMinutes,
  setDateQuery,
  todayIsoDate,
} from '../../shared/date-query';
import { httpErrorMessage } from '../../shared/http-error';

@Component({
  selector: 'app-reports',
  standalone: true,
  imports: [FormsModule, DecimalPipe],
  templateUrl: './reports.component.html',
  styleUrl: './reports.component.scss',
})
export class ReportsComponent implements OnInit {
  private readonly api = inject(ApiService);
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private readonly destroyRef = inject(DestroyRef);

  readonly loading = signal(true);
  readonly error = signal<string | null>(null);
  readonly report = signal<ReportResponse | null>(null);
  readonly cached = signal<ReportListItem[]>([]);
  readonly date = signal(todayIsoDate());
  readonly reportType = signal<ReportType>('weekly');

  readonly types: ReportType[] = ['daily', 'weekly', 'monthly'];
  readonly formatMinutes = formatMinutes;

  ngOnInit(): void {
    combineLatest([
      dateFromQuery(this.route),
      this.route.queryParamMap.pipe(
        map((params) => {
          const raw = params.get('reportType');
          if (raw === 'daily' || raw === 'weekly' || raw === 'monthly') {
            return raw;
          }
          return 'weekly' as ReportType;
        }),
      ),
    ])
      .pipe(
        switchMap(([date, reportType]) => {
          this.date.set(date);
          this.reportType.set(reportType);
          this.loading.set(true);
          this.error.set(null);
          return this.api.getReport(reportType, date).pipe(
            catchError((err: unknown) => {
              this.error.set(httpErrorMessage(err, 'Failed to load report'));
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
      .listReports()
      .pipe(
        catchError(() => of({ reports: [] as ReportListItem[] })),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe((list) => this.cached.set(list.reports ?? []));
  }

  onDateChange(value: string): void {
    if (!value) {
      return;
    }
    setDateQuery(this.router, this.route, value);
  }

  onTypeChange(value: ReportType): void {
    void this.router.navigate([], {
      relativeTo: this.route,
      queryParams: { reportType: value },
      queryParamsHandling: 'merge',
    });
  }

  export(format: 'json' | 'csv'): void {
    const url = this.api.exportReportUrl(
      format,
      this.reportType(),
      this.date(),
    );
    window.open(url, '_blank', 'noopener');
  }

  scoreLabel(report: ReportResponse): string {
    const s = report.summary;
    const value =
      s.averageProductivityScore ?? s.productivityScore ?? null;
    return value === null ? '—' : String(Math.round(value));
  }

  createdDay(iso: string): string {
    return iso.slice(0, 10);
  }
}
