import React, { useState } from 'react';
import { Binary, Copy, Check, RefreshCw } from 'lucide-react';
import { api } from '../api/client';

export const DecoderView: React.FC = () => {
  const [inputText, setInputText] = useState<string>('admin&token=xyz<script>alert(1)</script>');
  const [outputText, setOutputText] = useState<string>('');
  const [activeFormat, setActiveFormat] = useState<string>('url');
  const [copied, setCopied] = useState<boolean>(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const handleTransform = async (action: 'encode' | 'decode') => {
    setErrorMsg(null);
    try {
      if (action === 'encode') {
        const res = await api.encode(inputText, activeFormat);
        setOutputText(res);
      } else {
        const res = await api.decode(inputText, activeFormat);
        setOutputText(res);
      }
    } catch (err: any) {
      setErrorMsg(err.message);
    }
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(outputText);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  const handleSwap = () => {
    setInputText(outputText);
    setOutputText('');
  };

  return (
    <div className="p-6 space-y-6 h-[calc(100vh-3.5rem)] overflow-y-auto bg-dark-bg select-none">
      <div className="bg-dark-card border border-dark-border rounded-xl p-5">
        <h2 className="text-sm font-bold text-white flex items-center space-x-2">
          <Binary className="w-4 h-4 text-brand-primary" />
          <span>Decoder & Encoder Utilities</span>
        </h2>
        <p className="text-xs text-dark-muted mt-1">
          Fast multi-format conversion between raw text, URL encoding, Base64, Hexadecimal, and HTML entities.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 h-[calc(100%-7rem)]">
        {/* Input Panel */}
        <div className="flex flex-col bg-dark-card border border-dark-border rounded-xl p-4 space-y-3">
          <div className="flex items-center justify-between pb-2 border-b border-dark-border text-xs text-dark-muted font-semibold">
            <span className="text-white">Input Payload</span>
            <span>{inputText.length} characters</span>
          </div>

          <textarea
            value={inputText}
            onChange={(e) => setInputText(e.target.value)}
            className="flex-1 bg-dark-bg border border-dark-border rounded-lg p-3 font-mono text-xs text-dark-text outline-none focus:border-brand-primary resize-none leading-relaxed"
            placeholder="Type or paste payload to encode/decode..."
          />

          {/* Action Format Bar */}
          <div className="flex items-center justify-between pt-2">
            <div className="flex items-center space-x-1">
              {[
                { id: 'url', label: 'URL' },
                { id: 'base64', label: 'Base64' },
                { id: 'hex', label: 'Hex' },
                { id: 'html', label: 'HTML' },
              ].map((fmt) => (
                <button
                  key={fmt.id}
                  onClick={() => setActiveFormat(fmt.id)}
                  className={`px-2.5 py-1 rounded text-xs font-semibold ${
                    activeFormat === fmt.id
                      ? 'bg-brand-primary text-white'
                      : 'bg-dark-bg border border-dark-border text-dark-muted hover:text-dark-text'
                  }`}
                >
                  {fmt.label}
                </button>
              ))}
            </div>

            <div className="flex items-center space-x-2">
              <button
                onClick={() => handleTransform('decode')}
                className="px-3.5 py-1.5 rounded-lg text-xs font-bold bg-dark-cardHover border border-dark-border text-dark-text hover:border-dark-muted transition-colors"
              >
                Decode
              </button>
              <button
                onClick={() => handleTransform('encode')}
                className="px-3.5 py-1.5 rounded-lg text-xs font-bold bg-brand-primary text-white hover:bg-brand-hover transition-colors"
              >
                Encode
              </button>
            </div>
          </div>
        </div>

        {/* Output Panel */}
        <div className="flex flex-col bg-dark-card border border-dark-border rounded-xl p-4 space-y-3">
          <div className="flex items-center justify-between pb-2 border-b border-dark-border text-xs text-dark-muted font-semibold">
            <span className="text-white">Output Result</span>
            <div className="flex items-center space-x-2">
              {outputText && (
                <button
                  onClick={handleSwap}
                  className="flex items-center space-x-1 text-xs text-dark-muted hover:text-dark-text"
                  title="Use output as next input"
                >
                  <RefreshCw className="w-3 h-3" />
                  <span>Use as Input</span>
                </button>
              )}
              <button
                onClick={handleCopy}
                disabled={!outputText}
                className="flex items-center space-x-1 px-2 py-0.5 rounded text-xs bg-dark-bg border border-dark-border hover:border-dark-muted text-dark-text disabled:opacity-40"
              >
                {copied ? <Check className="w-3 h-3 text-brand-accent" /> : <Copy className="w-3 h-3" />}
                <span>{copied ? 'Copied' : 'Copy'}</span>
              </button>
            </div>
          </div>

          <textarea
            readOnly
            value={outputText}
            className="flex-1 bg-dark-bg border border-dark-border rounded-lg p-3 font-mono text-xs text-emerald-300 outline-none resize-none leading-relaxed"
            placeholder="Output will appear here..."
          />

          {errorMsg && (
            <div className="p-2.5 rounded bg-brand-danger/10 border border-brand-danger/30 text-xs text-brand-danger font-mono">
              Error: {errorMsg}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
