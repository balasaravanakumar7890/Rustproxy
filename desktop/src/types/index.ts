export interface FlowSummary {
  id: string;
  timestamp: string;
  method: string;
  host: string;
  port: number;
  path: string;
  query?: string;
  status?: number;
  mime_type?: string;
  request_len: number;
  response_len?: number;
  duration_ms?: number;
  in_scope: boolean;
  comment?: string;
  highlight?: string;
  has_error: boolean;
}

export interface RequestModel {
  id: string;
  method: string;
  uri: string;
  version: string;
  headers: [string, string][];
  host: string;
  port: number;
  scheme: string;
  path: string;
  query?: string;
  body_length: number;
  body?: string;
}

export interface ResponseModel {
  id: string;
  status: number;
  status_text: string;
  version: string;
  headers: [string, string][];
  body_length: number;
  mime_type?: string;
  duration_ms: number;
  body?: string;
}

export interface FlowDetail {
  flow: {
    id: string;
    session_id: string;
    timestamp: string;
    request: RequestModel;
    response?: ResponseModel;
    client_addr?: string;
    in_scope: boolean;
    comment?: string;
    highlight?: string;
    error?: string;
  };
  request_body_str: string;
  response_body_str?: string;
}

export interface ProxyStats {
  total_flows: number;
  total_findings: number;
  intercept_enabled: boolean;
  pending_intercepts: number;
}

export interface RepeaterTab {
  id: string;
  name: string;
  request: RequestModel;
  response?: ResponseModel;
  follow_redirects: boolean;
  is_in_flight: boolean;
  history: [RequestModel, ResponseModel | null][];
}

export interface SiteMapNode {
  id: string;
  name: string;
  path: string;
  is_endpoint: boolean;
  flow_ids: string[];
  methods: string[];
  status_codes: number[];
  children: Record<string, SiteMapNode>;
}

export interface SecurityFinding {
  id: string;
  flow_id: string;
  title: string;
  description: string;
  severity: 'Info' | 'Low' | 'Medium' | 'High' | 'Critical';
  confidence: 'Tentative' | 'Firm' | 'Certain';
  cwe_id?: number;
  remediation?: string;
  evidence?: string;
  url: string;
  timestamp: string;
}

export interface DiffLine {
  tag: 'Equal' | 'Insert' | 'Delete';
  content: string;
  old_line?: number;
  new_line?: number;
}

export interface DiffResult {
  lines: DiffLine[];
  additions: number;
  deletions: number;
  similarity_ratio: number;
}
