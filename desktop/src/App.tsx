import React, { useState, useEffect, useCallback } from 'react';
import { Navbar, NavTab } from './components/Navbar';
import { DashboardView } from './components/DashboardView';
import { ProxyInterceptView } from './components/ProxyInterceptView';
import { HistoryView } from './components/HistoryView';
import { RepeaterView } from './components/RepeaterView';
import { TargetView } from './components/TargetView';
import { ScannerView } from './components/ScannerView';
import { DecoderView } from './components/DecoderView';
import { ComparerView } from './components/ComparerView';
import { SettingsView } from './components/SettingsView';
import {
  FlowSummary,
  ProxyStats,
  RepeaterTab,
  RequestModel,
  SecurityFinding,
  SiteMapNode,
} from './types';
import { api } from './api/client';

export const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState<NavTab>('dashboard');
  const [stats, setStats] = useState<ProxyStats | null>(null);
  const [flows, setFlows] = useState<FlowSummary[]>([]);
  const [findings, setFindings] = useState<SecurityFinding[]>([]);
  const [sitemap, setSitemap] = useState<SiteMapNode[]>([]);
  const [pendingRequests, setPendingRequests] = useState<RequestModel[]>([]);
  const [repeaterTabs, setRepeaterTabs] = useState<RepeaterTab[]>([]);
  const [repeaterInitialReq, setRepeaterInitialReq] = useState<RequestModel | null>(null);

  // Poll backend state regularly for real-time telemetry
  const refreshAll = useCallback(async () => {
    try {
      const [s, f, findingsData, sm, ic, repTabs] = await Promise.all([
        api.getStats().catch(() => null),
        api.getFlows({ limit: 500 }).catch(() => []),
        api.getFindings().catch(() => []),
        api.getSitemap().catch(() => []),
        api.getInterceptStatus().catch(() => ({ enabled: false, pending: [] })),
        api.getRepeaterTabs().catch(() => []),
      ]);

      if (s) setStats(s);
      setFlows(f);
      setFindings(findingsData);
      setSitemap(sm);
      setPendingRequests(ic.pending);
      setRepeaterTabs(repTabs);
    } catch (err) {
      // Backend starting or reconnecting
    }
  }, []);

  useEffect(() => {
    refreshAll();
    const interval = setInterval(refreshAll, 1500);
    return () => clearInterval(interval);
  }, [refreshAll]);

  // Global Keyboard Shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'r') {
        e.preventDefault();
        setActiveTab('repeater');
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const handleToggleIntercept = async () => {
    const current = stats?.intercept_enabled ?? false;
    await api.toggleIntercept(!current);
    await refreshAll();
  };

  const handleForwardIntercept = async (id: string, modified?: RequestModel) => {
    await api.forwardIntercept(id, modified);
    await refreshAll();
  };

  const handleDropIntercept = async (id: string) => {
    await api.dropIntercept(id);
    await refreshAll();
  };

  const handleSendToRepeater = (req: RequestModel) => {
    setRepeaterInitialReq(req);
    setActiveTab('repeater');
  };

  const handleSendToComparer = (_text: string) => {
    setActiveTab('comparer');
  };

  return (
    <div className="flex flex-col h-screen w-screen bg-dark-bg text-dark-text overflow-hidden select-none">
      <Navbar
        activeTab={activeTab}
        onSelectTab={setActiveTab}
        stats={stats}
        onToggleIntercept={handleToggleIntercept}
      />

      <main className="flex-1 overflow-hidden">
        {activeTab === 'dashboard' && (
          <DashboardView
            stats={stats}
            recentFlows={flows}
            findings={findings}
            sitemap={sitemap}
            onNavigate={setActiveTab}
          />
        )}

        {activeTab === 'proxy' && (
          <ProxyInterceptView
            interceptEnabled={stats?.intercept_enabled ?? false}
            pendingRequests={pendingRequests}
            onToggleIntercept={handleToggleIntercept}
            onForward={handleForwardIntercept}
            onDrop={handleDropIntercept}
            onSendToRepeater={handleSendToRepeater}
          />
        )}

        {activeTab === 'history' && (
          <HistoryView
            flows={flows}
            onRefreshFlows={refreshAll}
            onSendToRepeater={handleSendToRepeater}
            onSendToComparer={handleSendToComparer}
          />
        )}

        {activeTab === 'repeater' && (
          <RepeaterView
            tabs={repeaterTabs}
            onRefreshTabs={refreshAll}
            initialRequest={repeaterInitialReq}
          />
        )}

        {activeTab === 'target' && (
          <TargetView sitemap={sitemap} onRefresh={refreshAll} />
        )}

        {activeTab === 'scanner' && (
          <ScannerView findings={findings} onRefresh={refreshAll} />
        )}

        {activeTab === 'decoder' && <DecoderView />}

        {activeTab === 'comparer' && <ComparerView />}

        {activeTab === 'settings' && <SettingsView />}
      </main>
    </div>
  );
};
