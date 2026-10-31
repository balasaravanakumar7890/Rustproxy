import React, { useState } from 'react';
import {
  Settings as SettingsIcon,
  Shield,
  Key,
  Copy,
  Check,
  Info,
} from 'lucide-react';

export const SettingsView: React.FC = () => {
  const [copied, setCopied] = useState<boolean>(false);
  const caPath = '~/.rustproxy/rustproxy_ca.crt';

  const handleCopyPath = () => {
    navigator.clipboard.writeText(caPath);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div className="p-6 space-y-6 h-[calc(100vh-3.5rem)] overflow-y-auto bg-dark-bg select-none max-w-4xl mx-auto">
      {/* Header */}
      <div className="bg-dark-card border border-dark-border rounded-xl p-5">
        <h2 className="text-sm font-bold text-white flex items-center space-x-2">
          <SettingsIcon className="w-4 h-4 text-brand-primary" />
          <span>Platform Settings & Trust Configuration</span>
        </h2>
        <p className="text-xs text-dark-muted mt-1">
          Configure proxy listener ports, upstream TLS options, and Root Certificate Authority trust.
        </p>
      </div>

      {/* Network Listener Settings */}
      <div className="bg-dark-card border border-dark-border rounded-xl p-5 space-y-4">
        <h3 className="text-xs font-bold text-white uppercase tracking-wider flex items-center space-x-2">
          <Shield className="w-4 h-4 text-brand-accent" />
          <span>Proxy Listener Settings</span>
        </h3>

        <div className="grid grid-cols-2 gap-4 text-xs">
          <div>
            <label className="text-dark-muted block mb-1">Bind Address</label>
            <input
              type="text"
              readOnly
              value="127.0.0.1"
              className="w-full bg-dark-bg border border-dark-border rounded-lg px-3 py-2 text-dark-text font-mono"
            />
          </div>

          <div>
            <label className="text-dark-muted block mb-1">Port</label>
            <input
              type="text"
              readOnly
              value="8080"
              className="w-full bg-dark-bg border border-dark-border rounded-lg px-3 py-2 text-dark-text font-mono"
            />
          </div>
        </div>

        <p className="text-[11px] text-dark-muted">
          RustProxy binds strictly to loopback (<code className="text-brand-primary">127.0.0.1</code>) to prevent external unauthorized access.
        </p>
      </div>

      {/* Root CA Certificate Trust */}
      <div className="bg-dark-card border border-dark-border rounded-xl p-5 space-y-4">
        <h3 className="text-xs font-bold text-white uppercase tracking-wider flex items-center space-x-2">
          <Key className="w-4 h-4 text-brand-warning" />
          <span>Root Certificate Authority (CA)</span>
        </h3>

        <div className="flex items-center space-x-3 text-xs">
          <div className="flex-1 bg-dark-bg border border-dark-border rounded-lg px-3 py-2 font-mono text-dark-text truncate">
            {caPath}
          </div>
          <button
            onClick={handleCopyPath}
            className="flex items-center space-x-1 px-3 py-2 rounded-lg bg-dark-cardHover border border-dark-border hover:border-dark-muted text-white font-semibold transition-colors"
          >
            {copied ? <Check className="w-3.5 h-3.5 text-brand-accent" /> : <Copy className="w-3.5 h-3.5" />}
            <span>{copied ? 'Copied' : 'Copy Path'}</span>
          </button>
        </div>

        <div className="space-y-2 pt-2 border-t border-dark-border/40 text-xs text-dark-text leading-relaxed">
          <h4 className="font-semibold text-white">Installing into Web Browsers:</h4>
          <ol className="list-decimal list-inside space-y-1 text-dark-muted">
            <li>
              <strong className="text-dark-text">Firefox:</strong> Settings &gt; Privacy &amp; Security &gt; Certificates &gt; View Certificates &gt; Authorities &gt; Import &gt; select <code className="text-brand-primary font-mono">{caPath}</code> &gt; check "Trust this CA to identify websites".
            </li>
            <li>
              <strong className="text-dark-text">Chrome / Chromium:</strong> Settings &gt; Privacy &amp; Security &gt; Security &gt; Manage Certificates &gt; Authorities &gt; Import &gt; select the certificate.
            </li>
          </ol>
        </div>
      </div>

      {/* About Box */}
      <div className="bg-dark-card border border-dark-border rounded-xl p-5 space-y-3">
        <h3 className="text-xs font-bold text-white uppercase tracking-wider flex items-center space-x-2">
          <Info className="w-4 h-4 text-brand-primary" />
          <span>Architecture & Safety Boundary</span>
        </h3>
        <p className="text-xs text-dark-muted leading-relaxed">
          RustProxy is built in Rust using Tokio, Hyper, Rustls, SQLite WAL mode, and React 19. It strictly enforces target scope boundaries, protects user session confidentiality, and does not engage in unauthorized third-party exploitation.
        </p>
      </div>
    </div>
  );
};
