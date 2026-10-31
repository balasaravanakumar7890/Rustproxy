import React, { useState, useEffect } from 'react';
import {
  Send,
  Plus,
  X,
  Copy,
} from 'lucide-react';
import { RepeaterTab, RequestModel, ResponseModel } from '../types';
import { api } from '../api/client';

interface RepeaterViewProps {
  tabs: RepeaterTab[];
  onRefreshTabs: () => Promise<void>;
  initialRequest?: RequestModel | null;
}

export const RepeaterView: React.FC<RepeaterViewProps> = ({
  tabs,
  onRefreshTabs,
  initialRequest,
}) => {
  const [activeTabId, setActiveTabId] = useState<string | null>(null);
  const [method, setMethod] = useState<string>('GET');
  const [url, setUrl] = useState<string>('http://127.0.0.1:8080/');
  const [headersText, setHeadersText] = useState<string>('Host: 127.0.0.1\nUser-Agent: RustProxy-Repeater/1.0\n');
  const [bodyText, setBodyText] = useState<string>('');
  const [activeResponse, setActiveResponse] = useState<ResponseModel | null>(null);
  const [responseBodyStr, setResponseBodyStr] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [activeResViewTab, setActiveResViewTab] = useState<'body' | 'headers' | 'raw'>('body');

  // Handle incoming request sent to Repeater from History / Intercept
  useEffect(() => {
    if (initialRequest) {
      api.createRepeaterTab(
        `${initialRequest.method} ${initialRequest.path}`,
        initialRequest
      ).then(async (res) => {
        await onRefreshTabs();
        setActiveTabId(res.tab_id);
      });
    }
  }, [initialRequest]);

  // Set active tab on initial load if none selected
  useEffect(() => {
    if (!activeTabId && tabs.length > 0) {
      setActiveTabId(tabs[0].id);
    }
  }, [tabs, activeTabId]);

  const currentTab = tabs.find((t) => t.id === activeTabId);

  // Sync tab data when tab changes
  useEffect(() => {
    if (currentTab) {
      setMethod(currentTab.request.method);
      setUrl(currentTab.request.uri);
      const hdrs = currentTab.request.headers
        .map(([k, v]) => `${k}: ${v}`)
        .join('\n');
      setHeadersText(hdrs);
      setBodyText(currentTab.request.body ?? '');
      setActiveResponse(currentTab.response ?? null);
      setResponseBodyStr('');
    }
  }, [activeTabId, currentTab?.id]);

  const handleCreateNewTab = async () => {
    const res = await api.createRepeaterTab();
    await onRefreshTabs();
    setActiveTabId(res.tab_id);
  };

  const handleDuplicateTab = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const res = await api.duplicateRepeaterTab(id);
    await onRefreshTabs();
    setActiveTabId(res.tab_id);
  };

  const handleCloseTab = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    await api.closeRepeaterTab(id);
    await onRefreshTabs();
    if (activeTabId === id) {
      const remaining = tabs.filter((t) => t.id !== id);
      setActiveTabId(remaining.length > 0 ? remaining[0].id : null);
    }
  };

  const handleSend = async () => {
    if (!activeTabId) return;
    setIsLoading(true);

    try {
      const res = await api.sendRepeaterTab(activeTabId);
      setActiveResponse(res.response);
      setResponseBodyStr(res.body_str);
      await onRefreshTabs();
    } catch (err: any) {
      alert(`Send Error: ${err.message}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex flex-col h-[calc(100vh-3.5rem)] bg-dark-bg select-none">
      {/* Top Tab Bar */}
      <div className="h-10 bg-dark-card border-b border-dark-border px-2 flex items-center justify-between overflow-x-auto">
        <div className="flex items-center space-x-1 overflow-x-auto flex-1">
          {tabs.map((tab) => {
            const isActive = tab.id === activeTabId;
            return (
              <div
                key={tab.id}
                onClick={() => setActiveTabId(tab.id)}
                className={`group flex items-center space-x-2 px-3 py-1.5 rounded-t-md text-xs font-mono font-medium cursor-pointer border-t border-l border-r transition-colors ${
                  isActive
                    ? 'bg-dark-bg border-dark-border text-white'
                    : 'border-transparent text-dark-muted hover:text-dark-text hover:bg-dark-cardHover'
                }`}
              >
                <span>{tab.name}</span>
                <button
                  onClick={(e) => handleDuplicateTab(tab.id, e)}
                  title="Duplicate Tab"
                  className="opacity-0 group-hover:opacity-100 text-dark-muted hover:text-dark-text"
                >
                  <Copy className="w-3 h-3" />
                </button>
                <button
                  onClick={(e) => handleCloseTab(tab.id, e)}
                  title="Close Tab"
                  className="opacity-0 group-hover:opacity-100 text-dark-muted hover:text-brand-danger"
                >
                  <X className="w-3 h-3" />
                </button>
              </div>
            );
          })}

          <button
            onClick={handleCreateNewTab}
            title="New Tab"
            className="p-1.5 rounded text-dark-muted hover:text-white hover:bg-dark-cardHover transition-colors ml-1"
          >
            <Plus className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Target URL & Method Bar */}
      <div className="h-12 bg-dark-card border-b border-dark-border px-4 flex items-center space-x-3">
        <select
          value={method}
          onChange={(e) => setMethod(e.target.value)}
          className="bg-dark-bg border border-dark-border text-white text-xs font-mono font-bold rounded-lg px-3 py-1.5 outline-none focus:border-brand-primary"
        >
          <option value="GET">GET</option>
          <option value="POST">POST</option>
          <option value="PUT">PUT</option>
          <option value="DELETE">DELETE</option>
          <option value="PATCH">PATCH</option>
          <option value="OPTIONS">OPTIONS</option>
          <option value="HEAD">HEAD</option>
        </select>

        <input
          type="text"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          className="flex-1 bg-dark-bg border border-dark-border text-dark-text text-xs font-mono rounded-lg px-3 py-1.5 outline-none focus:border-brand-primary"
          placeholder="https://example.com/api/endpoint"
        />

        <button
          onClick={handleSend}
          disabled={isLoading}
          className="px-4 py-1.5 rounded-lg text-xs font-bold bg-brand-primary text-white hover:bg-brand-hover disabled:opacity-40 flex items-center space-x-1.5 shadow-sm transition-colors"
        >
          <Send className="w-3.5 h-3.5 fill-current" />
          <span>{isLoading ? 'Sending...' : 'Send (Ctrl+Enter)'}</span>
        </button>
      </div>

      {/* Split Workspace: Left (Request Editor) vs Right (Response Viewer) */}
      <div className="flex-1 flex overflow-hidden p-4 space-x-4">
        {/* Left: Request Editor */}
        <div className="w-1/2 flex flex-col bg-dark-card border border-dark-border rounded-xl p-4 overflow-hidden space-y-3">
          <div className="flex items-center justify-between pb-2 border-b border-dark-border text-xs text-dark-muted font-semibold">
            <span className="text-white">Request Message</span>
            <span>Raw HTTP / Headers & Body</span>
          </div>

          <div className="text-[11px] font-semibold text-dark-muted">Headers</div>
          <textarea
            value={headersText}
            onChange={(e) => setHeadersText(e.target.value)}
            className="h-32 bg-dark-bg border border-dark-border rounded-lg p-2.5 font-mono text-xs text-dark-text outline-none focus:border-brand-primary resize-none"
            placeholder="Host: example.com&#10;User-Agent: Mozilla/5.0&#10;Content-Type: application/json"
          />

          <div className="text-[11px] font-semibold text-dark-muted">Body Payload</div>
          <textarea
            value={bodyText}
            onChange={(e) => setBodyText(e.target.value)}
            className="flex-1 bg-dark-bg border border-dark-border rounded-lg p-2.5 font-mono text-xs text-dark-text outline-none focus:border-brand-primary resize-none"
            placeholder="Payload data..."
          />
        </div>

        {/* Right: Response Viewer */}
        <div className="w-1/2 flex flex-col bg-dark-card border border-dark-border rounded-xl p-4 overflow-hidden space-y-3">
          <div className="flex items-center justify-between pb-2 border-b border-dark-border text-xs">
            <div className="flex items-center space-x-2">
              <span className="font-semibold text-white">Response</span>
              {activeResponse && (
                <span
                  className={`px-2 py-0.5 rounded font-mono font-bold text-[11px] ${
                    activeResponse.status < 300
                      ? 'bg-brand-accent/20 text-brand-accent'
                      : activeResponse.status < 400
                      ? 'bg-cyan-500/20 text-cyan-400'
                      : 'bg-brand-danger/20 text-brand-danger'
                  }`}
                >
                  {activeResponse.status} {activeResponse.status_text}
                </span>
              )}
            </div>

            {activeResponse && (
              <div className="flex items-center space-x-3 text-[11px] text-dark-muted font-mono">
                <span>{activeResponse.duration_ms} ms</span>
                <span>{activeResponse.body_length} bytes</span>
              </div>
            )}
          </div>

          {/* Sub-tabs: Body vs Headers */}
          <div className="flex items-center space-x-2 border-b border-dark-border/40 pb-2 text-xs">
            <button
              onClick={() => setActiveResViewTab('body')}
              className={`px-2.5 py-1 rounded text-xs font-semibold ${
                activeResViewTab === 'body'
                  ? 'bg-dark-border text-white'
                  : 'text-dark-muted hover:text-dark-text'
              }`}
            >
              Response Body
            </button>
            <button
              onClick={() => setActiveResViewTab('headers')}
              className={`px-2.5 py-1 rounded text-xs font-semibold ${
                activeResViewTab === 'headers'
                  ? 'bg-dark-border text-white'
                  : 'text-dark-muted hover:text-dark-text'
              }`}
            >
              Headers ({activeResponse?.headers.length ?? 0})
            </button>
          </div>

          <div className="flex-1 bg-dark-bg border border-dark-border rounded-lg p-3 overflow-auto font-mono text-xs text-dark-text leading-relaxed">
            {!activeResponse ? (
              <div className="h-full flex flex-col items-center justify-center text-center text-dark-muted">
                <Send className="w-8 h-8 text-dark-muted/40 mb-2" />
                <span>Click "Send" to issue request and view upstream response</span>
              </div>
            ) : activeResViewTab === 'body' ? (
              <pre className="whitespace-pre-wrap text-emerald-300">
                {responseBodyStr || '[Empty Body]'}
              </pre>
            ) : (
              <div className="space-y-1">
                {activeResponse.headers.map(([k, v], i) => (
                  <div key={i}>
                    <span className="text-white font-semibold">{k}:</span>{' '}
                    <span className="text-dark-muted">{v}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
