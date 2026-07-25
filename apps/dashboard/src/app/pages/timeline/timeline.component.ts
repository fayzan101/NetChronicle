import { Component, DestroyRef, inject, OnInit, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { catchError, of, switchMap } from 'rxjs';

import { ApiService } from '../../core/api/api.service';
import { TimelineEntry } from '../../core/api/models';
import {
  dateFromQuery,
  formatClock,
  formatDuration,
  setDateQuery,
} from '../../shared/date-query';
import { httpErrorMessage } from '../../shared/http-error';

@Component({
  selector: 'app-timeline',
  standalone: true,
  imports: [FormsModule],
  templateUrl: './timeline.component.html',
  styleUrl: './timeline.component.scss',
})
export class TimelineComponent implements OnInit {
  private readonly api = inject(ApiService);
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private readonly destroyRef = inject(DestroyRef);

  readonly loading = signal(true);
  readonly error = signal<string | null>(null);
  readonly entries = signal<TimelineEntry[]>([]);
  readonly date = signal('');

  readonly formatClock = formatClock;
  readonly formatDuration = formatDuration;

  ngOnInit(): void {
    dateFromQuery(this.route)
      .pipe(
        switchMap((date) => {
          this.date.set(date);
          this.loading.set(true);
          this.error.set(null);
          return this.api.getTimeline(date).pipe(
            catchError((err: unknown) => {
              this.error.set(httpErrorMessage(err, 'Failed to load timeline'));
              return of({ date, entries: [] as TimelineEntry[] });
            }),
          );
        }),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe((response) => {
        this.entries.set(response.entries ?? []);
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
