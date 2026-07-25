export interface DailyReport {
  date: string;
  productivityScore: number;
  totalOnlineMinutes: number;
  networkHealthScore: number;
  distractionRatio: number;
  focusMinutes: number;
  cached: boolean;
}

export interface LiveStatus {
  currentApp: string | null;
  currentSite: string | null;
  focusScore: number;
  sessionElapsedSec: number;
  networkLatencyMs: number | null;
  deviceId: string | null;
  deviceName: string | null;
}

export interface TimelineEntry {
  time: string;
  label: string;
  category: string;
  source: string;
  durationSec: number;
  sessionId?: string | null;
}

export interface TimelineResponse {
  date: string;
  entries: TimelineEntry[];
}

export interface NetworkSample {
  recordedAt: string;
  latencyMs: number | null;
  packetLossPct: number | null;
  bandwidthMbps: number | null;
  stability: string | null;
  disconnect: boolean;
}

export interface NetworkAggregation {
  sampleCount: number;
  avgLatencyMs: number | null;
  p95LatencyMs: number | null;
  avgPacketLossPct: number | null;
  avgBandwidthMbps: number | null;
  disconnectCount: number;
}

export interface NetworkStatsResponse {
  samples: NetworkSample[];
  aggregation: NetworkAggregation;
  stabilityScore: number;
}

export interface NetworkEvent {
  recordedAt: string;
  kind: string;
  latencyMs: number | null;
  packetLossPct: number | null;
  bandwidthMbps: number | null;
  stability: string | null;
  disconnect: boolean;
}

export interface NetworkEventsResponse {
  events: NetworkEvent[];
}

export interface SessionWebsite {
  domain: string;
  url: string;
  timeSpentSec: number;
  category: string;
}

export interface SessionItem {
  sessionId: string;
  startTime: string;
  endTime: string | null;
  category: string;
  productivityScore: number | null;
  primaryApps: string[];
  networkStability: string | null;
  websites: SessionWebsite[];
}

export interface SessionsResponse {
  sessions: SessionItem[];
  limit: number;
  offset: number;
}

export interface InsightItem {
  title: string;
  body: string;
  severity: string;
}

export interface InsightsResponse {
  insights: InsightItem[];
}

export interface NamedMinutes {
  category?: string;
  app?: string;
  domain?: string;
  minutes: number;
}

export interface HourBucket {
  hour: number;
  totalMinutes: number;
  productiveMinutes: number;
  distractionMinutes: number;
}

export interface ReportSummary {
  productivityScore?: number;
  totalOnlineMinutes?: number;
  productiveMinutes?: number;
  networkHealthScore?: number;
  distractionRatio?: number;
  distractionImpactPct?: number;
  focusMinutes?: number;
  sessionCount?: number;
  averageProductivityScore?: number;
  categoryMinutes?: NamedMinutes[];
  timeOfDay?: HourBucket[];
  topApps?: NamedMinutes[];
  topDomains?: NamedMinutes[];
}

export type ReportType = 'daily' | 'weekly' | 'monthly';

export interface ReportResponse {
  reportType: ReportType | string;
  periodStart: string;
  periodEnd: string;
  summary: ReportSummary;
  cached: boolean;
}

export interface ReportListItem {
  reportType: string;
  periodStart: string;
  periodEnd: string;
  summary: ReportSummary;
  createdAt: string;
}

export interface ReportsListResponse {
  reports: ReportListItem[];
}
