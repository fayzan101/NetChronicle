import { Component, DestroyRef, inject, OnInit, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { catchError, of, switchMap } from 'rxjs';

import { ApiService } from '../../core/api/api.service';
import { InsightItem } from '../../core/api/models';
import { dateFromQuery, setDateQuery } from '../../shared/date-query';
import { httpErrorMessage } from '../../shared/http-error';

@Component({
  selector: 'app-insights',
  standalone: true,
  imports: [FormsModule],
  templateUrl: './insights.component.html',
  styleUrl: './insights.component.scss',
})
export class InsightsComponent implements OnInit {
  private readonly api = inject(ApiService);
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private readonly destroyRef = inject(DestroyRef);

  readonly loading = signal(true);
  readonly error = signal<string | null>(null);
  readonly insights = signal<InsightItem[]>([]);
  readonly date = signal('');

  ngOnInit(): void {
    dateFromQuery(this.route)
      .pipe(
        switchMap((date) => {
          this.date.set(date);
          this.loading.set(true);
          this.error.set(null);
          return this.api.getInsights(date).pipe(
            catchError((err: unknown) => {
              this.error.set(httpErrorMessage(err, 'Failed to load insights'));
              return of({ insights: [] as InsightItem[] });
            }),
          );
        }),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe((response) => {
        this.insights.set(response.insights ?? []);
        this.loading.set(false);
      });
  }

  onDateChange(value: string): void {
    if (!value) {
      return;
    }
    setDateQuery(this.router, this.route, value);
  }
}
