import { useState, useEffect } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import './index.css'
import './App.css'
import CameraView from './components/CameraView'
import BottomControls from './components/BottomControls'
import SettingsPanel from './components/SettingsPanel'
import GalleryPanel from './components/GalleryPanel'

const ASPECT_RATIOS = ['4:3', '1:1', '16:9'];
const FILE_FORMATS = ['JPG', 'PNG'];
const FPS_OPTIONS = [30, 60];

function App() {
  const [toastMessage, setToastMessage] = useState('');
  const [showToast, setShowToast] = useState(false);
  
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isGalleryOpen, setIsGalleryOpen] = useState(false);
  
  const [mirror, setMirror] = useState(() => localStorage.getItem('arcamera-mirror') !== 'false');
  const [grid, setGrid] = useState(() => localStorage.getItem('arcamera-grid') === 'true');
  
  const [mode, setMode] = useState('photo');
  const [isRecording, setIsRecording] = useState(false);

  // Phase 3 state
  const [aspectRatio, setAspectRatio] = useState(() => localStorage.getItem('arcamera-aspect') || '4:3');
  const [fileFormat, setFileFormat] = useState(() => localStorage.getItem('arcamera-format') || 'JPG');
  const [fps, setFps] = useState(() => parseInt(localStorage.getItem('arcamera-fps')) || 30);
  const [flashEnabled, setFlashEnabled] = useState(() => localStorage.getItem('arcamera-flash') === 'true');

  useEffect(() => { localStorage.setItem('arcamera-mirror', mirror); }, [mirror]);
  useEffect(() => { localStorage.setItem('arcamera-grid', grid); }, [grid]);
  useEffect(() => { localStorage.setItem('arcamera-aspect', aspectRatio); }, [aspectRatio]);
  useEffect(() => { localStorage.setItem('arcamera-format', fileFormat); }, [fileFormat]);
  useEffect(() => { localStorage.setItem('arcamera-fps', fps); }, [fps]);
  useEffect(() => { localStorage.setItem('arcamera-flash', flashEnabled); }, [flashEnabled]);

  const cycleAspectRatio = () => {
    const idx = ASPECT_RATIOS.indexOf(aspectRatio);
    const next = ASPECT_RATIOS[(idx + 1) % ASPECT_RATIOS.length];
    setAspectRatio(next);
    triggerToast(`Aspect ratio: ${next}`);
  };

  const cycleFileFormat = () => {
    const idx = FILE_FORMATS.indexOf(fileFormat);
    const next = FILE_FORMATS[(idx + 1) % FILE_FORMATS.length];
    setFileFormat(next);
    triggerToast(`Format: ${next}`);
  };

  const cycleFps = () => {
    const idx = FPS_OPTIONS.indexOf(fps);
    const next = FPS_OPTIONS[(idx + 1) % FPS_OPTIONS.length];
    setFps(next);
    triggerToast(`FPS: ${next}`);
  };

  let toastTimer;
  const triggerToast = (msg) => {
    setToastMessage(msg);
    setShowToast(true);
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => setShowToast(false), 3000);
  };

  return (
    <div className="app-container animate-fade-in">
      <header className="top-app-bar" data-tauri-drag-region>
        <button className="icon-button transparent-btn" onClick={() => setIsSettingsOpen(true)}>
          <span className="material-symbols-outlined" style={{ pointerEvents: 'none' }}>settings</span>
        </button>

        <div className="top-bar-info">
          <button className="top-bar-toggle" onClick={cycleAspectRatio}>
            {aspectRatio}
          </button>
          <span className="separator">|</span>
          {mode === 'photo' ? (
            <button className="top-bar-toggle" onClick={cycleFileFormat}>
              {fileFormat}
            </button>
          ) : (
            <button className="top-bar-toggle" onClick={cycleFps}>
              {fps} FPS
            </button>
          )}
          <span className="separator">|</span>
          {mode === 'photo' && (
            <button 
              className={`top-bar-toggle ${flashEnabled ? 'toggle-active' : ''}`}
              onClick={() => { setFlashEnabled(!flashEnabled); triggerToast(flashEnabled ? 'Flash off' : 'Flash on'); }}
            >
              <span className="material-symbols-outlined" style={{ fontSize: '18px' }}>
                {flashEnabled ? 'flash_on' : 'flash_off'}
              </span>
            </button>
          )}
          {mode === 'video' && (
            <span className="top-bar-label">REC</span>
          )}
        </div>

        <div className="actions" data-tauri-drag-region>
          <button className="icon-button transparent-btn close-btn" onClick={() => getCurrentWindow().close()}>
            <span className="material-symbols-outlined" style={{ pointerEvents: 'none' }}>close</span>
          </button>
        </div>
      </header>

      <CameraView 
        mirror={mirror} 
        grid={grid}
        aspectRatio={aspectRatio}
        flashEnabled={flashEnabled}
      />

      <div className={`toast ${showToast ? 'show' : ''}`}>
        {toastMessage}
      </div>

      <BottomControls 
        onGalleryClick={() => setIsGalleryOpen(true)}
        mode={mode}
        setMode={setMode}
        isRecording={isRecording}
        setIsRecording={setIsRecording}
        triggerToast={triggerToast}
        fileFormat={fileFormat}
        fps={fps}
        flashEnabled={flashEnabled}
      />

      <SettingsPanel 
        isOpen={isSettingsOpen} 
        onClose={() => setIsSettingsOpen(false)}
        mirror={mirror}
        setMirror={setMirror}
        grid={grid}
        setGrid={setGrid}
        aspectRatio={aspectRatio}
        setAspectRatio={setAspectRatio}
        fileFormat={fileFormat}
        setFileFormat={setFileFormat}
        fps={fps}
        setFps={setFps}
        flashEnabled={flashEnabled}
        setFlashEnabled={setFlashEnabled}
      />
      
      <GalleryPanel 
        isOpen={isGalleryOpen} 
        onClose={() => setIsGalleryOpen(false)}
        triggerToast={triggerToast}
      />
    </div>
  )
}

export default App
