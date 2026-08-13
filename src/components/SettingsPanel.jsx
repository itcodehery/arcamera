import './SettingsPanel.css';

const ASPECT_RATIOS = ['4:3', '1:1', '16:9'];
const FILE_FORMATS = ['JPG', 'PNG'];
const FPS_OPTIONS = [30, 60];

export default function SettingsPanel({ 
    isOpen, onClose, 
    mirror, setMirror, 
    grid, setGrid,
    aspectRatio, setAspectRatio,
    fileFormat, setFileFormat,
    fps, setFps,
    flashEnabled, setFlashEnabled
}) {
    return (
        <div className={`panel-overlay ${isOpen ? 'open' : ''}`}>
            <header className="panel-header">
                <button className="icon-button transparent-btn" onClick={onClose}>
                    <span className="material-symbols-outlined">arrow_back</span>
                </button>
                <h2 className="panel-title">Settings</h2>
                <div style={{ width: '48px' }}></div>
            </header>
            <div className="settings-content">
                <div className="settings-group">
                    <h3 className="settings-group-title">Camera</h3>
                    <div className="settings-item">
                        <div className="settings-item-text">
                            <span className="settings-label">Mirror preview</span>
                            <span className="settings-desc">Flip the camera horizontally</span>
                        </div>
                        <label className="switch">
                            <input 
                                type="checkbox" 
                                checked={mirror} 
                                onChange={(e) => setMirror(e.target.checked)} 
                            />
                            <span className="switch-track">
                                <span className="switch-thumb"></span>
                            </span>
                        </label>
                    </div>
                    <div className="settings-item">
                        <div className="settings-item-text">
                            <span className="settings-label">Grid overlay</span>
                            <span className="settings-desc">Show rule of thirds guide</span>
                        </div>
                        <label className="switch">
                            <input 
                                type="checkbox" 
                                checked={grid} 
                                onChange={(e) => setGrid(e.target.checked)} 
                            />
                            <span className="switch-track">
                                <span className="switch-thumb"></span>
                            </span>
                        </label>
                    </div>
                    <div className="settings-item">
                        <div className="settings-item-text">
                            <span className="settings-label">Flash mode</span>
                            <span className="settings-desc">Full-screen flash for photos</span>
                        </div>
                        <label className="switch">
                            <input 
                                type="checkbox" 
                                checked={flashEnabled} 
                                onChange={(e) => setFlashEnabled(e.target.checked)} 
                            />
                            <span className="switch-track">
                                <span className="switch-thumb"></span>
                            </span>
                        </label>
                    </div>
                </div>

                <div className="settings-group">
                    <h3 className="settings-group-title">Photo</h3>
                    <div className="settings-item">
                        <div className="settings-item-text">
                            <span className="settings-label">Aspect ratio</span>
                            <span className="settings-desc">Viewfinder crop ratio</span>
                        </div>
                        <div className="segmented-control">
                            {ASPECT_RATIOS.map((r) => (
                                <button
                                    key={r}
                                    className={`segment ${aspectRatio === r ? 'active' : ''}`}
                                    onClick={() => setAspectRatio(r)}
                                >
                                    {r}
                                </button>
                            ))}
                        </div>
                    </div>
                    <div className="settings-item">
                        <div className="settings-item-text">
                            <span className="settings-label">File format</span>
                            <span className="settings-desc">Output image format</span>
                        </div>
                        <div className="segmented-control">
                            {FILE_FORMATS.map((f) => (
                                <button
                                    key={f}
                                    className={`segment ${fileFormat === f ? 'active' : ''}`}
                                    onClick={() => setFileFormat(f)}
                                >
                                    {f}
                                </button>
                            ))}
                        </div>
                    </div>
                </div>

                <div className="settings-group">
                    <h3 className="settings-group-title">Video</h3>
                    <div className="settings-item">
                        <div className="settings-item-text">
                            <span className="settings-label">Frame rate</span>
                            <span className="settings-desc">Recording FPS target</span>
                        </div>
                        <div className="segmented-control">
                            {FPS_OPTIONS.map((f) => (
                                <button
                                    key={f}
                                    className={`segment ${fps === f ? 'active' : ''}`}
                                    onClick={() => setFps(f)}
                                >
                                    {f}
                                </button>
                            ))}
                        </div>
                    </div>
                </div>

                <div className="settings-group">
                    <h3 className="settings-group-title">Storage</h3>
                    <div className="settings-item">
                        <div className="settings-item-text">
                            <span className="settings-label">Photos</span>
                            <span className="settings-desc">~/Pictures/Arcamera</span>
                        </div>
                        <span className="material-symbols-outlined settings-icon">folder</span>
                    </div>
                    <div className="settings-item">
                        <div className="settings-item-text">
                            <span className="settings-label">Videos</span>
                            <span className="settings-desc">~/Videos/Arcamera</span>
                        </div>
                        <span className="material-symbols-outlined settings-icon">folder</span>
                    </div>
                </div>
                <div className="settings-group">
                    <h3 className="settings-group-title">About</h3>
                    <div className="settings-item">
                        <div className="settings-item-text">
                            <span className="settings-label">Arcamera</span>
                            <span className="settings-desc">Version 0.1.0</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
