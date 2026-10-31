import {
  DiffResult,
  FlowDetail,
  FlowSummary,
  ProxyStats,
  RepeaterTab,
  RequestModel,
  ResponseModel,
  SecurityFinding,
  SiteMapNode,
} from '../types';

const API_BASE =
  typeof window !== 'undefined' && window.location.port === '1420'
    ? 'http://127.0.0.1:8081'
    : '';

async function fetchJson<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const url = `${API_BASE}${endpoint}`;
  const res = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!res.ok) {
    let errorMsg = `HTTP ${res.status}`;
    try {
      const err = await res.json();
      if (err.error) errorMsg = err.error;
    } catch {
      // ignore
    }
    throw new Error(errorMsg);
  }

  return res.json() as Promise<T>;
}

export const api = {
  // Stats
  async getStats(): Promise<ProxyStats> {
    return fetchJson<ProxyStats>('/api/stats');
  },

  // Flows
  async getFlows(filter?: { search?: string; method?: string; in_scope?: boolean; limit?: number }): Promise<FlowSummary[]> {
    const params = new URLSearchParams();
    if (filter?.search) params.append('search', filter.search);
    if (filter?.method) params.append('method', filter.method);
    if (filter?.in_scope) params.append('in_scope', 'true');
    if (filter?.limit) params.append('limit', filter.limit.toString());
    const query = params.toString() ? `?${params.toString()}` : '';
    return fetchJson<FlowSummary[]>(`/api/flows${query}`);
  },

  async getFlowDetail(id: string): Promise<FlowDetail> {
    return fetchJson<FlowDetail>(`/api/flows/${id}`);
  },

  async clearFlows(): Promise<void> {
    await fetchJson('/api/flows/clear', { method: 'POST' });
  },

  // Intercept
  async getInterceptStatus(): Promise<{ enabled: boolean; pending: RequestModel[] }> {
    return fetchJson<{ enabled: boolean; pending: RequestModel[] }>('/api/intercept');
  },

  async toggleIntercept(enabled: boolean): Promise<void> {
    await fetchJson('/api/intercept/toggle', {
      method: 'POST',
      body: JSON.stringify({ enabled }),
    });
  },

  async forwardIntercept(id: string, modified?: RequestModel): Promise<boolean> {
    const res = await fetchJson<{ success: boolean }>('/api/intercept/forward', {
      method: 'POST',
      body: JSON.stringify({ id, modified }),
    });
    return res.success;
  },

  async dropIntercept(id: string): Promise<boolean> {
    const res = await fetchJson<{ success: boolean }>('/api/intercept/drop', {
      method: 'POST',
      body: JSON.stringify({ id }),
    });
    return res.success;
  },

  // Repeater
  async getRepeaterTabs(): Promise<RepeaterTab[]> {
    return fetchJson<RepeaterTab[]>('/api/repeater/tabs');
  },

  async createRepeaterTab(name?: string, request?: RequestModel): Promise<{ tab_id: string }> {
    return fetchJson<{ tab_id: string }>('/api/repeater/tabs', {
      method: 'POST',
      body: JSON.stringify({ name, request }),
    });
  },

  async duplicateRepeaterTab(id: string): Promise<{ tab_id: string }> {
    return fetchJson<{ tab_id: string }>(`/api/repeater/tabs/${id}/duplicate`, {
      method: 'POST',
    });
  },

  async closeRepeaterTab(id: string): Promise<void> {
    await fetchJson(`/api/repeater/tabs/${id}`, {
      method: 'DELETE',
    });
  },

  async sendRepeaterTab(id: string): Promise<{ response: ResponseModel; body_str: string }> {
    return fetchJson<{ response: ResponseModel; body_str: string }>(`/api/repeater/tabs/${id}/send`, {
      method: 'POST',
    });
  },

  // Sitemap
  async getSitemap(): Promise<SiteMapNode[]> {
    return fetchJson<SiteMapNode[]>('/api/sitemap');
  },

  // Scanner Findings
  async getFindings(): Promise<SecurityFinding[]> {
    return fetchJson<SecurityFinding[]>('/api/findings');
  },

  // Decoder
  async encode(text: string, format: string): Promise<string> {
    const res = await fetchJson<{ result: string }>('/api/decoder/encode', {
      method: 'POST',
      body: JSON.stringify({ text, format }),
    });
    return res.result;
  },

  async decode(text: string, format: string): Promise<string> {
    const res = await fetchJson<{ result: string }>('/api/decoder/decode', {
      method: 'POST',
      body: JSON.stringify({ text, format }),
    });
    return res.result;
  },

  // Comparer
  async diff(oldText: string, newText: string): Promise<DiffResult> {
    return fetchJson<DiffResult>('/api/comparer/diff', {
      method: 'POST',
      body: JSON.stringify({ old: oldText, new: newText }),
    });
  },
};
