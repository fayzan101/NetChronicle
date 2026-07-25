import { HttpClient, HttpParams, HttpResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { environment } from '../../../environments/environment';
import {
  ApiKeyItem,
  AuthResponse,
  CreateApiKeyResponse,
  DailyReport,
  DeleteDataResponse,
  DeleteTokenResponse,
  DeviceItem,
  InsightsResponse,
  LiveStatus,
  NetworkEventsResponse,
  NetworkStatsResponse,
  ReportResponse,
  ReportsListResponse,
  ReportType,
  SessionsResponse,
  TimelineResponse,
  UserSettings,
  UserSettingsPatch,
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

  register(
    email: string,
    password: string,
    displayName?: string,
  ): Observable<AuthResponse> {
    return this.http.post<AuthResponse>(`${this.baseUrl}/auth/register`, {
      email,
      password,
      displayName: displayName || undefined,
    });
  }

  login(email: string, password: string): Observable<AuthResponse> {
    return this.http.post<AuthResponse>(`${this.baseUrl}/auth/login`, {
      email,
      password,
    });
  }

  getSettings(): Observable<UserSettings> {
    return this.http.get<UserSettings>(`${this.baseUrl}/settings`);
  }

  patchSettings(patch: UserSettingsPatch): Observable<UserSettings> {
    return this.http.patch<UserSettings>(`${this.baseUrl}/settings`, patch);
  }

  listApiKeys(): Observable<ApiKeyItem[]> {
    return this.http.get<ApiKeyItem[]>(`${this.baseUrl}/auth/api-keys`);
  }

  createApiKey(name?: string): Observable<CreateApiKeyResponse> {
    return this.http.post<CreateApiKeyResponse>(
      `${this.baseUrl}/auth/api-keys`,
      { name: name || undefined },
    );
  }

  revokeApiKey(id: string): Observable<void> {
    return this.http.delete<void>(`${this.baseUrl}/auth/api-keys/${id}`);
  }

  listDevices(): Observable<DeviceItem[]> {
    return this.http.get<DeviceItem[]>(`${this.baseUrl}/devices`);
  }

  registerDevice(agentId: string, name?: string): Observable<DeviceItem> {
    return this.http.post<DeviceItem>(`${this.baseUrl}/devices`, {
      agentId,
      name: name || undefined,
    });
  }

  exportActivity(
    format: 'json' | 'csv',
  ): Observable<HttpResponse<Blob>> {
    return this.http.post(
      `${this.baseUrl}/export`,
      { format },
      { responseType: 'blob', observe: 'response' },
    );
  }

  getDeleteToken(): Observable<DeleteTokenResponse> {
    return this.http.post<DeleteTokenResponse>(
      `${this.baseUrl}/data/delete-token`,
      {},
    );
  }

  deleteData(confirmation: string): Observable<DeleteDataResponse> {
    return this.http.request<DeleteDataResponse>('DELETE', `${this.baseUrl}/data`, {
      body: { confirmation },
    });
  }

  private dateParams(date?: string): HttpParams {
    let params = new HttpParams();
    if (date) {
      params = params.set('date', date);
    }
    return params;
  }
}
