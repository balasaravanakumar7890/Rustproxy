import React, { useState, useEffect } from 'react';
import {
  Search,
  Trash2,
  Repeat,
  Copy,
  Check,
} from 'lucide-react';
import { FlowDetail, FlowSummary, RequestModel } from '../types';
import { api } from '../api/client';

interface HistoryViewProps {
  flows: FlowSummary[];
  onRefreshFlows: () => Promise<void>;
  onSendToRepeater: (req: RequestModel) => void;
  onSendToComparer: (text: string) => void;
}

export const HistoryView: React.FC<HistoryViewProps> = ({
  flows,
  onRefreshFlows,
  onSendToRepeater,
  onSendToComparer,
}) => {
  const [selectedFlowId, setSelectedFlowId] = useState<string | null>(null);
  const [flowDetail, setFlowDetail] = useState<FlowDetail | null>(null);
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [methodFilter, setMethodFilter] = useState<string>('ALL');
  const [inScopeOnly, setInScopeOnly] = useState<boolean>(false);
  const [activePaneTab, setActivePaneTab] = useState<'request' | 'response'>('request');
  const [copied, setCopied] = useState<boolean>(false);

  // Load flow detail when selection changes
  useEffect(() => {
    if (selectedFlowId) {
      api.getFlowDetail(selectedFlowId).then(setFlowDetail).catch(console.error);
    } else {
      setFlowDetail(null);
    }
  }, [selectedFlowId]);

  const filteredFlows = flows.filter((f) => {
    if (inScopeOnly && !f.in_scope) return false;
    if (methodFilter !== 'ALL' && f.method !== methodFilter) return false;
    if (searchQuery) {
      const q = searchQuery.toLowerCase();
      const matchHost = f.host.toLowerCase().includes(q);
      const matchPath = f.path.toLowerCase().includes(q);
      const matchMethod = f.method.toLowerCase().includes(q);
      const matchStatus = f.status?.toString().includes(q);
      if (!matchHost && !matchPath && !matchMethod && !matchStatus) return false;
    }
    return true;
  });

  const handleCopyRaw = () => {
    if (!flowDetail) return;
    const textToCopy =
      activePaneTab === 'request'
        ? flowDetail.request_body_str
        : flowDetail.response_body_str ?? '';
    navigator.clipboard.writeText(textToCopy);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  const handleClearHistory = async () => {
    if (window.confirm('Clear all captured HTTP traffic history?')) {
      await api.clearFlows();
      setSelectedFlowId(null);
      await onRefreshFlows();
    }
  };

  return (
    <div className="flex flex-col h-[calc(100vh-3.5rem)] bg-dark-bg select-none">
      {/* Top Filter Bar */}
      <div className="h-12 bg-dark-card border-b border-dark-border px-4 flex items-center justify-between">
        <div className="flex items-center space-x-3 flex-1 max-w-xl">
          <div className="relative flex-1">
            <Search className="w-3.5 h-3.5 text-dark-muted absolute left-3 top-2.5" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search Host, Path, Method, or Status..."
              className="w-full bg-dark-bg border border-dark-border rounded-lg pl-9 pr-3 py-1.5 text-xs text-dark-text outline-none focus:border-brand-primary font-mono"
            />
          </div>

          <select
            value={methodFilter}
            onChange={(e) => setMethodFilter(e.target.value)}
            className="bg-dark-bg border border-dark-border text-dark-text text-xs rounded-lg px-2.5 py-1.5 outline-none focus:border-brand-primary font-mono font-semibold"
          >
            <option value="ALL">All Methods</option>
            <option value="GET">GET</option>
            <option value="POST">POST</option>
            <option value="PUT">PUT</option>
            <option value="DELETE">DELETE</option>
            <option value="PATCH">PATCH</option>
            <option value="OPTIONS">OPTIONS</option>
          </select>

          <label className="flex items-center space-x-1.5 text-xs text-dark-muted cursor-pointer hover:text-dark-text">
            <input
              type="checkbox"
              checked={inScopeOnly}
              onChange={(e) => setInScopeOnly(e.target.checked)}
              className="rounded bg-dark-bg border-dark-border text-brand-primary"
            />
            <span>In-Scope Only</span>
          </label>
        </div>

        <div className="flex items-center space-x-3 text-xs">
          <span className="text-dark-muted">
            Showing <strong className="text-white">{filteredFlows.length}</strong> of{' '}
            {flows.length} flows
          </span>
          <button
            onClick={handleClearHistory}
            className="flex items-center space-x-1 px-2.5 py-1 rounded border border-dark-border text-dark-muted hover:text-brand-danger hover:border-brand-danger/40 transition-colors"
          >
            <Trash2 className="w-3.5 h-3.5" />
            <span>Clear</span>
          </button>
        </div>
      </div>

      {/* Main Split Layout: Table (Top/Left) and Inspector (Bottom/Right) */}
      <div className="flex-1 flex flex-col min-h-0">
        {/* Flows Table */}
        <div className={`overflow-auto border-b border-dark-border ${selectedFlowId ? 'h-1/2' : 'h-full'}`}>
          <table className="w-full text-left text-xs border-collapse">
            <thead className="sticky top-0 bg-dark-card border-b border-dark-border z-10 text-dark-muted font-mono">
              <tr>
                <th className="py-2 px-3 font-semibold w-12 text-center">#</th>
                <th className="py-2 px-3 font-semibold w-20">Method</th>
                <th className="py-2 px-3 font-semibold w-20">Status</th>
                <th className="py-2 px-3 font-semibold w-64">Host</th>
                <th className="py-2 px-3 font-semibold">Path</th>
                <th className="py-2 px-3 font-semibold w-28">MIME</th>
                <th className="py-2 px-3 font-semibold w-20 text-right">Size</th>
                <th className="py-2 px-3 font-semibold w-20 text-right">Time</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-dark-border/30 font-mono">
              {filteredFlows.length === 0 ? (
                <tr>
                  <td colSpan={8} className="py-12 text-center text-dark-muted font-sans text-xs">
                    No HTTP traffic recorded matching current filters.
                  </td>
                </tr>
              ) : (
                filteredFlows.map((flow, index) => {
                  const isSelected = selectedFlowId === flow.id;
                  return (
                    <tr
                      key={flow.id}
                      onClick={() => setSelectedFlowId(flow.id)}
                      className={`cursor-pointer transition-colors ${
                        isSelected
                          ? 'bg-brand-primary/20 text-white'
                          : 'hover:bg-dark-cardHover text-dark-text'
                      }`}
                    >
                      <td className="py-1.5 px-3 text-center text-dark-muted font-mono text-[11px]">
                        {filteredFlows.length - index}
                      </td>
                      <td className="py-1.5 px-3 font-bold text-[11px]">
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
                      <td className="py-1.5 px-3 font-bold text-[11px]">
                        <span
                          className={
                            flow.status && flow.status < 300
                              ? 'text-emerald-400'
                              : flow.status && flow.status < 400
                              ? 'text-cyan-400'
                              : flow.status && flow.status < 500
                              ? 'text-amber-400'
                              : 'text-rose-400'
                          }
                        >
                          {flow.status ?? 'ERR'}
                        </span>
                      </td>
                      <td className="py-1.5 px-3 truncate max-w-[200px] text-dark-text">
                        {flow.host}
                      </td>
                      <td className="py-1.5 px-3 truncate max-w-[320px] text-dark-muted">
                        {flow.path}
                      </td>
                      <td className="py-1.5 px-3 truncate text-dark-muted text-[11px]">
                        {flow.mime_type ?? '-'}
                      </td>
                      <td className="py-1.5 px-3 text-right text-dark-muted">
                        {flow.response_len ? `${flow.response_len} B` : '-'}
                      </td>
                      <td className="py-1.5 px-3 text-right text-dark-muted">
                        {flow.duration_ms ? `${flow.duration_ms} ms` : '-'}
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
          </table>
        </div>

        {/* Bottom Split Detail Inspector */}
        {selectedFlowId && flowDetail && (
          <div className="h-1/2 flex flex-col bg-dark-card overflow-hidden">
            {/* Inspector Tab Bar */}
            <div className="h-10 bg-dark-bg border-b border-dark-border px-4 flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <button
                  onClick={() => setActivePaneTab('request')}
                  className={`px-3 py-1 rounded text-xs font-semibold ${
                    activePaneTab === 'request'
                      ? 'bg-dark-card text-white border-b-2 border-brand-primary'
                      : 'text-dark-muted hover:text-dark-text'
                  }`}
                >
                  Request ({flowDetail.flow.request.method} {flowDetail.flow.request.path})
                </button>
                <button
                  onClick={() => setActivePaneTab('response')}
                  className={`px-3 py-1 rounded text-xs font-semibold ${
                    activePaneTab === 'response'
                      ? 'bg-dark-card text-white border-b-2 border-brand-primary'
                      : 'text-dark-muted hover:text-dark-text'
                  }`}
                >
                  Response ({flowDetail.flow.response?.status ?? 'No Response'})
                </button>
              </div>

              <div className="flex items-center space-x-2">
                <button
                  onClick={() => onSendToRepeater(flowDetail.flow.request)}
                  className="flex items-center space-x-1 px-2.5 py-1 rounded text-xs bg-dark-cardHover border border-dark-border hover:border-dark-muted text-white font-medium"
                >
                  <Repeat className="w-3.5 h-3.5" />
                  <span>Send to Repeater (Ctrl+R)</span>
                </button>
                <button
                  onClick={() =>
                    onSendToComparer(
                      activePaneTab === 'request'
                        ? flowDetail.request_body_str
                        : flowDetail.response_body_str ?? ''
                    )
                  }
                  className="flex items-center space-x-1 px-2.5 py-1 rounded text-xs bg-dark-cardHover border border-dark-border hover:border-dark-muted text-white font-medium"
                >
                  <span>Send to Comparer</span>
                </button>
                <button
                  onClick={handleCopyRaw}
                  className="flex items-center space-x-1 px-2.5 py-1 rounded text-xs bg-dark-cardHover border border-dark-border hover:border-dark-muted text-white font-medium"
                >
                  {copied ? <Check className="w-3.5 h-3.5 text-brand-accent" /> : <Copy className="w-3.5 h-3.5" />}
                  <span>{copied ? 'Copied!' : 'Copy'}</span>
                </button>
              </div>
            </div>

            {/* Inspector Body Content */}
            <div className="flex-1 overflow-auto p-4 font-mono text-xs text-dark-text bg-dark-bg/40 leading-relaxed">
              {activePaneTab === 'request' ? (
                <div>
                  <div className="text-brand-primary font-bold pb-2 mb-2 border-b border-dark-border/60">
                    {flowDetail.flow.request.method} {flowDetail.flow.request.uri} {flowDetail.flow.request.version}
                  </div>
                  <div className="text-dark-muted mb-4 space-y-0.5">
                    {flowDetail.flow.request.headers.map(([k, v], i) => (
                      <div key={i}>
                        <span className="text-dark-text font-semibold">{k}:</span> {v}
                      </div>
                    ))}
                  </div>
                  {flowDetail.request_body_str && (
                    <div className="pt-2 border-t border-dark-border/60 whitespace-pre-wrap text-emerald-300">
                      {flowDetail.request_body_str}
                    </div>
                  )}
                </div>
              ) : (
                <div>
                  {flowDetail.flow.response ? (
                    <div>
                      <div className="text-brand-accent font-bold pb-2 mb-2 border-b border-dark-border/60">
                        {flowDetail.flow.response.version} {flowDetail.flow.response.status}{' '}
                        {flowDetail.flow.response.status_text}
                      </div>
                      <div className="text-dark-muted mb-4 space-y-0.5">
                        {flowDetail.flow.response.headers.map(([k, v], i) => (
                          <div key={i}>
                            <span className="text-dark-text font-semibold">{k}:</span> {v}
                          </div>
                        ))}
                      </div>
                      {flowDetail.response_body_str && (
                        <div className="pt-2 border-t border-dark-border/60 whitespace-pre-wrap text-amber-200">
                          {flowDetail.response_body_str}
                        </div>
                      )}
                    </div>
                  ) : (
                    <div className="text-dark-muted italic">No response received (Request error or in-flight).</div>
                  )}
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
