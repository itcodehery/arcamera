import './BottomControls.css';
import { invoke } from '@tauri-apps/api/core';

export default function BottomControls({ onGalleryClick, mode, setMode, isRecording, setIsRecording, triggerToast }) {
    
    const handleCapture = async () => {
        if (mode === 'photo') {
            window.dispatchEvent(new Event('arcamera-flash'));
            try {
                await invoke('save_photo');
                triggerToast('Photo saved!');
            } catch (error) {
                console.error('Save failed:', error);
                triggerToast('Failed to save photo');
            }
        } else if (mode === 'video') {
            if (!isRecording) {
                try {
                    await invoke('start_recording');
                    setIsRecording(true);
                } catch (e) {
                    triggerToast('Failed to start recording: ' + e);
                }
            } else {
                setIsRecording(false);
                triggerToast('Processing video...');
                try {
                    await invoke('stop_recording');
                    triggerToast('Video saved!');
                } catch (e) {
                    triggerToast('Error: ' + e);
                }
            }
        }
    };

    return (
        <footer className="bottom-controls">
            <button className="icon-button" onClick={onGalleryClick}>
                <span className="material-symbols-outlined">photo_library</span>
            </button>
            
            <div className="capture-container">
                <button 
                    className={`fab capture-btn ${isRecording ? 'recording' : ''}`} 
                    onClick={handleCapture}
                >
                    <div className="inner-circle"></div>
                </button>
                <div className={`mode-toggle ${mode === 'video' ? 'mode-video-active' : ''}`}>
                    <div className="mode-slider"></div>
                    <button 
                        className={`mode-btn ${mode === 'photo' ? 'active' : ''}`} 
                        onClick={() => !isRecording && setMode('photo')}
                    >
                        Photo
                    </button>
                    <button 
                        className={`mode-btn ${mode === 'video' ? 'active' : ''}`} 
                        onClick={() => !isRecording && setMode('video')}
                    >
                        Video
                    </button>
                </div>
            </div>

            <div style={{ width: '48px' }}></div>
        </footer>
    );
}
