import { useState, useEffect } from 'react'
import './index.css'
import './App.css'
import CameraView from './components/CameraView'
import BottomControls from './components/BottomControls'
import SettingsPanel from './components/SettingsPanel'
import GalleryPanel from './components/GalleryPanel'

function App() {
  const [toastMessage, setToastMessage] = useState('');
  const [showToast, setShowToast] = useState(false);
  
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isGalleryOpen, setIsGalleryOpen] = useState(false);
  
  const [mirror, setMirror] = useState(() => localStorage.getItem('arcamera-mirror') !== 'false');
  const [grid, setGrid] = useState(() => localStorage.getItem('arcamera-grid') === 'true');
  
  const [mode, setMode] = useState('photo');
  const [isRecording, setIsRecording] = useState(false);

  useEffect(() => {
    localStorage.setItem('arcamera-mirror', mirror);
  }, [mirror]);

  useEffect(() => {
    localStorage.setItem('arcamera-grid', grid);
  }, [grid]);

  let toastTimer;
  const triggerToast = (msg) => {
    setToastMessage(msg);
    setShowToast(true);
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => setShowToast(false), 3000);
  };

  return (
    <div className="app-container animate-fade-in">
      <header className="top-app-bar">
        <h1 className="title">Arcamera</h1>
        <div className="actions">
          <button className="icon-button transparent-btn" onClick={() => setIsSettingsOpen(true)}>
            <span className="material-symbols-outlined">settings</span>
          </button>
        </div>
      </header>

      <CameraView 
        mirror={mirror} 
        grid={grid} 
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
      />

      <SettingsPanel 
        isOpen={isSettingsOpen} 
        onClose={() => setIsSettingsOpen(false)}
        mirror={mirror}
        setMirror={setMirror}
        grid={grid}
        setGrid={setGrid}
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
