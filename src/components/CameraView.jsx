import './CameraView.css';
import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

const ASPECT_MAP = {
    '4:3': 4 / 3,
    '1:1': 1,
    '16:9': 16 / 9,
};

export default function CameraView({ mirror, grid, aspectRatio, flashEnabled }) {
    const imgRef = useRef(null);
    const [flash, setFlash] = useState(false);

    useEffect(() => {
        let active = true;
        let lastUrl = null;
        let port = 49283;

        // Query dynamic port from backend
        invoke('get_stream_port').then(p => {
            if (p) port = p;
        }).catch(() => {});

        let lastFetchTime = 0;
        const FRAME_INTERVAL = 1000 / 30; // ~33.3ms for 30fps camera cadence
        let animId = null;

        const updateFrame = async (timestamp) => {
            if (!active) return;

            if (timestamp - lastFetchTime >= FRAME_INTERVAL) {
                lastFetchTime = timestamp;
                try {
                    const response = await fetch(`http://127.0.0.1:${port}/frame`, {
                        cache: 'no-store'
                    });
                    if (response.ok && active) {
                        const blob = await response.blob();
                        const url = URL.createObjectURL(blob);
                        
                        if (imgRef.current) {
                            imgRef.current.src = url;
                        }
                        
                        // Revoke the old URL *after* assigning the new one to prevent flickering
                        if (lastUrl) {
                            URL.revokeObjectURL(lastUrl);
                        }
                        lastUrl = url;
                    }
                } catch {
                    // Ignore transient network errors
                }
            }
            
            if (active) {
                animId = requestAnimationFrame(updateFrame);
            }
        };

        // Start the loop
        animId = requestAnimationFrame(updateFrame);
        
        const handleFlash = () => {
            if (flashEnabled) {
                setFlash(true);
                setTimeout(() => setFlash(false), 3000);
            }
        };
        
        window.addEventListener('arcamera-flash', handleFlash);
        return () => {
            active = false;
            if (animId) cancelAnimationFrame(animId);
            if (lastUrl) URL.revokeObjectURL(lastUrl);
            window.removeEventListener('arcamera-flash', handleFlash);
        };
    }, [flashEnabled]);

    const ratio = ASPECT_MAP[aspectRatio] || 4 / 3;

    // Use max-width to constrain the viewport shape based on aspect ratio
    const containerStyle = {
        maxWidth: `calc((100vh - 280px) * ${ratio})`,
    };

    return (
        <div className="camera-wrapper">
            <div className="camera-container" style={containerStyle}>
                <img 
                    ref={imgRef}
                    id="camera-feed" 
                    style={{ transform: mirror ? 'translate(-50%, -50%) scaleX(-1)' : 'translate(-50%, -50%)' }}
                    alt="Camera Feed"
                />
                {grid && (
                    <div className="grid-overlay" style={{ display: 'block' }}>
                        <div className="grid-line grid-h" style={{ top: '33.33%' }}></div>
                        <div className="grid-line grid-h" style={{ top: '66.66%' }}></div>
                        <div className="grid-line grid-v" style={{ left: '33.33%' }}></div>
                        <div className="grid-line grid-v" style={{ left: '66.66%' }}></div>
                    </div>
                )}
                
                {flash && (
                    <div className="flash-overlay"></div>
                )}
            </div>
        </div>
    );
}
