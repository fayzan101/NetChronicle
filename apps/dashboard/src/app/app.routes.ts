import { Routes } from '@angular/router';

import { ShellComponent } from './core/layout/shell.component';
import { AnalyticsComponent } from './pages/analytics/analytics.component';
import { InsightsComponent } from './pages/insights/insights.component';
import { LiveComponent } from './pages/live/live.component';
import { NetworkComponent } from './pages/network/network.component';
import { ReportsComponent } from './pages/reports/reports.component';
import { SettingsComponent } from './pages/settings/settings.component';
import { TimelineComponent } from './pages/timeline/timeline.component';
import { TodayComponent } from './pages/today/today.component';

export const routes: Routes = [
  {
    path: '',
    component: ShellComponent,
    children: [
      { path: '', component: TodayComponent },
      { path: 'timeline', component: TimelineComponent },
      { path: 'live', component: LiveComponent },
      { path: 'network', component: NetworkComponent },
      { path: 'analytics', component: AnalyticsComponent },
      { path: 'insights', component: InsightsComponent },
      { path: 'reports', component: ReportsComponent },
      { path: 'settings', component: SettingsComponent },
      { path: '**', redirectTo: '' },
    ],
  },
];
