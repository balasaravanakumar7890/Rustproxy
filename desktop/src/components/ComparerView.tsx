import React, { useState } from 'react';
import { GitCompare } from 'lucide-react';
import { DiffResult } from '../types';
import { api } from '../api/client';

export const ComparerView: React.FC = () => {
  const [text1, setText1] = useState<string>(
    'HTTP/1.1 200 OK\nContent-Type: application/json\n\n{"status": "authenticated", "admin": true}'
  );
  const [text2, setText2] = useState<string>(
    'HTTP/1.1 403 Forbidden\nContent-Type: application/json\n\n{"status": "unauthorized", "admin": false}'
  );
  const [diffResult, setDiffResult] = useState<DiffResult | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  const handleCompare = async () => {
    setIsLoading(true);
    try {
      const res = await api.diff(text1, text2);
      setDiffResult(res);
    } catch (err: any) {
      alert(`Diff Error: ${err.message}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="p-6 space-y-6 h-[calc(100vh-3.5rem)] overflow-y-auto bg-dark-bg select-none">
      {/* Header */}
      <div className="bg-dark-card border border-dark-border rounded-xl p-5 flex items-center justify-between">
        <div>
          <h2 className="text-sm font-bold text-white flex items-center space-x-2">
            <GitCompare className="w-4 h-4 text-brand-primary" />
            <span>Differential Comparer</span>
          </h2>
          <p className="text-xs text-dark-muted mt-1">
            Compare two HTTP responses or payloads to quickly identify authorization discrepancies or parameter reflections.
          </p>
        </div>

        <button
          onClick={handleCompare}
          disabled={isLoading}
          className="px-4 py-2 rounded-lg text-xs font-bold bg-brand-primary text-white hover:bg-brand-hover disabled:opacity-40 flex items-center space-x-1.5 shadow-sm transition-colors"
        >
          <GitCompare className="w-4 h-4" />
          <span>{isLoading ? 'Comparing...' : 'Run Diff'}</span>
        </button>
      </div>

      {/* Input Split */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 h-64">
        <div className="flex flex-col bg-dark-card border border-dark-border rounded-xl p-3 space-y-2">
          <div className="text-xs font-semibold text-white">Item 1 (Original)</div>
          <textarea
            value={text1}
            onChange={(e) => setText1(e.target.value)}
            className="flex-1 bg-dark-bg border border-dark-border rounded-lg p-2.5 font-mono text-xs text-dark-text outline-none focus:border-brand-primary resize-none"
          />
        </div>

        <div className="flex flex-col bg-dark-card border border-dark-border rounded-xl p-3 space-y-2">
          <div className="text-xs font-semibold text-white">Item 2 (Modified)</div>
          <textarea
            value={text2}
            onChange={(e) => setText2(e.target.value)}
            className="flex-1 bg-dark-bg border border-dark-border rounded-lg p-2.5 font-mono text-xs text-dark-text outline-none focus:border-brand-primary resize-none"
          />
        </div>
      </div>

      {/* Diff Result Viewer */}
      {diffResult && (
        <div className="bg-dark-card border border-dark-border rounded-xl p-5 space-y-3">
          <div className="flex items-center justify-between pb-3 border-b border-dark-border">
            <div className="flex items-center space-x-3 text-xs">
              <span className="font-bold text-white">Differential Analysis</span>
              <span className="px-2 py-0.5 rounded bg-brand-accent/20 text-brand-accent font-mono font-bold">
                + {diffResult.additions} additions
              </span>
              <span className="px-2 py-0.5 rounded bg-brand-danger/20 text-brand-danger font-mono font-bold">
                - {diffResult.deletions} deletions
              </span>
            </div>

            <div className="text-xs text-dark-muted font-mono">
              Similarity: {(diffResult.similarity_ratio * 100).toFixed(1)}%
            </div>
          </div>

          <div className="bg-dark-bg border border-dark-border rounded-lg p-3 font-mono text-xs space-y-0.5 max-h-80 overflow-y-auto">
            {diffResult.lines.map((line, i) => (
              <div
                key={i}
                className={`py-0.5 px-2 rounded flex items-start space-x-3 ${
                  line.tag === 'Insert'
                    ? 'bg-emerald-950/60 text-emerald-300'
                    : line.tag === 'Delete'
                    ? 'bg-rose-950/60 text-rose-300'
                    : 'text-dark-text'
                }`}
              >
                <span className="w-5 text-right font-bold text-dark-muted select-none">
                  {line.tag === 'Insert' ? '+' : line.tag === 'Delete' ? '-' : ' '}
                </span>
                <span className="whitespace-pre-wrap">{line.content}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
