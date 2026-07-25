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
