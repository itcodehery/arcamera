import './SettingsPanel.css';

export default function SettingsPanel({ isOpen, onClose, mirror, setMirror, grid, setGrid }) {
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
