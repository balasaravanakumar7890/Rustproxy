import React, { useState } from 'react';
import {
  FolderTree,
  ChevronDown,
  Globe,
  FileCode,
  Search,
} from 'lucide-react';
import { SiteMapNode } from '../types';

interface TargetViewProps {
  sitemap: SiteMapNode[];
  onRefresh: () => Promise<void>;
}

export const TargetView: React.FC<TargetViewProps> = ({ sitemap }) => {
  const [selectedNode, setSelectedNode] = useState<SiteMapNode | null>(null);
  const [filterQuery, setFilterQuery] = useState<string>('');

  const renderNode = (node: SiteMapNode, level: number = 0) => {
    const hasChildren = Object.keys(node.children).length > 0;
    const isSelected = selectedNode?.id === node.id;

    return (
      <div key={node.id} className="select-none">
        <div
          onClick={() => setSelectedNode(node)}
          style={{ paddingLeft: `${level * 16 + 8}px` }}
          className={`flex items-center justify-between py-1.5 pr-3 text-xs cursor-pointer rounded transition-colors ${
            isSelected
              ? 'bg-brand-primary/20 text-white font-semibold'
              : 'hover:bg-dark-cardHover text-dark-text'
          }`}
        >
          <div className="flex items-center space-x-1.5 truncate">
            {hasChildren ? (
              <ChevronDown className="w-3.5 h-3.5 text-dark-muted flex-shrink-0" />
            ) : (
              <FileCode className="w-3.5 h-3.5 text-brand-primary flex-shrink-0" />
            )}
            <span className="font-mono truncate">{node.name}</span>
          </div>

          <div className="flex items-center space-x-1.5 flex-shrink-0">
            {node.methods.map((m) => (
              <span
                key={m}
                className="px-1.5 py-0.2 rounded text-[10px] font-mono font-bold bg-dark-border text-dark-muted"
              >
                {m}
              </span>
            ))}
          </div>
        </div>

        {hasChildren && (
          <div className="border-l border-dark-border/40 ml-4">
            {Object.values(node.children).map((child) => renderNode(child, level + 1))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="flex h-[calc(100vh-3.5rem)] bg-dark-bg select-none">
      {/* Left Pane: Site Tree */}
      <div className="w-1/2 border-r border-dark-border flex flex-col bg-dark-card">
        <div className="p-3 border-b border-dark-border flex items-center space-x-2">
          <Search className="w-3.5 h-3.5 text-dark-muted" />
          <input
            type="text"
            value={filterQuery}
            onChange={(e) => setFilterQuery(e.target.value)}
            placeholder="Filter site hierarchy..."
            className="w-full bg-dark-bg border border-dark-border rounded px-2.5 py-1 text-xs text-dark-text outline-none focus:border-brand-primary font-mono"
          />
        </div>

        <div className="flex-1 overflow-y-auto p-2 space-y-1">
          {sitemap.length === 0 ? (
            <div className="py-12 text-center text-xs text-dark-muted">
              <FolderTree className="w-8 h-8 text-dark-muted/40 mx-auto mb-2" />
              <span>No endpoints mapped yet. Send requests through proxy to populate site map.</span>
            </div>
          ) : (
            sitemap.map((host) => renderNode(host, 0))
          )}
        </div>
      </div>

      {/* Right Pane: Selected Node Detail */}
      <div className="w-1/2 p-6 overflow-y-auto">
        {!selectedNode ? (
          <div className="h-full flex flex-col items-center justify-center text-center text-dark-muted">
            <Globe className="w-10 h-10 text-dark-muted/40 mb-3" />
            <h3 className="text-sm font-semibold text-dark-text">Select an Endpoint</h3>
            <p className="text-xs max-w-xs mt-1">
              Select any host or path in the hierarchy to inspect methods, status codes, and associated flows.
            </p>
          </div>
        ) : (
          <div className="space-y-6">
            <div className="bg-dark-card border border-dark-border rounded-xl p-5">
              <span className="text-[11px] font-bold text-brand-primary uppercase tracking-wider">
                {selectedNode.is_endpoint ? 'Endpoint Resource' : 'Path Segment'}
              </span>
              <h2 className="text-lg font-bold font-mono text-white mt-1">{selectedNode.path}</h2>

              <div className="mt-4 grid grid-cols-3 gap-3 text-xs">
                <div className="bg-dark-bg p-3 rounded-lg border border-dark-border">
                  <div className="text-dark-muted">Total Requests</div>
                  <div className="text-base font-bold font-mono text-white mt-1">
                    {selectedNode.flow_ids.length}
                  </div>
                </div>
                <div className="bg-dark-bg p-3 rounded-lg border border-dark-border">
                  <div className="text-dark-muted">Observed Methods</div>
                  <div className="text-base font-bold font-mono text-white mt-1">
                    {selectedNode.methods.join(', ') || '-'}
                  </div>
                </div>
                <div className="bg-dark-bg p-3 rounded-lg border border-dark-border">
                  <div className="text-dark-muted">Status Codes</div>
                  <div className="text-base font-bold font-mono text-white mt-1">
                    {selectedNode.status_codes.join(', ') || '-'}
                  </div>
                </div>
              </div>
            </div>

            {/* Child Resources list */}
            {Object.keys(selectedNode.children).length > 0 && (
              <div className="bg-dark-card border border-dark-border rounded-xl p-5">
                <h3 className="text-xs font-bold text-white uppercase tracking-wider mb-3">
                  Sub-paths ({Object.keys(selectedNode.children).length})
                </h3>
                <div className="space-y-2">
                  {Object.values(selectedNode.children).map((child) => (
                    <div
                      key={child.id}
                      onClick={() => setSelectedNode(child)}
                      className="flex items-center justify-between p-2.5 rounded-lg bg-dark-bg border border-dark-border text-xs cursor-pointer hover:border-dark-muted transition-colors"
                    >
                      <span className="font-mono text-dark-text">{child.name}</span>
                      <div className="flex items-center space-x-1.5">
                        {child.methods.map((m) => (
                          <span
                            key={m}
                            className="px-1.5 py-0.5 rounded text-[10px] bg-dark-border text-dark-muted font-bold"
                          >
                            {m}
                          </span>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
