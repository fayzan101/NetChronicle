import { ActivatedRoute, Router } from '@angular/router';
import { map, Observable } from 'rxjs';

/** Local calendar date as YYYY-MM-DD. */
export function todayIsoDate(): string {
  const now = new Date();
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, '0');
  const d = String(now.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

export function dateFromQuery(route: ActivatedRoute): Observable<string> {
  return route.queryParamMap.pipe(
    map((params) => {
      const raw = params.get('date');
      if (raw && /^\d{4}-\d{2}-\d{2}$/.test(raw)) {
        return raw;
      }
      return todayIsoDate();
    }),
  );
}

export function setDateQuery(
  router: Router,
  route: ActivatedRoute,
  date: string,
): void {
  void router.navigate([], {
    relativeTo: route,
    queryParams: { date },
    queryParamsHandling: 'merge',
  });
}

export function formatDuration(sec: number): string {
  if (sec < 60) {
    return `${sec}s`;
  }
  const minutes = Math.floor(sec / 60);
  if (minutes < 60) {
    return `${minutes}m`;
  }
  const hours = Math.floor(minutes / 60);
  const rem = minutes % 60;
  return rem ? `${hours}h ${rem}m` : `${hours}h`;
}

export function formatClock(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) {
    return iso;
  }
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

export function formatMinutes(minutes: number): string {
  const m = Math.round(minutes);
  if (m < 60) {
    return `${m}m`;
  }
  const hours = Math.floor(m / 60);
  const rem = m % 60;
  return rem ? `${hours}h ${rem}m` : `${hours}h`;
}

export function optionalNumber(value: number | null | undefined): string {
  if (value === null || value === undefined || Number.isNaN(value)) {
    return '—';
  }
  return String(Math.round(value * 10) / 10);
}

