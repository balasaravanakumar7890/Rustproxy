import React, { useState } from 'react';
import {
  Play,
  Square,
  Repeat,
  Plus,
  Trash2,
  CheckCircle,
  Radio,
  ArrowUpRight,
  ArrowDownLeft,
} from 'lucide-react';
import { RequestModel, ResponseModel } from '../types';

interface ProxyInterceptViewProps {
  interceptEnabled: boolean;
  pendingRequests: RequestModel[];
  pendingResponses?: ResponseModel[];
  onToggleIntercept: () => void;
  onForward: (id: string, modified?: RequestModel) => Promise<void>;
  onDrop: (id: string) => Promise<void>;
  onForwardResponse?: (id: string, modified?: ResponseModel) => Promise<void>;
  onDropResponse?: (id: string) => Promise<void>;
  onSendToRepeater: (req: RequestModel) => void;
}

export const ProxyInterceptView: React.FC<ProxyInterceptViewProps> = ({
  interceptEnabled,
  pendingRequests,
  pendingResponses = [],
  onToggleIntercept,
  onForward,
  onDrop,
  onForwardResponse,
  onDropResponse,
  onSendToRepeater,
}) => {
  const [activeTab, setActiveTab] = useState<'requests' | 'responses'>('requests');
  const [selectedReqIdx, setSelectedReqIdx] = useState<number>(0);
  const [selectedResIdx, setSelectedResIdx] = useState<number>(0);

  // Request editable state
  const currentReq = pendingRequests[selectedReqIdx];
  const [editableMethod, setEditableMethod] = useState<string>('GET');
  const [editableUri, setEditableUri] = useState<string>('');
  const [editableReqHeaders, setEditableReqHeaders] = useState<[string, string][]>([]);
  const [editableReqBody, setEditableReqBody] = useState<string>('');
  const [lastLoadedReqId, setLastLoadedReqId] = useState<string | null>(null);

  if (currentReq && currentReq.id !== lastLoadedReqId) {
    setLastLoadedReqId(currentReq.id);
    setEditableMethod(currentReq.method);
    setEditableUri(currentReq.uri);
    setEditableReqHeaders([...currentReq.headers]);
    setEditableReqBody(currentReq.body ?? '');
  }

  // Response editable state
  const currentRes = pendingResponses[selectedResIdx];
  const [editableStatus, setEditableStatus] = useState<number>(200);
  const [editableStatusText, setEditableStatusText] = useState<string>('OK');
  const [editableResHeaders, setEditableResHeaders] = useState<[string, string][]>([]);
  const [editableResBody, setEditableResBody] = useState<string>('');
  const [lastLoadedResId, setLastLoadedResId] = useState<string | null>(null);

  if (currentRes && currentRes.id !== lastLoadedResId) {
    setLastLoadedResId(currentRes.id);
    setEditableStatus(currentRes.status);
    setEditableStatusText(currentRes.status_text);
    setEditableResHeaders([...currentRes.headers]);
    setEditableResBody(currentRes.body ?? '');
  }

  const handleForwardRequest = async () => {
    if (!currentReq) return;
    const modified: RequestModel = {
      ...currentReq,
      method: editableMethod,
      uri: editableUri,
      headers: editableReqHeaders,
      body: editableReqBody,
      body_length: editableReqBody.length,
    };
    await onForward(currentReq.id, modified);
    if (selectedReqIdx >= pendingRequests.length - 1) {
      setSelectedReqIdx(Math.max(0, pendingRequests.length - 2));
    }
  };

  const handleDropRequest = async () => {
    if (!currentReq) return;
    await onDrop(currentReq.id);
    if (selectedReqIdx >= pendingRequests.length - 1) {
      setSelectedReqIdx(Math.max(0, pendingRequests.length - 2));
    }
  };

  const handleForwardResponse = async () => {
    if (!currentRes || !onForwardResponse) return;
    const modified: ResponseModel = {
      ...currentRes,
      status: editableStatus,
      status_text: editableStatusText,
      headers: editableResHeaders,
      body: editableResBody,
      body_length: editableResBody.length,
    };
    await onForwardResponse(currentRes.id, modified);
    if (selectedResIdx >= pendingResponses.length - 1) {
      setSelectedResIdx(Math.max(0, pendingResponses.length - 2));
    }
  };

  const handleDropResponse = async () => {
    if (!currentRes || !onDropResponse) return;
    await onDropResponse(currentRes.id);
    if (selectedResIdx >= pendingResponses.length - 1) {
      setSelectedResIdx(Math.max(0, pendingResponses.length - 2));
    }
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

          {/* Sub-Tabs: Requests vs Responses */}
          <div className="flex items-center bg-dark-bg border border-dark-border rounded-lg p-0.5 text-xs font-medium">
            <button
              onClick={() => setActiveTab('requests')}
              className={`px-3 py-1 rounded-md flex items-center space-x-1.5 transition-colors ${
                activeTab === 'requests'
                  ? 'bg-dark-card text-brand-primary shadow-sm'
                  : 'text-dark-muted hover:text-dark-text'
              }`}
            >
              <ArrowUpRight className="w-3.5 h-3.5" />
              <span>Requests ({pendingRequests.length})</span>
            </button>
            <button
              onClick={() => setActiveTab('responses')}
              className={`px-3 py-1 rounded-md flex items-center space-x-1.5 transition-colors ${
                activeTab === 'responses'
                  ? 'bg-dark-card text-brand-info shadow-sm'
                  : 'text-dark-muted hover:text-dark-text'
              }`}
            >
              <ArrowDownLeft className="w-3.5 h-3.5" />
              <span>Responses ({pendingResponses.length})</span>
            </button>
          </div>

          <div className="h-5 w-px bg-dark-border" />

          {/* Action Buttons */}
          {activeTab === 'requests' ? (
            <>
              <button
                onClick={handleForwardRequest}
                disabled={!currentReq}
                className="px-3 py-1.5 rounded-md text-xs font-semibold bg-brand-primary text-white hover:bg-brand-hover disabled:opacity-40 flex items-center space-x-1.5 shadow-sm transition-colors"
              >
                <Play className="w-3.5 h-3.5 fill-current" />
                <span>Forward Request</span>
              </button>

              <button
                onClick={handleDropRequest}
                disabled={!currentReq}
                className="px-3 py-1.5 rounded-md text-xs font-semibold bg-brand-danger/20 border border-brand-danger/30 text-brand-danger hover:bg-brand-danger/30 disabled:opacity-40 flex items-center space-x-1.5 transition-colors"
              >
                <Square className="w-3.5 h-3.5 fill-current" />
                <span>Drop Request</span>
              </button>

              <button
                onClick={() => currentReq && onSendToRepeater(currentReq)}
                disabled={!currentReq}
                className="px-3 py-1.5 rounded-md text-xs font-semibold bg-dark-cardHover border border-dark-border text-dark-text hover:border-dark-muted disabled:opacity-40 flex items-center space-x-1.5 transition-colors"
              >
                <Repeat className="w-3.5 h-3.5" />
                <span>Send to Repeater</span>
              </button>
            </>
          ) : (
            <>
              <button
                onClick={handleForwardResponse}
                disabled={!currentRes}
                className="px-3 py-1.5 rounded-md text-xs font-semibold bg-brand-primary text-white hover:bg-brand-hover disabled:opacity-40 flex items-center space-x-1.5 shadow-sm transition-colors"
              >
                <Play className="w-3.5 h-3.5 fill-current" />
                <span>Forward Response</span>
              </button>

              <button
                onClick={handleDropResponse}
                disabled={!currentRes}
                className="px-3 py-1.5 rounded-md text-xs font-semibold bg-brand-danger/20 border border-brand-danger/30 text-brand-danger hover:bg-brand-danger/30 disabled:opacity-40 flex items-center space-x-1.5 transition-colors"
              >
                <Square className="w-3.5 h-3.5 fill-current" />
                <span>Drop Response</span>
              </button>
            </>
          )}
        </div>

        {/* Pending Queue Count */}
        <div className="flex items-center space-x-2 text-xs">
          <span className="text-dark-muted">Active items:</span>
          <span className="px-2 py-0.5 rounded-full font-mono font-bold bg-dark-border text-dark-text text-[11px]">
            {activeTab === 'requests' ? pendingRequests.length : pendingResponses.length}
          </span>
        </div>
      </div>

      {/* Main Workspace */}
      <div className="flex-1 flex overflow-hidden">
        {activeTab === 'requests' ? (
          /* Request View */
          <>
            {pendingRequests.length > 1 && (
              <div className="w-64 border-r border-dark-border bg-dark-card overflow-y-auto">
                <div className="p-2 text-[11px] font-bold text-dark-muted uppercase tracking-wider border-b border-dark-border">
                  Pending Requests
                </div>
                <div className="divide-y divide-dark-border/40">
                  {pendingRequests.map((req, idx) => (
                    <div
                      key={req.id}
                      onClick={() => setSelectedReqIdx(idx)}
                      className={`p-2.5 text-xs cursor-pointer transition-colors ${
                        selectedReqIdx === idx
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

            <div className="flex-1 flex flex-col overflow-hidden p-4 space-y-4">
              {!currentReq ? (
                <div className="flex-1 flex flex-col items-center justify-center text-center text-dark-muted p-8">
                  <CheckCircle className="w-12 h-12 text-dark-muted/40 mb-3" />
                  <h3 className="text-base font-semibold text-dark-text">No Requests Pending</h3>
                  <p className="text-xs max-w-sm mt-1 text-dark-muted">
                    {interceptEnabled
                      ? 'Outbound HTTP requests will pause here for inspection and modification before reaching upstream.'
                      : 'Interception is currently turned OFF. Traffic passes through automatically.'}
                  </p>
                </div>
              ) : (
                <div className="flex-1 flex flex-col bg-dark-card border border-dark-border rounded-xl p-4 overflow-hidden space-y-4">
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

                  <div className="flex-1 flex flex-col min-h-0 border border-dark-border/60 rounded-lg p-3 bg-dark-bg/60 overflow-hidden">
                    <div className="flex items-center justify-between pb-2 mb-2 border-b border-dark-border text-xs text-dark-muted">
                      <span className="font-semibold text-white">HTTP Headers</span>
                      <button
                        onClick={() => setEditableReqHeaders([...editableReqHeaders, ['New-Header', 'Value']])}
                        className="flex items-center space-x-1 text-brand-primary hover:underline text-[11px]"
                      >
                        <Plus className="w-3 h-3" />
                        <span>Add Header</span>
                      </button>
                    </div>

                    <div className="flex-1 overflow-y-auto space-y-2 pr-1">
                      {editableReqHeaders.map(([k, v], i) => (
                        <div key={i} className="flex items-center space-x-2 text-xs font-mono">
                          <input
                            type="text"
                            value={k}
                            onChange={(e) => {
                              const updated = [...editableReqHeaders];
                              updated[i] = [e.target.value, v];
                              setEditableReqHeaders(updated);
                            }}
                            className="w-1/3 bg-dark-card border border-dark-border rounded px-2 py-1 text-dark-text outline-none focus:border-brand-primary"
                          />
                          <span className="text-dark-muted">:</span>
                          <input
                            type="text"
                            value={v}
                            onChange={(e) => {
                              const updated = [...editableReqHeaders];
                              updated[i] = [k, e.target.value];
                              setEditableReqHeaders(updated);
                            }}
                            className="flex-1 bg-dark-card border border-dark-border rounded px-2 py-1 text-dark-text outline-none focus:border-brand-primary"
                          />
                          <button
                            onClick={() => setEditableReqHeaders(editableReqHeaders.filter((_, idx) => idx !== i))}
                            className="text-dark-muted hover:text-brand-danger p-1"
                          >
                            <Trash2 className="w-3.5 h-3.5" />
                          </button>
                        </div>
                      ))}
                    </div>
                  </div>

                  <div className="h-44 flex flex-col border border-dark-border/60 rounded-lg p-3 bg-dark-bg/60">
                    <div className="text-xs font-semibold text-white mb-2">Request Body Payload</div>
                    <textarea
                      value={editableReqBody}
                      onChange={(e) => setEditableReqBody(e.target.value)}
                      className="flex-1 bg-dark-card border border-dark-border rounded p-2 text-xs font-mono text-dark-text outline-none focus:border-brand-primary resize-none"
                      placeholder="Request body (JSON, XML, or raw text)..."
                    />
                  </div>
                </div>
              )}
            </div>
          </>
        ) : (
          /* Response View */
          <>
            {pendingResponses.length > 1 && (
              <div className="w-64 border-r border-dark-border bg-dark-card overflow-y-auto">
                <div className="p-2 text-[11px] font-bold text-dark-muted uppercase tracking-wider border-b border-dark-border">
                  Pending Responses
                </div>
                <div className="divide-y divide-dark-border/40">
                  {pendingResponses.map((res, idx) => (
                    <div
                      key={res.id}
                      onClick={() => setSelectedResIdx(idx)}
                      className={`p-2.5 text-xs cursor-pointer transition-colors ${
                        selectedResIdx === idx
                          ? 'bg-dark-border text-white'
                          : 'text-dark-muted hover:bg-dark-cardHover hover:text-dark-text'
                      }`}
                    >
                      <div className="flex items-center justify-between font-mono font-bold">
                        <span className="text-brand-info">{res.status} {res.status_text}</span>
                        <span className="text-[10px] text-dark-muted">{res.body_length} B</span>
                      </div>
                      <div className="text-[11px] truncate font-mono mt-0.5 text-dark-muted">
                        {res.mime_type || 'untyped'}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div className="flex-1 flex flex-col overflow-hidden p-4 space-y-4">
              {!currentRes ? (
                <div className="flex-1 flex flex-col items-center justify-center text-center text-dark-muted p-8">
                  <CheckCircle className="w-12 h-12 text-dark-muted/40 mb-3" />
                  <h3 className="text-base font-semibold text-dark-text">No Responses Pending</h3>
                  <p className="text-xs max-w-sm mt-1 text-dark-muted">
                    Inbound server responses matching interception rules will pause here for inspection and tampering before reaching client applications.
                  </p>
                </div>
              ) : (
                <div className="flex-1 flex flex-col bg-dark-card border border-dark-border rounded-xl p-4 overflow-hidden space-y-4">
                  <div className="flex items-center space-x-2">
                    <input
                      type="number"
                      value={editableStatus}
                      onChange={(e) => setEditableStatus(parseInt(e.target.value, 10) || 200)}
                      className="w-24 bg-dark-bg border border-dark-border text-white text-xs font-mono font-bold rounded-lg px-3 py-2 outline-none focus:border-brand-primary"
                      placeholder="Status"
                    />
                    <input
                      type="text"
                      value={editableStatusText}
                      onChange={(e) => setEditableStatusText(e.target.value)}
                      className="w-48 bg-dark-bg border border-dark-border text-white text-xs font-mono rounded-lg px-3 py-2 outline-none focus:border-brand-primary"
                      placeholder="Status text..."
                    />
                    <div className="text-xs text-dark-muted ml-auto font-mono">
                      MIME: {currentRes.mime_type || 'none'}
                    </div>
                  </div>

                  <div className="flex-1 flex flex-col min-h-0 border border-dark-border/60 rounded-lg p-3 bg-dark-bg/60 overflow-hidden">
                    <div className="flex items-center justify-between pb-2 mb-2 border-b border-dark-border text-xs text-dark-muted">
                      <span className="font-semibold text-white">Response Headers</span>
                      <button
                        onClick={() => setEditableResHeaders([...editableResHeaders, ['New-Header', 'Value']])}
                        className="flex items-center space-x-1 text-brand-primary hover:underline text-[11px]"
                      >
                        <Plus className="w-3 h-3" />
                        <span>Add Header</span>
                      </button>
                    </div>

                    <div className="flex-1 overflow-y-auto space-y-2 pr-1">
                      {editableResHeaders.map(([k, v], i) => (
                        <div key={i} className="flex items-center space-x-2 text-xs font-mono">
                          <input
                            type="text"
                            value={k}
                            onChange={(e) => {
                              const updated = [...editableResHeaders];
                              updated[i] = [e.target.value, v];
                              setEditableResHeaders(updated);
                            }}
                            className="w-1/3 bg-dark-card border border-dark-border rounded px-2 py-1 text-dark-text outline-none focus:border-brand-primary"
                          />
                          <span className="text-dark-muted">:</span>
                          <input
                            type="text"
                            value={v}
                            onChange={(e) => {
                              const updated = [...editableResHeaders];
                              updated[i] = [k, e.target.value];
                              setEditableResHeaders(updated);
                            }}
                            className="flex-1 bg-dark-card border border-dark-border rounded px-2 py-1 text-dark-text outline-none focus:border-brand-primary"
                          />
                          <button
                            onClick={() => setEditableResHeaders(editableResHeaders.filter((_, idx) => idx !== i))}
                            className="text-dark-muted hover:text-brand-danger p-1"
                          >
                            <Trash2 className="w-3.5 h-3.5" />
                          </button>
                        </div>
                      ))}
                    </div>
                  </div>

                  <div className="h-44 flex flex-col border border-dark-border/60 rounded-lg p-3 bg-dark-bg/60">
                    <div className="text-xs font-semibold text-white mb-2">Response Body Payload</div>
                    <textarea
                      value={editableResBody}
                      onChange={(e) => setEditableResBody(e.target.value)}
                      className="flex-1 bg-dark-card border border-dark-border rounded p-2 text-xs font-mono text-dark-text outline-none focus:border-brand-primary resize-none"
                      placeholder="Response body payload..."
                    />
                  </div>
                </div>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
};
