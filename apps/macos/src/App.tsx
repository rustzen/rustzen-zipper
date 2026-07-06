import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';
import { check } from '@tauri-apps/plugin-updater';
import { useEffect, useMemo, useState } from 'react';

type InspectSummary = {
  source: string;
  file_count: number;
  dir_count: number;
  compressed_size: number;
  uncompressed_size: number;
};

type UnpackSummary = {
  source: string;
  output_dir: string;
  extracted_entries: number;
};

export default function App() {
  const [source, setSource] = useState('');
  const [outputDir, setOutputDir] = useState('');
  const [inspect, setInspect] = useState<InspectSummary | null>(null);
  const [unpackResult, setUnpackResult] = useState<UnpackSummary | null>(null);
  const [status, setStatus] = useState('Drop a .zip file or choose one from Finder.');
  const [busy, setBusy] = useState(false);

  const canUnpack = useMemo(() => Boolean(source && outputDir && !busy), [source, outputDir, busy]);

  useEffect(() => {
    let disposed = false;
    let cleanup: undefined | (() => void);

    getCurrentWindow()
      .onDragDropEvent((event) => {
        if (event.payload.type === 'drop') {
          const firstPath = event.payload.paths[0];
          if (firstPath) {
            setSource(firstPath);
            setStatus('Archive selected from drag and drop.');
          }
        }
      })
      .then((unlisten) => {
        if (disposed) {
          unlisten();
        } else {
          cleanup = unlisten;
        }
      })
      .catch(() => setStatus('Drag and drop is not available in this runtime.'));

    return () => {
      disposed = true;
      cleanup?.();
    };
  }, []);

  async function chooseArchive() {
    const selected = await open({ multiple: false, filters: [{ name: 'ZIP archive', extensions: ['zip'] }] });
    if (typeof selected === 'string') {
      setSource(selected);
      setStatus('Archive selected.');
    }
  }

  async function chooseOutputDir() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === 'string') {
      setOutputDir(selected);
      setStatus('Output directory selected.');
    }
  }

  async function inspectArchive() {
    if (!source) return;
    setBusy(true);
    setUnpackResult(null);
    try {
      const result = await invoke<InspectSummary>('inspect_zip', { source });
      setInspect(result);
      setStatus('Archive inspected.');
    } catch (error) {
      setStatus(String(error));
    } finally {
      setBusy(false);
    }
  }

  async function unpackArchive() {
    if (!canUnpack) return;
    setBusy(true);
    try {
      const result = await invoke<UnpackSummary>('unpack_zip', { source, outputDir });
      setUnpackResult(result);
      setStatus('Archive extracted.');
    } catch (error) {
      setStatus(String(error));
    } finally {
      setBusy(false);
    }
  }

  async function revealOutput() {
    if (!outputDir) return;
    await invoke('open_path', { path: outputDir });
  }

  async function checkForUpdates() {
    setBusy(true);
    try {
      const update = await check();
      if (!update) {
        setStatus('No update available.');
        return;
      }
      setStatus(`Downloading ${update.version}...`);
      await update.downloadAndInstall();
      setStatus('Update installed. Restart the app to use the new version.');
    } catch (error) {
      setStatus(String(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="app-shell">
      <section className="hero">
        <div>
          <p className="eyebrow">Rustzen Zipper</p>
          <h1>Lightweight archive utility for macOS.</h1>
          <p className="subtitle">ZIP-first desktop shell powered by the existing Rust CLI core.</p>
        </div>
        <button className="ghost-button" onClick={checkForUpdates} disabled={busy}>Check updates</button>
      </section>

      <section className="drop-zone" onClick={chooseArchive}>
        <strong>{source ? source : 'Drop .zip here'}</strong>
        <span>Click to choose a ZIP archive.</span>
      </section>

      <section className="controls">
        <button onClick={chooseArchive} disabled={busy}>Choose archive</button>
        <button onClick={chooseOutputDir} disabled={busy}>Choose output</button>
        <button onClick={inspectArchive} disabled={!source || busy}>Inspect</button>
        <button className="primary" onClick={unpackArchive} disabled={!canUnpack}>Extract</button>
      </section>

      <section className="panel">
        <div>
          <span className="label">Output</span>
          <p>{outputDir || 'No output directory selected.'}</p>
        </div>
        {outputDir ? <button onClick={revealOutput}>Open in Finder</button> : null}
      </section>

      {inspect ? (
        <section className="stats-grid">
          <Stat label="Files" value={inspect.file_count} />
          <Stat label="Directories" value={inspect.dir_count} />
          <Stat label="Compressed" value={`${inspect.compressed_size} B`} />
          <Stat label="Original" value={`${inspect.uncompressed_size} B`} />
        </section>
      ) : null}

      {unpackResult ? (
        <section className="panel success">
          <strong>Extracted {unpackResult.extracted_entries} entries</strong>
          <span>{unpackResult.output_dir}</span>
        </section>
      ) : null}

      <footer>{status}</footer>
    </main>
  );
}

function Stat({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="stat-card">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
