import React, { useState } from 'react';
import {
  ShieldAlert,
  ShieldCheck,
  BookOpen,
  Wrench,
} from 'lucide-react';
import { SecurityFinding } from '../types';

interface ScannerViewProps {
  findings: SecurityFinding[];
  onRefresh: () => Promise<void>;
}

export const ScannerView: React.FC<ScannerViewProps> = ({ findings }) => {
  const [selectedFindingId, setSelectedFindingId] = useState<string | null>(null);
  const [severityFilter, setSeverityFilter] = useState<string>('ALL');

  const filteredFindings = findings.filter((f) => {
    if (severityFilter !== 'ALL' && f.severity !== severityFilter) return false;
    return true;
  });

  const selectedFinding = findings.find((f) => f.id === selectedFindingId);

  return (
    <div className="flex h-[calc(100vh-3.5rem)] bg-dark-bg select-none">
      {/* Left List */}
      <div className="w-1/2 border-r border-dark-border flex flex-col bg-dark-card">
        {/* Severity Filter Bar */}
        <div className="p-3 border-b border-dark-border flex items-center justify-between">
          <div className="flex items-center space-x-1">
            {['ALL', 'Critical', 'High', 'Medium', 'Low', 'Info'].map((sev) => (
              <button
                key={sev}
                onClick={() => setSeverityFilter(sev)}
                className={`px-2.5 py-1 rounded text-xs font-semibold transition-colors ${
                  severityFilter === sev
                    ? 'bg-dark-border text-white'
                    : 'text-dark-muted hover:text-dark-text'
                }`}
              >
                {sev}
              </button>
            ))}
          </div>

          <span className="text-xs text-dark-muted">
            {filteredFindings.length} findings
          </span>
        </div>

        {/* Findings Items */}
        <div className="flex-1 overflow-y-auto divide-y divide-dark-border/40">
          {filteredFindings.length === 0 ? (
            <div className="p-12 text-center text-xs text-dark-muted">
              <ShieldCheck className="w-10 h-10 text-brand-accent/40 mx-auto mb-2" />
              <span>No findings matching selected criteria.</span>
            </div>
          ) : (
            filteredFindings.map((f) => {
              const isSelected = selectedFindingId === f.id;
              const sevBadgeColor =
                f.severity === 'Critical' || f.severity === 'High'
                  ? 'bg-brand-danger/20 text-brand-danger border-brand-danger/30'
                  : f.severity === 'Medium'
                  ? 'bg-brand-warning/20 text-brand-warning border-brand-warning/30'
                  : 'bg-brand-primary/20 text-brand-primary border-brand-primary/30';

              return (
                <div
                  key={f.id}
                  onClick={() => setSelectedFindingId(f.id)}
                  className={`p-3 text-xs cursor-pointer transition-colors ${
                    isSelected
                      ? 'bg-brand-primary/20 text-white'
                      : 'hover:bg-dark-cardHover text-dark-text'
                  }`}
                >
                  <div className="flex items-center justify-between mb-1">
                    <span
                      className={`px-2 py-0.5 rounded text-[10px] font-bold uppercase border ${sevBadgeColor}`}
                    >
                      {f.severity}
                    </span>
                    <span className="text-[11px] text-dark-muted font-mono">
                      {f.cwe_id ? `CWE-${f.cwe_id}` : ''}
                    </span>
                  </div>

                  <h4 className="font-semibold text-white text-xs">{f.title}</h4>
                  <div className="text-[11px] text-dark-muted font-mono truncate mt-0.5">
                    {f.url}
                  </div>
                </div>
              );
            })
          )}
        </div>
      </div>

      {/* Right Detail Pane */}
      <div className="w-1/2 p-6 overflow-y-auto">
        {!selectedFinding ? (
          <div className="h-full flex flex-col items-center justify-center text-center text-dark-muted">
            <ShieldAlert className="w-10 h-10 text-dark-muted/40 mb-3" />
            <h3 className="text-sm font-semibold text-dark-text">Select a Security Finding</h3>
            <p className="text-xs max-w-xs mt-1">
              Select any finding from the list to view vulnerability details, CWE references, remediation guidance, and evidence.
            </p>
          </div>
        ) : (
          <div className="space-y-6">
            <div className="bg-dark-card border border-dark-border rounded-xl p-5 space-y-3">
              <div className="flex items-center justify-between">
                <span
                  className={`px-2.5 py-0.5 rounded text-xs font-bold uppercase ${
                    selectedFinding.severity === 'Critical' || selectedFinding.severity === 'High'
                      ? 'bg-brand-danger/20 text-brand-danger'
                      : selectedFinding.severity === 'Medium'
                      ? 'bg-brand-warning/20 text-brand-warning'
                      : 'bg-brand-primary/20 text-brand-primary'
                  }`}
                >
                  {selectedFinding.severity} Severity
                </span>
                {selectedFinding.cwe_id && (
                  <span className="px-2 py-0.5 rounded bg-dark-bg border border-dark-border text-xs font-mono text-dark-text">
                    CWE-{selectedFinding.cwe_id}
                  </span>
                )}
              </div>

              <h2 className="text-lg font-bold text-white">{selectedFinding.title}</h2>
              <div className="text-xs font-mono text-dark-muted truncate">{selectedFinding.url}</div>
            </div>

            {/* Description Card */}
            <div className="bg-dark-card border border-dark-border rounded-xl p-5 space-y-2">
              <h3 className="text-xs font-bold text-white uppercase tracking-wider flex items-center space-x-1.5">
                <BookOpen className="w-3.5 h-3.5 text-brand-primary" />
                <span>Vulnerability Description</span>
              </h3>
              <p className="text-xs text-dark-text leading-relaxed">{selectedFinding.description}</p>
            </div>

            {/* Remediation Card */}
            {selectedFinding.remediation && (
              <div className="bg-dark-card border border-dark-border rounded-xl p-5 space-y-2">
                <h3 className="text-xs font-bold text-brand-accent uppercase tracking-wider flex items-center space-x-1.5">
                  <Wrench className="w-3.5 h-3.5 text-brand-accent" />
                  <span>Remediation Guidance</span>
                </h3>
                <p className="text-xs text-dark-text leading-relaxed">
                  {selectedFinding.remediation}
                </p>
              </div>
            )}

            {/* Evidence Card */}
            {selectedFinding.evidence && (
              <div className="bg-dark-card border border-dark-border rounded-xl p-5 space-y-2">
                <h3 className="text-xs font-bold text-brand-warning uppercase tracking-wider">
                  Collected Evidence
                </h3>
                <pre className="p-3 bg-dark-bg border border-dark-border rounded-lg text-xs font-mono text-amber-300 overflow-x-auto whitespace-pre-wrap">
                  {selectedFinding.evidence}
                </pre>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
