import './BottomControls.css';
import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

export default function BottomControls({ onGalleryClick, mode, setMode, isRecording, setIsRecording, triggerToast, fileFormat, fps, flashEnabled, mirrorOutput, burstCount, aspectRatio }) {
    const [recordingTime, setRecordingTime] = useState(0);
    const timerRef = useRef(null);

    useEffect(() => {
        if (isRecording) {
            setRecordingTime(0);
            timerRef.current = setInterval(() => {
                setRecordingTime(prev => prev + 1);
            }, 1000);
        } else {
            if (timerRef.current) {
                clearInterval(timerRef.current);
                timerRef.current = null;
            }
            setRecordingTime(0);
        }
        return () => {
            if (timerRef.current) clearInterval(timerRef.current);
        };
    }, [isRecording]);

    const formatTime = (seconds) => {
        const m = Math.floor(seconds / 60).toString().padStart(2, '0');
        const s = (seconds % 60).toString().padStart(2, '0');
        return `${m}:${s}`;
    };
    
    const handleCapture = async () => {
        if (mode === 'photo') {
            if (flashEnabled) {
                window.dispatchEvent(new Event('arcamera-flash'));
            }
            try {
                for (let i = 0; i < (burstCount || 1); i++) {
                    await invoke('save_photo', { 
                        format: fileFormat.toLowerCase(), 
                        mirrorOutput,
                        aspectRatio 
                    });
                    if (burstCount > 1 && i < burstCount - 1) {
                        // Wait 100ms between bursts (approx 10fps)
                        await new Promise(r => setTimeout(r, 100));
                    }
                }
                triggerToast(burstCount > 1 ? `Saved ${burstCount} photos!` : 'Photo saved!');
            } catch (error) {
                console.error('Save failed:', error);
                triggerToast('Failed to save: ' + error);
            }
        } else if (mode === 'video') {
            if (!isRecording) {
                try {
                    await invoke('start_recording', { targetFps: fps });
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
            {/* Info bar */}
            <div className="info-bar">
                {mode === 'photo' ? (
                    <>
                        <div className="info-chip">
                            <span className="info-chip-label">FMT</span>
                            <span className="info-chip-value">{fileFormat}</span>
                        </div>
                        <div className="info-chip">
                            <span className="info-chip-label">WB</span>
                            <span className="info-chip-value">AUTO</span>
                        </div>
                    </>
                ) : (
                    <>
                        <div className="info-chip">
                            <span className="info-chip-label">FPS</span>
                            <span className="info-chip-value">{fps}</span>
                        </div>
                        {isRecording && (
                            <div className="info-chip recording-chip">
                                <span className="rec-dot"></span>
                                <span className="info-chip-value">{formatTime(recordingTime)}</span>
                            </div>
                        )}
                    </>
                )}
            </div>

            {/* Main controls row */}
            <div className="controls-row">
                <button className="gallery-btn" onClick={onGalleryClick}>
                    <span className="material-symbols-outlined">photo_library</span>
                </button>
                
                <div className="capture-container">
                    <button 
                        className={`capture-btn ${isRecording ? 'recording' : ''}`} 
                        onClick={handleCapture}
                    >
                        <div className="inner-circle"></div>
                    </button>
                </div>

                <div className="controls-right">
                    <div className={`mode-toggle ${mode === 'video' ? 'mode-video-active' : ''}`}>
                        <div className="mode-slider"></div>
                        <button 
                            className={`mode-btn ${mode === 'photo' ? 'active' : ''}`} 
                            onClick={() => !isRecording && setMode('photo')}
                        >
                            <span className="material-symbols-outlined mode-icon">photo_camera</span>
                            Photo
                        </button>
                        <button 
                            className={`mode-btn ${mode === 'video' ? 'active' : ''}`} 
                            onClick={() => !isRecording && setMode('video')}
                        >
                            <span className="material-symbols-outlined mode-icon">videocam</span>
                            Video
                        </button>
                    </div>
                </div>
            </div>
        </footer>
    );
}
