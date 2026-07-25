import { Routes } from '@angular/router';

import { ShellComponent } from './core/layout/shell.component';
import { LiveComponent } from './pages/live/live.component';
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
      { path: '**', redirectTo: '' },
    ],
  },
];
