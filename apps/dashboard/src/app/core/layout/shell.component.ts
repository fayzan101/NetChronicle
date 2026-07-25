import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-shell',
  standalone: true,
  imports: [RouterOutlet, RouterLink, RouterLinkActive],
  templateUrl: './shell.component.html',
  styleUrl: './shell.component.scss',
})
export class ShellComponent {
  readonly nav = [
    { path: '/', label: 'Today', exact: true, ready: true },
    { path: '/timeline', label: 'Timeline', exact: false, ready: true },
    { path: '/live', label: 'Live', exact: false, ready: true },
    { path: '/network', label: 'Network', exact: false, ready: true },
    { path: '/analytics', label: 'Analytics', exact: false, ready: true },
    { path: '/insights', label: 'Insights', exact: false, ready: true },
    { path: '/reports', label: 'Reports', exact: false, ready: true },
    { path: '/settings', label: 'Settings', exact: false, ready: false },
  ] as const;
}
