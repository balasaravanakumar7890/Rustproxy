import React, { useState } from 'react';
import {
  Play,
  Square,
  Repeat,
  Plus,
  Trash2,
  CheckCircle,
  Radio,
} from 'lucide-react';
import { RequestModel } from '../types';

interface ProxyInterceptViewProps {
  interceptEnabled: boolean;
  pendingRequests: RequestModel[];
  onToggleIntercept: () => void;
  onForward: (id: string, modified?: RequestModel) => Promise<void>;
  onDrop: (id: string) => Promise<void>;
  onSendToRepeater: (req: RequestModel) => void;
}

export const ProxyInterceptView: React.FC<ProxyInterceptViewProps> = ({
  interceptEnabled,
  pendingRequests,
  onToggleIntercept,
  onForward,
  onDrop,
  onSendToRepeater,
}) => {
  const [selectedIdx, setSelectedIdx] = useState<number>(0);
  const currentReq = pendingRequests[selectedIdx];

  // Editable local state for the active request
  const [editableMethod, setEditableMethod] = useState<string>('GET');
  const [editableUri, setEditableUri] = useState<string>('');
  const [editableHeaders, setEditableHeaders] = useState<[string, string][]>([]);
  const [editableBody, setEditableBody] = useState<string>('');
  const [lastLoadedId, setLastLoadedId] = useState<string | null>(null);

  // Sync state when new request arrives
  if (currentReq && currentReq.id !== lastLoadedId) {
    setLastLoadedId(currentReq.id);
    setEditableMethod(currentReq.method);
    setEditableUri(currentReq.uri);
    setEditableHeaders([...currentReq.headers]);
    setEditableBody(currentReq.body ?? '');
  }

  const handleForwardClick = async () => {
    if (!currentReq) return;
    const modified: RequestModel = {
      ...currentReq,
      method: editableMethod,
      uri: editableUri,
      headers: editableHeaders,
      body: editableBody,
      body_length: editableBody.length,
    };
    await onForward(currentReq.id, modified);
    if (selectedIdx >= pendingRequests.length - 1) {
      setSelectedIdx(Math.max(0, pendingRequests.length - 2));
    }
  };

  const handleDropClick = async () => {
    if (!currentReq) return;
    await onDrop(currentReq.id);
    if (selectedIdx >= pendingRequests.length - 1) {
      setSelectedIdx(Math.max(0, pendingRequests.length - 2));
    }
  };

  const handleHeaderChange = (index: number, key: string, value: string) => {
    const updated = [...editableHeaders];
    updated[index] = [key, value];
    setEditableHeaders(updated);
  };

  const handleAddHeader = () => {
    setEditableHeaders([...editableHeaders, ['New-Header', 'Value']]);
  };

  const handleRemoveHeader = (index: number) => {
    setEditableHeaders(editableHeaders.filter((_, i) => i !== index));
  };

  return (
    <div className="flex flex-col h-[calc(100vh-3.5rem)] bg-dark-bg select-none">
      {/* Control Bar */}
      <div className="h-14 bg-dark-card border-b border-dark-border px-4 flex items-center justify-between">
        <div className="flex items-center space-x-3">
          {/* Main Toggle Switch */}
          <button
            onClick={onToggleIntercept}
            className={`px-3.5 py-1.5 rounded-lg text-xs font-bold flex items-center space-x-2 border shadow-sm transition-all ${
              interceptEnabled
                ? 'bg-brand-warning/15 border-brand-warning text-brand-warning hover:bg-brand-warning/25'
                : 'bg-dark-border/40 border-dark-border text-dark-muted hover:text-dark-text'
            }`}
          >
            <Radio className="w-4 h-4" />
            <span>{interceptEnabled ? 'INTERCEPT IS ON' : 'INTERCEPT IS OFF'}</span>
          </button>

          <div className="h-5 w-px bg-dark-border" />

          {/* Action Buttons */}
          <button
            onClick={handleForwardClick}
            disabled={!currentReq}
            className="px-3 py-1.5 rounded-md text-xs font-semibold bg-brand-primary text-white hover:bg-brand-hover disabled:opacity-40 disabled:hover:bg-brand-primary flex items-center space-x-1.5 shadow-sm transition-colors"
          >
            <Play className="w-3.5 h-3.5 fill-current" />
            <span>Forward (Ctrl+F)</span>
          </button>

          <button
            onClick={handleDropClick}
            disabled={!currentReq}
            className="px-3 py-1.5 rounded-md text-xs font-semibold bg-brand-danger/20 border border-brand-danger/30 text-brand-danger hover:bg-brand-danger/30 disabled:opacity-40 flex items-center space-x-1.5 transition-colors"
          >
            <Square className="w-3.5 h-3.5 fill-current" />
            <span>Drop (Ctrl+D)</span>
          </button>

          <button
            onClick={() => currentReq && onSendToRepeater(currentReq)}
            disabled={!currentReq}
            className="px-3 py-1.5 rounded-md text-xs font-semibold bg-dark-cardHover border border-dark-border text-dark-text hover:border-dark-muted disabled:opacity-40 flex items-center space-x-1.5 transition-colors"
          >
            <Repeat className="w-3.5 h-3.5" />
            <span>Send to Repeater</span>
          </button>
        </div>

        {/* Pending Queue Count */}
        <div className="flex items-center space-x-2 text-xs">
          <span className="text-dark-muted">Pending in queue:</span>
          <span className="px-2 py-0.5 rounded-full font-mono font-bold bg-dark-border text-dark-text text-[11px]">
            {pendingRequests.length}
          </span>
        </div>
      </div>

      {/* Main Workspace */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left: Pending Requests Queue Selector */}
        {pendingRequests.length > 1 && (
          <div className="w-64 border-r border-dark-border bg-dark-card overflow-y-auto">
            <div className="p-2 text-[11px] font-bold text-dark-muted uppercase tracking-wider border-b border-dark-border">
              Intercepted Queue
            </div>
            <div className="divide-y divide-dark-border/40">
              {pendingRequests.map((req, idx) => (
                <div
                  key={req.id}
                  onClick={() => setSelectedIdx(idx)}
                  className={`p-2.5 text-xs cursor-pointer transition-colors ${
                    selectedIdx === idx
                      ? 'bg-dark-border text-white'
                      : 'text-dark-muted hover:bg-dark-cardHover hover:text-dark-text'
                  }`}
                >
                  <div className="flex items-center justify-between font-mono font-bold">
                    <span>{req.method}</span>
                    <span className="text-[10px] text-dark-muted">{req.host}</span>
                  </div>
                  <div className="text-[11px] truncate font-mono mt-0.5">{req.path}</div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Center: Request Editor / Viewer */}
        <div className="flex-1 flex flex-col overflow-hidden p-4 space-y-4">
          {!currentReq ? (
            <div className="flex-1 flex flex-col items-center justify-center text-center text-dark-muted p-8">
              <CheckCircle className="w-12 h-12 text-dark-muted/40 mb-3" />
              <h3 className="text-base font-semibold text-dark-text">No Requests Pending</h3>
              <p className="text-xs max-w-sm mt-1 text-dark-muted">
                {interceptEnabled
                  ? 'Traffic through proxy will pause here for inspection and modification before being dispatched upstream.'
                  : 'Interception is currently turned OFF. Traffic passes through automatically to the HTTP History.'}
              </p>
            </div>
          ) : (
            <div className="flex-1 flex flex-col bg-dark-card border border-dark-border rounded-xl p-4 overflow-hidden space-y-4">
              {/* Method and URI bar */}
              <div className="flex items-center space-x-2">
                <select
                  value={editableMethod}
                  onChange={(e) => setEditableMethod(e.target.value)}
                  className="bg-dark-bg border border-dark-border text-white text-xs font-mono font-bold rounded-lg px-3 py-2 outline-none focus:border-brand-primary"
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
                  value={editableUri}
                  onChange={(e) => setEditableUri(e.target.value)}
                  className="flex-1 bg-dark-bg border border-dark-border text-dark-text text-xs font-mono rounded-lg px-3 py-2 outline-none focus:border-brand-primary"
                  placeholder="Target URI..."
                />
              </div>

              {/* Headers Table */}
              <div className="flex-1 flex flex-col min-h-0 border border-dark-border/60 rounded-lg p-3 bg-dark-bg/60 overflow-hidden">
                <div className="flex items-center justify-between pb-2 mb-2 border-b border-dark-border text-xs text-dark-muted">
                  <span className="font-semibold text-white">HTTP Headers</span>
                  <button
                    onClick={handleAddHeader}
                    className="flex items-center space-x-1 text-brand-primary hover:underline text-[11px]"
                  >
                    <Plus className="w-3 h-3" />
                    <span>Add Header</span>
                  </button>
                </div>

                <div className="flex-1 overflow-y-auto space-y-2 pr-1">
                  {editableHeaders.map(([k, v], i) => (
                    <div key={i} className="flex items-center space-x-2 text-xs font-mono">
                      <input
                        type="text"
                        value={k}
                        onChange={(e) => handleHeaderChange(i, e.target.value, v)}
                        className="w-1/3 bg-dark-card border border-dark-border rounded px-2 py-1 text-dark-text outline-none focus:border-brand-primary"
                        placeholder="Header..."
                      />
                      <span className="text-dark-muted">:</span>
                      <input
                        type="text"
                        value={v}
                        onChange={(e) => handleHeaderChange(i, k, e.target.value)}
                        className="flex-1 bg-dark-card border border-dark-border rounded px-2 py-1 text-dark-text outline-none focus:border-brand-primary"
                        placeholder="Value..."
                      />
                      <button
                        onClick={() => handleRemoveHeader(i)}
                        className="text-dark-muted hover:text-brand-danger p-1"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  ))}
                </div>
              </div>

              {/* Request Body Editor */}
              <div className="h-44 flex flex-col border border-dark-border/60 rounded-lg p-3 bg-dark-bg/60">
                <div className="text-xs font-semibold text-white mb-2">Request Body Payload</div>
                <textarea
                  value={editableBody}
                  onChange={(e) => setEditableBody(e.target.value)}
                  className="flex-1 bg-dark-card border border-dark-border rounded p-2 text-xs font-mono text-dark-text outline-none focus:border-brand-primary resize-none"
                  placeholder="Request body (JSON, XML, or raw text)..."
                />
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
