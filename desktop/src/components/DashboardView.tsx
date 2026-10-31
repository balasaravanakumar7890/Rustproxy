import React from 'react';
import {
  Activity,
  Layers,
  Repeat,
  ShieldAlert,
  Radio,
  CheckCircle2,
  Clock,
  ArrowUpRight,
  ExternalLink,
} from 'lucide-react';
import { FlowSummary, ProxyStats, SecurityFinding, SiteMapNode } from '../types';
import { NavTab } from './Navbar';

interface DashboardViewProps {
  stats: ProxyStats | null;
  recentFlows: FlowSummary[];
  findings: SecurityFinding[];
  sitemap: SiteMapNode[];
  onNavigate: (tab: NavTab) => void;
}

export const DashboardView: React.FC<DashboardViewProps> = ({
  stats,
  recentFlows,
  findings,
  sitemap,
  onNavigate,
}) => {
  return (
    <div className="p-6 space-y-6 overflow-y-auto h-[calc(100vh-3.5rem)]">
      {/* Top Banner */}
      <div className="bg-gradient-to-r from-dark-card to-dark-cardHover border border-dark-border rounded-xl p-6 flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-white flex items-center space-x-2">
            <span>Defensive Security Platform</span>
            <span className="px-2 py-0.5 rounded text-xs bg-brand-primary/20 text-brand-primary font-mono border border-brand-primary/30">
              v0.1.0-Rust
            </span>
          </h1>
          <p className="text-sm text-dark-muted mt-1">
            Engineered with native Tokio async runtime, Rustls dynamic CA MITM interception, and SQLite WAL storage.
          </p>
        </div>
        <div className="flex items-center space-x-3">
          <button
            onClick={() => onNavigate('repeater')}
            className="flex items-center space-x-1.5 px-3.5 py-2 rounded-lg bg-brand-primary text-white text-xs font-semibold hover:bg-brand-hover transition-colors shadow-sm"
          >
            <Repeat className="w-4 h-4" />
            <span>New Request</span>
          </button>
          <button
            onClick={() => onNavigate('proxy')}
            className="flex items-center space-x-1.5 px-3.5 py-2 rounded-lg bg-dark-bg border border-dark-border text-dark-text text-xs font-semibold hover:border-dark-muted transition-colors"
          >
            <Radio className="w-4 h-4" />
            <span>Proxy Controls</span>
          </button>
        </div>
      </div>

      {/* Metric Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        {/* Card 1: Proxy Status */}
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center justify-between text-dark-muted text-xs font-medium">
            <span>Proxy Listener</span>
            <Activity className="w-4 h-4 text-brand-accent" />
          </div>
          <div className="mt-2 flex items-baseline space-x-2">
            <span className="text-2xl font-bold font-mono text-white">8080</span>
            <span className="text-xs text-brand-accent font-medium">Active (127.0.0.1)</span>
          </div>
          <div className="mt-3 text-[11px] text-dark-muted flex items-center space-x-1">
            <CheckCircle2 className="w-3.5 h-3.5 text-brand-accent" />
            <span>HTTP/1.1 & HTTPS MITM</span>
          </div>
        </div>

        {/* Card 2: Total Flows */}
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center justify-between text-dark-muted text-xs font-medium">
            <span>Total Flows</span>
            <Layers className="w-4 h-4 text-brand-primary" />
          </div>
          <div className="mt-2 flex items-baseline space-x-2">
            <span className="text-2xl font-bold font-mono text-white">
              {stats?.total_flows ?? 0}
            </span>
            <span className="text-xs text-dark-muted">recorded</span>
          </div>
          <div className="mt-3 text-[11px] text-dark-muted flex items-center justify-between">
            <span>Persistent in SQLite WAL</span>
            <button
              onClick={() => onNavigate('history')}
              className="text-brand-primary hover:underline flex items-center space-x-0.5"
            >
              <span>View</span>
              <ArrowUpRight className="w-3 h-3" />
            </button>
          </div>
        </div>

        {/* Card 3: Intercept Queue */}
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center justify-between text-dark-muted text-xs font-medium">
            <span>Interception Mode</span>
            <Radio className="w-4 h-4 text-brand-warning" />
          </div>
          <div className="mt-2 flex items-baseline space-x-2">
            <span
              className={`text-2xl font-bold font-mono ${
                stats?.intercept_enabled ? 'text-brand-warning' : 'text-dark-muted'
              }`}
            >
              {stats?.intercept_enabled ? 'ON' : 'OFF'}
            </span>
            <span className="text-xs text-dark-muted">
              ({stats?.pending_intercepts ?? 0} held)
            </span>
          </div>
          <div className="mt-3 text-[11px] text-dark-muted flex items-center justify-between">
            <span>Pause & Tamper</span>
            <button
              onClick={() => onNavigate('proxy')}
              className="text-brand-primary hover:underline flex items-center space-x-0.5"
            >
              <span>Manage</span>
              <ArrowUpRight className="w-3 h-3" />
            </button>
          </div>
        </div>

        {/* Card 4: Security Findings */}
        <div className="bg-dark-card border border-dark-border rounded-xl p-4">
          <div className="flex items-center justify-between text-dark-muted text-xs font-medium">
            <span>Passive Findings</span>
            <ShieldAlert className="w-4 h-4 text-brand-danger" />
          </div>
          <div className="mt-2 flex items-baseline space-x-2">
            <span className="text-2xl font-bold font-mono text-white">
              {findings.length}
            </span>
            <span className="text-xs text-dark-muted">OWASP checks</span>
          </div>
          <div className="mt-3 text-[11px] text-dark-muted flex items-center justify-between">
            <span>Zero-latency scanner</span>
            <button
              onClick={() => onNavigate('scanner')}
              className="text-brand-primary hover:underline flex items-center space-x-0.5"
            >
              <span>Audit</span>
              <ArrowUpRight className="w-3 h-3" />
            </button>
          </div>
        </div>
      </div>

      {/* Grid: Discovered Hosts & Recent Findings */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Discovered Targets */}
        <div className="bg-dark-card border border-dark-border rounded-xl p-5">
          <div className="flex items-center justify-between pb-3 border-b border-dark-border">
            <h2 className="text-sm font-bold text-white flex items-center space-x-2">
              <span>Discovered Target Hosts</span>
              <span className="text-xs text-dark-muted font-normal">
                ({sitemap.length} hosts)
              </span>
            </h2>
            <button
              onClick={() => onNavigate('target')}
              className="text-xs text-brand-primary hover:underline flex items-center space-x-1"
            >
              <span>Site Map</span>
              <ExternalLink className="w-3 h-3" />
            </button>
          </div>

          <div className="mt-4 space-y-2 max-h-56 overflow-y-auto">
            {sitemap.length === 0 ? (
              <p className="text-xs text-dark-muted py-6 text-center">
                No targets mapped yet. Requests through proxy will automatically build the tree.
              </p>
            ) : (
              sitemap.map((host) => (
                <div
                  key={host.id}
                  className="flex items-center justify-between p-2.5 rounded-lg bg-dark-bg/60 border border-dark-border/40 text-xs hover:border-dark-border"
                >
                  <span className="font-mono text-dark-text truncate max-w-[280px]">
                    {host.name}
                  </span>
                  <div className="flex items-center space-x-2">
                    <span className="px-1.5 py-0.5 rounded bg-dark-border text-[10px] text-dark-muted">
                      {Object.keys(host.children).length} endpoints
                    </span>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        {/* Recent Passive Findings */}
        <div className="bg-dark-card border border-dark-border rounded-xl p-5">
          <div className="flex items-center justify-between pb-3 border-b border-dark-border">
            <h2 className="text-sm font-bold text-white flex items-center space-x-2">
              <span>Security Diagnostic Alerts</span>
              <span className="text-xs text-dark-muted font-normal">
                ({findings.length} total)
              </span>
            </h2>
            <button
              onClick={() => onNavigate('scanner')}
              className="text-xs text-brand-primary hover:underline flex items-center space-x-1"
            >
              <span>All Findings</span>
              <ExternalLink className="w-3 h-3" />
            </button>
          </div>

          <div className="mt-4 space-y-2 max-h-56 overflow-y-auto">
            {findings.length === 0 ? (
              <p className="text-xs text-dark-muted py-6 text-center">
                No configuration vulnerabilities or security issues detected yet.
              </p>
            ) : (
              findings.slice(0, 5).map((f) => (
                <div
                  key={f.id}
                  className="flex items-start justify-between p-2.5 rounded-lg bg-dark-bg/60 border border-dark-border/40 text-xs"
                >
                  <div>
                    <div className="font-semibold text-dark-text">{f.title}</div>
                    <div className="text-[11px] text-dark-muted font-mono truncate max-w-[320px] mt-0.5">
                      {f.url}
                    </div>
                  </div>
                  <span
                    className={`px-2 py-0.5 rounded text-[10px] font-bold uppercase ${
                      f.severity === 'Critical' || f.severity === 'High'
                        ? 'bg-brand-danger/20 text-brand-danger'
                        : f.severity === 'Medium'
                        ? 'bg-brand-warning/20 text-brand-warning'
                        : 'bg-brand-primary/20 text-brand-primary'
                    }`}
                  >
                    {f.severity}
                  </span>
                </div>
              ))
            )}
          </div>
        </div>
      </div>

      {/* Recent Traffic Table */}
      <div className="bg-dark-card border border-dark-border rounded-xl p-5">
        <div className="flex items-center justify-between pb-3 border-b border-dark-border">
          <h2 className="text-sm font-bold text-white flex items-center space-x-2">
            <Clock className="w-4 h-4 text-dark-muted" />
            <span>Recent Traffic Stream</span>
          </h2>
          <button
            onClick={() => onNavigate('history')}
            className="text-xs text-brand-primary hover:underline flex items-center space-x-1"
          >
            <span>Full History</span>
            <ExternalLink className="w-3 h-3" />
          </button>
        </div>

        <div className="mt-4 overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead>
              <tr className="text-dark-muted border-b border-dark-border pb-2">
                <th className="py-2 px-3 font-medium">Method</th>
                <th className="py-2 px-3 font-medium">Status</th>
                <th className="py-2 px-3 font-medium">Host</th>
                <th className="py-2 px-3 font-medium">Path</th>
                <th className="py-2 px-3 font-medium text-right">Size</th>
                <th className="py-2 px-3 font-medium text-right">Duration</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-dark-border/40">
              {recentFlows.length === 0 ? (
                <tr>
                  <td colSpan={6} className="py-8 text-center text-dark-muted">
                    No requests captured yet. Send traffic through proxy to see real-time flows.
                  </td>
                </tr>
              ) : (
                recentFlows.slice(0, 8).map((flow) => (
                  <tr
                    key={flow.id}
                    onClick={() => onNavigate('history')}
                    className="hover:bg-dark-cardHover/60 cursor-pointer transition-colors"
                  >
                    <td className="py-2.5 px-3 font-mono font-bold">
                      <span
                        className={
                          flow.method === 'GET'
                            ? 'text-blue-400'
                            : flow.method === 'POST'
                            ? 'text-emerald-400'
                            : flow.method === 'PUT'
                            ? 'text-amber-400'
                            : 'text-rose-400'
                        }
                      >
                        {flow.method}
                      </span>
                    </td>
                    <td className="py-2.5 px-3 font-mono font-semibold">
                      <span
                        className={
                          flow.status && flow.status < 300
                            ? 'text-emerald-400'
                            : flow.status && flow.status < 400
                            ? 'text-cyan-400'
                            : 'text-rose-400'
                        }
                      >
                        {flow.status ?? 'ERR'}
                      </span>
                    </td>
                    <td className="py-2.5 px-3 font-mono text-dark-text truncate max-w-[200px]">
                      {flow.host}
                    </td>
                    <td className="py-2.5 px-3 font-mono text-dark-muted truncate max-w-[260px]">
                      {flow.path}
                    </td>
                    <td className="py-2.5 px-3 font-mono text-right text-dark-muted">
                      {flow.response_len ? `${flow.response_len} B` : '-'}
                    </td>
                    <td className="py-2.5 px-3 font-mono text-right text-dark-muted">
                      {flow.duration_ms ? `${flow.duration_ms} ms` : '-'}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
