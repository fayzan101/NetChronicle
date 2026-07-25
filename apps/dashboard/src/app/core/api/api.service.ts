import { HttpClient, HttpParams } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../../../environments/environment';
import {
  DailyReport,
  InsightsResponse,
  LiveStatus,
  NetworkEventsResponse,
  NetworkStatsResponse,
  ReportResponse,
  ReportsListResponse,
  ReportType,
  SessionsResponse,
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

  getNetworkStats(date?: string): Observable<NetworkStatsResponse> {
    return this.http.get<NetworkStatsResponse>(`${this.baseUrl}/network-stats`, {
      params: this.dateParams(date),
    });
  }

  getNetworkEvents(date?: string): Observable<NetworkEventsResponse> {
    return this.http.get<NetworkEventsResponse>(
      `${this.baseUrl}/network-events`,
      { params: this.dateParams(date) },
    );
  }

  getSessions(date?: string): Observable<SessionsResponse> {
    return this.http.get<SessionsResponse>(`${this.baseUrl}/sessions`, {
      params: this.dateParams(date),
    });
  }

  getInsights(date?: string): Observable<InsightsResponse> {
    return this.http.get<InsightsResponse>(`${this.baseUrl}/insights`, {
      params: this.dateParams(date),
    });
  }

  getReport(type: ReportType, date?: string): Observable<ReportResponse> {
    return this.http.get<ReportResponse>(`${this.baseUrl}/reports/${type}`, {
      params: this.dateParams(date),
    });
  }

  listReports(reportType?: ReportType): Observable<ReportsListResponse> {
    let params = new HttpParams();
    if (reportType) {
      params = params.set('reportType', reportType);
    }
    return this.http.get<ReportsListResponse>(`${this.baseUrl}/reports`, {
      params,
    });
  }

  exportReportUrl(
    format: 'json' | 'csv',
    reportType: ReportType,
    date: string,
  ): string {
    const params = new HttpParams()
      .set('format', format)
      .set('reportType', reportType)
      .set('date', date);
    return `${this.baseUrl}/reports/export?${params.toString()}`;
  }

  private dateParams(date?: string): HttpParams {
    let params = new HttpParams();
    if (date) {
      params = params.set('date', date);
    }
    return params;
  }
}
