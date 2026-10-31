import React from 'react';
import {
  Activity,
  Layers,
  Repeat,
  Radio,
  FolderTree,
  ShieldAlert,
  Binary,
  GitCompare,
  Settings,
  CircleDot,
  PauseCircle,
  PlayCircle,
} from 'lucide-react';
import { ProxyStats } from '../types';

export type NavTab =
  | 'dashboard'
  | 'proxy'
  | 'history'
  | 'repeater'
  | 'target'
  | 'scanner'
  | 'decoder'
  | 'comparer'
  | 'settings';

interface NavbarProps {
  activeTab: NavTab;
  onSelectTab: (tab: NavTab) => void;
  stats: ProxyStats | null;
  onToggleIntercept: () => void;
}

export const Navbar: React.FC<NavbarProps> = ({
  activeTab,
  onSelectTab,
  stats,
  onToggleIntercept,
}) => {
  const tabs = [
    { id: 'dashboard', label: 'Dashboard', icon: Activity },
    { id: 'proxy', label: 'Proxy', icon: Radio },
    { id: 'history', label: 'HTTP History', icon: Layers },
    { id: 'repeater', label: 'Repeater', icon: Repeat },
    { id: 'target', label: 'Target', icon: FolderTree },
    { id: 'scanner', label: 'Scanner', icon: ShieldAlert },
    { id: 'decoder', label: 'Decoder', icon: Binary },
    { id: 'comparer', label: 'Comparer', icon: GitCompare },
    { id: 'settings', label: 'Settings', icon: Settings },
  ] as const;

  return (
    <header className="h-14 bg-dark-card border-b border-dark-border flex items-center justify-between px-4 select-none">
      <div className="flex items-center space-x-6">
        <div className="flex items-center space-x-2">
          <div className="w-8 h-8 rounded bg-brand-primary/20 border border-brand-primary/40 flex items-center justify-center font-bold text-brand-primary font-mono text-sm">
            RP
          </div>
          <span className="font-bold text-base tracking-wide text-dark-text">
            Rust<span className="text-brand-primary">Proxy</span>
          </span>
        </div>

        <nav className="flex items-center space-x-1">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => onSelectTab(tab.id as NavTab)}
                className={`flex items-center space-x-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${
                  isActive
                    ? 'bg-dark-border text-white shadow-sm'
                    : 'text-dark-muted hover:text-dark-text hover:bg-dark-cardHover'
                }`}
              >
                <Icon className="w-3.5 h-3.5" />
                <span>{tab.label}</span>
                {tab.id === 'scanner' && stats && stats.total_findings > 0 && (
                  <span className="ml-1 px-1.5 py-0.2 rounded-full bg-brand-warning/20 text-brand-warning text-[10px] font-bold">
                    {stats.total_findings}
                  </span>
                )}
                {tab.id === 'proxy' && stats && stats.pending_intercepts > 0 && (
                  <span className="ml-1 px-1.5 py-0.2 rounded-full bg-brand-danger/20 text-brand-danger text-[10px] font-bold animate-pulse">
                    {stats.pending_intercepts}
                  </span>
                )}
              </button>
            );
          })}
        </nav>
      </div>

      <div className="flex items-center space-x-4">
        {/* Intercept Toggle Button */}
        <button
          onClick={onToggleIntercept}
          className={`flex items-center space-x-1.5 px-2.5 py-1 rounded text-xs font-semibold border transition-all ${
            stats?.intercept_enabled
              ? 'bg-brand-warning/10 border-brand-warning text-brand-warning'
              : 'bg-dark-border/40 border-dark-border text-dark-muted hover:text-dark-text'
          }`}
        >
          {stats?.intercept_enabled ? (
            <>
              <PauseCircle className="w-3.5 h-3.5" />
              <span>Intercept ON</span>
            </>
          ) : (
            <>
              <PlayCircle className="w-3.5 h-3.5" />
              <span>Intercept OFF</span>
            </>
          )}
        </button>

        {/* Status Badge */}
        <div className="flex items-center space-x-2 bg-dark-bg px-2.5 py-1 rounded border border-dark-border text-xs">
          <CircleDot className="w-3 h-3 text-brand-accent animate-pulse" />
          <span className="font-mono text-dark-text">127.0.0.1:8080</span>
        </div>
      </div>
    </header>
  );
};
