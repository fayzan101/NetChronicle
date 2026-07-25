import { HttpClient, HttpParams } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../../../environments/environment';
import {
  DailyReport,
  LiveStatus,
  TimelineResponse,
} from './models';

@Injectable({ providedIn: 'root' })
export class ApiService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = environment.apiUrl.replace(/\/$/, '');

  getDailyReport(date?: string): Observable<DailyReport> {
    return this.http.get<DailyReport>(`${this.baseUrl}/daily-report`, {
      params: this.dateParams(date),
    });
  }

  getTimeline(date?: string): Observable<TimelineResponse> {
    return this.http.get<TimelineResponse>(`${this.baseUrl}/timeline`, {
      params: this.dateParams(date),
    });
  }

  getLiveStatus(deviceId?: string): Observable<LiveStatus> {
    let params = new HttpParams();
    if (deviceId) {
      params = params.set('deviceId', deviceId);
    }
    return this.http.get<LiveStatus>(`${this.baseUrl}/live-status`, { params });
  }

  private dateParams(date?: string): HttpParams {
    let params = new HttpParams();
    if (date) {
      params = params.set('date', date);
    }
    return params;
  }
}
